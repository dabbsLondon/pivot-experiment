#!/bin/bash
set -e

# Benchmark Configuration
SIZES=(1000 10000 100000 500000 1000000)
API_URL="http://localhost:8080"
RESULTS_FILE="benchmark_results.json"
REPORT_FILE="docs/benchmark-report.md"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date +%H:%M:%S)]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[$(date +%H:%M:%S)]${NC} $1"
}

# Initialize results JSON
echo '{"benchmarks": [], "cache_tests": [], "grouping_tests": []}' > $RESULTS_FILE

# Function to time an API call and extract query_time_ms
benchmark_call() {
    local name=$1
    local method=$2
    local endpoint=$3
    local data=$4
    local size=$5

    local start_time=$(python3 -c 'import time; print(int(time.time() * 1000))')

    if [ "$method" == "GET" ]; then
        response=$(curl -s -w "\n%{time_total}" "$API_URL$endpoint")
    else
        response=$(curl -s -w "\n%{time_total}" -X POST -H "Content-Type: application/json" -d "$data" "$API_URL$endpoint")
    fi

    local end_time=$(python3 -c 'import time; print(int(time.time() * 1000))')
    local total_time=$((end_time - start_time))

    # Extract curl time (last line)
    local curl_time=$(echo "$response" | tail -1)
    local curl_time_ms=$(python3 -c "print(int(float('$curl_time') * 1000))")

    # Extract query_time_ms from response if present
    local query_time=$(echo "$response" | head -n -1 | python3 -c "import sys, json; d=json.load(sys.stdin); print(d.get('metadata', {}).get('query_time_ms', 0))" 2>/dev/null || echo "0")

    # Extract cached status
    local cached=$(echo "$response" | head -n -1 | python3 -c "import sys, json; d=json.load(sys.stdin); print(str(d.get('metadata', {}).get('cached', False)).lower())" 2>/dev/null || echo "false")

    # Extract row count
    local rows=$(echo "$response" | head -n -1 | python3 -c "import sys, json; d=json.load(sys.stdin); print(d.get('metadata', {}).get('returned_rows', 0))" 2>/dev/null || echo "0")

    echo "$name,$size,$total_time,$curl_time_ms,$query_time,$cached,$rows"
}

# Function to clear Redis cache
clear_cache() {
    docker exec pivot-redis redis-cli FLUSHALL > /dev/null 2>&1
}

# Function to truncate and reload data
load_data() {
    local size=$1
    local data_dir="data/benchmark"

    mkdir -p $data_dir

    log "Generating $size rows..."

    # Generate data with constituent explosion
    cargo run -p pivot-data-gen --release -- \
        --rows $size \
        --portfolio-managers 50 \
        --output "$data_dir/trades_$size.csv" \
        --instruments-output "$data_dir/instruments.csv" \
        --constituents-output "$data_dir/constituents.csv" \
        --explode-constituents \
        --seed 42 2>/dev/null

    log "Truncating tables..."
    curl -s "$API_URL/../" --data "TRUNCATE TABLE pivot.trades_1d" > /dev/null 2>&1 || \
        docker exec pivot-clickhouse clickhouse-client --query "TRUNCATE TABLE pivot.trades_1d" 2>/dev/null

    log "Loading $size rows into ClickHouse..."

    # Load instruments
    docker exec -i pivot-clickhouse clickhouse-client --query \
        "INSERT INTO pivot.instruments FORMAT CSVWithNames" < "$data_dir/instruments.csv" 2>/dev/null

    # Load constituents
    docker exec -i pivot-clickhouse clickhouse-client --query \
        "INSERT INTO pivot.constituents FORMAT CSVWithNames" < "$data_dir/constituents.csv" 2>/dev/null

    # Load trades
    docker exec -i pivot-clickhouse clickhouse-client --query \
        "INSERT INTO pivot.trades_1d FORMAT CSVWithNames" < "$data_dir/trades_$size.csv" 2>/dev/null

    # Verify count
    local count=$(docker exec pivot-clickhouse clickhouse-client --query "SELECT count() FROM pivot.trades_1d" 2>/dev/null)
    log "Loaded $count rows into trades_1d"
}

# Start API server if not running
start_api() {
    if ! curl -s "$API_URL/health" > /dev/null 2>&1; then
        log "Starting API server..."
        cargo run -p pivot-api --release &
        API_PID=$!
        sleep 3
    fi
}

# Main benchmark function
run_benchmarks() {
    local size=$1

    log "=== Benchmarking with $size rows ==="

    # Clear cache before benchmarks
    clear_cache

    # Health check
    benchmark_call "health" "GET" "/health" "" $size

    # Pivot with 1 dimension
    benchmark_call "pivot_1dim" "POST" "/api/v1/pivot" \
        '{"dimensions":["asset_class"],"metrics":["notional","pnl"],"filters":{"trade_date":"2024-01-15"}}' $size

    # Pivot with 2 dimensions
    benchmark_call "pivot_2dim" "POST" "/api/v1/pivot" \
        '{"dimensions":["asset_class","region"],"metrics":["notional","pnl"],"filters":{"trade_date":"2024-01-15"}}' $size

    # Pivot with 3 dimensions
    benchmark_call "pivot_3dim" "POST" "/api/v1/pivot" \
        '{"dimensions":["asset_class","region","desk"],"metrics":["notional","pnl"],"filters":{"trade_date":"2024-01-15"}}' $size

    # Pivot with 4 dimensions
    benchmark_call "pivot_4dim" "POST" "/api/v1/pivot" \
        '{"dimensions":["asset_class","region","desk","book"],"metrics":["notional","pnl"],"filters":{"trade_date":"2024-01-15"}}' $size

    # Pivot with 5 dimensions
    benchmark_call "pivot_5dim" "POST" "/api/v1/pivot" \
        '{"dimensions":["asset_class","region","desk","book","symbol"],"metrics":["notional","pnl"],"filters":{"trade_date":"2024-01-15"}}' $size

    # Exposure endpoint
    benchmark_call "exposure" "GET" "/api/v1/exposure?trade_date=2024-01-15&group_by=asset_class" "" $size

    # P&L endpoint
    benchmark_call "pnl" "GET" "/api/v1/pnl?trade_date=2024-01-15&group_by=portfolio_manager_id" "" $size

    # Instruments
    benchmark_call "instruments" "GET" "/api/v1/instruments" "" $size

    # Constituents
    benchmark_call "constituents" "GET" "/api/v1/constituents" "" $size
}

# Cache effectiveness test
test_cache() {
    local size=$1

    log "=== Testing Redis Cache with $size rows ==="

    # Clear cache
    clear_cache

    # First call (cache miss)
    local miss=$(benchmark_call "pivot_cache_miss" "POST" "/api/v1/pivot" \
        '{"dimensions":["asset_class","region"],"metrics":["notional","pnl"],"filters":{"trade_date":"2024-01-15"}}' $size)
    echo "$miss"

    # Second call (cache hit)
    local hit=$(benchmark_call "pivot_cache_hit" "POST" "/api/v1/pivot" \
        '{"dimensions":["asset_class","region"],"metrics":["notional","pnl"],"filters":{"trade_date":"2024-01-15"}}' $size)
    echo "$hit"

    # With cache_bypass
    local bypass=$(benchmark_call "pivot_cache_bypass" "POST" "/api/v1/pivot" \
        '{"dimensions":["asset_class","region"],"metrics":["notional","pnl"],"filters":{"trade_date":"2024-01-15"},"cache_bypass":true}' $size)
    echo "$bypass"
}

# Main execution
main() {
    log "Starting Pivot API Benchmark Suite"
    log "=================================="

    # Ensure API is running
    start_api

    # Wait for API to be ready
    sleep 2

    # Check API health
    if ! curl -s "$API_URL/health" > /dev/null 2>&1; then
        echo "Error: API server not responding"
        exit 1
    fi

    log "API server is ready"

    # Create CSV output file
    CSV_FILE="data/benchmark/results.csv"
    mkdir -p data/benchmark
    echo "test_name,data_size,total_time_ms,curl_time_ms,query_time_ms,cached,rows_returned" > $CSV_FILE

    # Run benchmarks for each size
    for size in "${SIZES[@]}"; do
        load_data $size

        # Run main benchmarks
        run_benchmarks $size >> $CSV_FILE

        # Run cache tests
        echo "" >> $CSV_FILE
        test_cache $size >> $CSV_FILE
    done

    log "Benchmarks complete! Results saved to $CSV_FILE"

    # Generate report
    log "Generating report..."
}

main "$@"
