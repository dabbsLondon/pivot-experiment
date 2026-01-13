#!/usr/bin/env python3
"""
Pivot API Benchmark Suite

Tests API performance with datasets from 1K to 1M rows,
including multi-dimensional pivot queries and cache effectiveness.
"""

import subprocess
import requests
import time
import json
import os
from dataclasses import dataclass, asdict
from typing import List, Dict, Optional
import statistics

API_URL = "http://localhost:8080"
DATA_DIR = "data/benchmark"

# Dataset sizes to test
SIZES = [1_000, 10_000, 100_000, 500_000, 1_000_000]

# Number of iterations for averaging
ITERATIONS = 3


@dataclass
class BenchmarkResult:
    test_name: str
    data_size: int
    total_time_ms: float
    query_time_ms: float
    cached: bool
    rows_returned: int
    iteration: int


def log(msg: str):
    print(f"[{time.strftime('%H:%M:%S')}] {msg}")


def clear_cache():
    """Clear Redis cache"""
    subprocess.run(
        ["docker", "exec", "pivot-redis", "redis-cli", "FLUSHALL"],
        capture_output=True
    )


def get_row_count() -> int:
    """Get current row count from ClickHouse"""
    result = subprocess.run(
        ["docker", "exec", "pivot-clickhouse", "clickhouse-client",
         "--query", "SELECT count() FROM pivot.trades_1d"],
        capture_output=True, text=True
    )
    return int(result.stdout.strip()) if result.stdout.strip() else 0


def generate_data(size: int):
    """Generate test data using pivot-data-gen"""
    os.makedirs(DATA_DIR, exist_ok=True)

    log(f"Generating {size:,} rows...")
    result = subprocess.run([
        "cargo", "run", "-p", "pivot-data-gen", "--release", "--",
        "--rows", str(size),
        "--portfolio-managers", "50",
        "--output", f"{DATA_DIR}/trades_{size}.csv",
        "--instruments-output", f"{DATA_DIR}/instruments.csv",
        "--constituents-output", f"{DATA_DIR}/constituents.csv",
        "--explode-constituents",
        "--seed", "42"
    ], capture_output=True, text=True)

    if result.returncode != 0:
        log(f"Error generating data: {result.stderr}")
        return False
    return True


def load_data(size: int):
    """Load data into ClickHouse"""
    log(f"Loading {size:,} rows into ClickHouse...")

    # Truncate tables
    subprocess.run([
        "docker", "exec", "pivot-clickhouse", "clickhouse-client",
        "--query", "TRUNCATE TABLE pivot.trades_1d"
    ], capture_output=True)

    subprocess.run([
        "docker", "exec", "pivot-clickhouse", "clickhouse-client",
        "--query", "TRUNCATE TABLE pivot.instruments"
    ], capture_output=True)

    subprocess.run([
        "docker", "exec", "pivot-clickhouse", "clickhouse-client",
        "--query", "TRUNCATE TABLE pivot.constituents"
    ], capture_output=True)

    # Load instruments
    with open(f"{DATA_DIR}/instruments.csv", "rb") as f:
        subprocess.run([
            "docker", "exec", "-i", "pivot-clickhouse", "clickhouse-client",
            "--query", "INSERT INTO pivot.instruments FORMAT CSVWithNames"
        ], stdin=f, capture_output=True)

    # Load constituents
    with open(f"{DATA_DIR}/constituents.csv", "rb") as f:
        subprocess.run([
            "docker", "exec", "-i", "pivot-clickhouse", "clickhouse-client",
            "--query", "INSERT INTO pivot.constituents FORMAT CSVWithNames"
        ], stdin=f, capture_output=True)

    # Load trades
    with open(f"{DATA_DIR}/trades_{size}.csv", "rb") as f:
        subprocess.run([
            "docker", "exec", "-i", "pivot-clickhouse", "clickhouse-client",
            "--query", "INSERT INTO pivot.trades_1d FORMAT CSVWithNames"
        ], stdin=f, capture_output=True)

    count = get_row_count()
    log(f"Loaded {count:,} rows")
    return count


def benchmark_request(method: str, endpoint: str, data: Optional[dict] = None) -> dict:
    """Time a single API request"""
    url = f"{API_URL}{endpoint}"

    start = time.perf_counter()
    if method == "GET":
        response = requests.get(url)
    else:
        response = requests.post(url, json=data)
    end = time.perf_counter()

    total_time_ms = (end - start) * 1000

    result = response.json()
    metadata = result.get("metadata", {})

    return {
        "total_time_ms": total_time_ms,
        "query_time_ms": metadata.get("query_time_ms", 0),
        "cached": metadata.get("cached", False),
        "rows_returned": metadata.get("returned_rows", 0)
    }


def run_pivot_benchmarks(size: int, iteration: int) -> List[BenchmarkResult]:
    """Run pivot queries with 1-5 dimensions"""
    results = []

    dimension_sets = [
        ["asset_class"],
        ["asset_class", "region"],
        ["asset_class", "region", "desk"],
        ["asset_class", "region", "desk", "book"],
        ["asset_class", "region", "desk", "book", "symbol"],
    ]

    for dims in dimension_sets:
        data = {
            "dimensions": dims,
            "metrics": ["notional", "pnl"],
            "filters": {"trade_date": "2024-01-15"}
        }

        r = benchmark_request("POST", "/api/v1/pivot", data)
        results.append(BenchmarkResult(
            test_name=f"pivot_{len(dims)}dim",
            data_size=size,
            total_time_ms=r["total_time_ms"],
            query_time_ms=r["query_time_ms"],
            cached=r["cached"],
            rows_returned=r["rows_returned"],
            iteration=iteration
        ))

    return results


def run_endpoint_benchmarks(size: int, iteration: int) -> List[BenchmarkResult]:
    """Run benchmarks on all endpoints"""
    results = []

    # Health
    start = time.perf_counter()
    requests.get(f"{API_URL}/health")
    end = time.perf_counter()
    results.append(BenchmarkResult(
        test_name="health",
        data_size=size,
        total_time_ms=(end - start) * 1000,
        query_time_ms=0,
        cached=False,
        rows_returned=0,
        iteration=iteration
    ))

    # Exposure
    r = benchmark_request("GET", "/api/v1/exposure?trade_date=2024-01-15&group_by=asset_class")
    results.append(BenchmarkResult(
        test_name="exposure",
        data_size=size,
        total_time_ms=r["total_time_ms"],
        query_time_ms=r["query_time_ms"],
        cached=r["cached"],
        rows_returned=r["rows_returned"],
        iteration=iteration
    ))

    # P&L
    r = benchmark_request("GET", "/api/v1/pnl?trade_date=2024-01-15&group_by=portfolio_manager_id")
    results.append(BenchmarkResult(
        test_name="pnl",
        data_size=size,
        total_time_ms=r["total_time_ms"],
        query_time_ms=r["query_time_ms"],
        cached=r["cached"],
        rows_returned=r["rows_returned"],
        iteration=iteration
    ))

    # Instruments
    r = benchmark_request("GET", "/api/v1/instruments")
    results.append(BenchmarkResult(
        test_name="instruments",
        data_size=size,
        total_time_ms=r["total_time_ms"],
        query_time_ms=r["query_time_ms"],
        cached=r["cached"],
        rows_returned=r["rows_returned"],
        iteration=iteration
    ))

    # Constituents
    r = benchmark_request("GET", "/api/v1/constituents")
    results.append(BenchmarkResult(
        test_name="constituents",
        data_size=size,
        total_time_ms=r["total_time_ms"],
        query_time_ms=r["query_time_ms"],
        cached=r["cached"],
        rows_returned=r["rows_returned"],
        iteration=iteration
    ))

    return results


def run_cache_test(size: int) -> List[dict]:
    """Test Redis cache effectiveness"""
    results = []

    clear_cache()

    data = {
        "dimensions": ["asset_class", "region"],
        "metrics": ["notional", "pnl"],
        "filters": {"trade_date": "2024-01-15"}
    }

    # First call - cache miss
    r1 = benchmark_request("POST", "/api/v1/pivot", data)
    results.append({
        "test": "cache_miss",
        "size": size,
        "total_time_ms": r1["total_time_ms"],
        "query_time_ms": r1["query_time_ms"],
        "cached": r1["cached"]
    })

    # Second call - cache hit
    r2 = benchmark_request("POST", "/api/v1/pivot", data)
    results.append({
        "test": "cache_hit",
        "size": size,
        "total_time_ms": r2["total_time_ms"],
        "query_time_ms": r2["query_time_ms"],
        "cached": r2["cached"]
    })

    # Third call - cache bypass
    data["cache_bypass"] = True
    r3 = benchmark_request("POST", "/api/v1/pivot", data)
    results.append({
        "test": "cache_bypass",
        "size": size,
        "total_time_ms": r3["total_time_ms"],
        "query_time_ms": r3["query_time_ms"],
        "cached": r3["cached"]
    })

    return results


def aggregate_results(results: List[BenchmarkResult]) -> Dict:
    """Aggregate results by test_name and data_size"""
    aggregated = {}

    for r in results:
        key = (r.test_name, r.data_size)
        if key not in aggregated:
            aggregated[key] = {
                "test_name": r.test_name,
                "data_size": r.data_size,
                "total_times": [],
                "query_times": [],
                "rows_returned": r.rows_returned
            }
        aggregated[key]["total_times"].append(r.total_time_ms)
        aggregated[key]["query_times"].append(r.query_time_ms)

    # Calculate averages
    summary = []
    for key, data in aggregated.items():
        summary.append({
            "test_name": data["test_name"],
            "data_size": data["data_size"],
            "avg_total_ms": round(statistics.mean(data["total_times"]), 2),
            "min_total_ms": round(min(data["total_times"]), 2),
            "max_total_ms": round(max(data["total_times"]), 2),
            "avg_query_ms": round(statistics.mean(data["query_times"]), 2),
            "rows_returned": data["rows_returned"]
        })

    return sorted(summary, key=lambda x: (x["data_size"], x["test_name"]))


def generate_report(all_results: List[BenchmarkResult], cache_results: List[dict]):
    """Generate markdown benchmark report"""
    summary = aggregate_results(all_results)

    report = """# Pivot API Benchmark Report

## Test Environment

- **Date**: {date}
- **Dataset Sizes**: 1K, 10K, 100K, 500K, 1M rows
- **Iterations**: {iterations} per test (averaged)
- **Pivot Dimensions Tested**: 1 to 5 levels
- **Infrastructure**: ClickHouse + Redis (Docker)

## Summary

This report benchmarks the Pivot API across different dataset sizes, testing:
1. Multi-dimensional pivot queries (1-5 dimension groupings)
2. Endpoint performance (health, exposure, pnl, instruments, constituents)
3. Redis cache effectiveness

---

## Pivot Query Performance by Dimension Count

| Dimensions | 1K rows | 10K rows | 100K rows | 500K rows | 1M rows |
|------------|---------|----------|-----------|-----------|---------|
""".format(date=time.strftime("%Y-%m-%d %H:%M"), iterations=ITERATIONS)

    # Build pivot timing table
    for dim in range(1, 6):
        row = f"| {dim} dimension{'s' if dim > 1 else '':<3} |"
        for size in SIZES:
            match = next((s for s in summary if s["test_name"] == f"pivot_{dim}dim" and s["data_size"] == size), None)
            if match:
                row += f" {match['avg_total_ms']:.1f}ms |"
            else:
                row += " N/A |"
        report += row + "\n"

    report += """
## Endpoint Performance (Average Total Time)

| Endpoint | 1K rows | 10K rows | 100K rows | 500K rows | 1M rows |
|----------|---------|----------|-----------|-----------|---------|
"""

    # Build endpoint timing table
    for endpoint in ["health", "exposure", "pnl", "instruments", "constituents"]:
        row = f"| {endpoint:<12} |"
        for size in SIZES:
            match = next((s for s in summary if s["test_name"] == endpoint and s["data_size"] == size), None)
            if match:
                row += f" {match['avg_total_ms']:.1f}ms |"
            else:
                row += " N/A |"
        report += row + "\n"

    report += """
## ClickHouse Query Time (Server-side)

| Test | 1K rows | 10K rows | 100K rows | 500K rows | 1M rows |
|------|---------|----------|-----------|-----------|---------|
"""

    # Query time table
    for test in ["pivot_1dim", "pivot_3dim", "pivot_5dim", "exposure", "pnl"]:
        row = f"| {test:<12} |"
        for size in SIZES:
            match = next((s for s in summary if s["test_name"] == test and s["data_size"] == size), None)
            if match:
                row += f" {match['avg_query_ms']:.1f}ms |"
            else:
                row += " N/A |"
        report += row + "\n"

    report += """
## Redis Cache Effectiveness

| Dataset Size | Cache Miss | Cache Hit | Speedup | Cache Bypass |
|--------------|------------|-----------|---------|--------------|
"""

    # Cache results table
    for size in SIZES:
        miss = next((c for c in cache_results if c["test"] == "cache_miss" and c["size"] == size), None)
        hit = next((c for c in cache_results if c["test"] == "cache_hit" and c["size"] == size), None)
        bypass = next((c for c in cache_results if c["test"] == "cache_bypass" and c["size"] == size), None)

        if miss and hit and bypass:
            speedup = miss["total_time_ms"] / hit["total_time_ms"] if hit["total_time_ms"] > 0 else 0
            report += f"| {size:>12,} | {miss['total_time_ms']:.1f}ms | {hit['total_time_ms']:.1f}ms | {speedup:.1f}x | {bypass['total_time_ms']:.1f}ms |\n"

    report += """
## Key Findings

### Performance Characteristics

1. **Query Scaling**: Analyze how query times scale with data size
2. **Dimension Impact**: Impact of adding more grouping dimensions
3. **Cache Benefit**: Redis cache provides significant speedup for repeated queries

### Recommendations

Based on the benchmark results:
- Redis caching is {cache_status}
- Query performance {perf_status}

---

## Detailed Results

```json
{json_results}
```

---

*Generated by Pivot API Benchmark Suite*
"""

    # Determine cache status
    if cache_results:
        avg_speedup = sum(
            (next((c for c in cache_results if c["test"] == "cache_miss" and c["size"] == s), {"total_time_ms": 1})["total_time_ms"] /
             max(next((c for c in cache_results if c["test"] == "cache_hit" and c["size"] == s), {"total_time_ms": 1})["total_time_ms"], 0.1))
            for s in SIZES if any(c["size"] == s for c in cache_results)
        ) / len(SIZES)
        cache_status = f"effective with {avg_speedup:.1f}x average speedup"
    else:
        cache_status = "not tested"

    # Performance status
    if summary:
        max_1m = max((s["avg_total_ms"] for s in summary if s["data_size"] == 1_000_000), default=0)
        if max_1m < 1000:
            perf_status = f"is excellent (max {max_1m:.0f}ms at 1M rows)"
        elif max_1m < 5000:
            perf_status = f"is good (max {max_1m:.0f}ms at 1M rows)"
        else:
            perf_status = f"may need optimization (max {max_1m:.0f}ms at 1M rows)"
    else:
        perf_status = "not measured"

    report = report.format(
        cache_status=cache_status,
        perf_status=perf_status,
        json_results=json.dumps(summary[:10], indent=2)  # First 10 results
    )

    return report


def main():
    log("=" * 60)
    log("Pivot API Benchmark Suite")
    log("=" * 60)

    # Check API health
    try:
        r = requests.get(f"{API_URL}/health")
        health = r.json()
        log(f"API Status: {health['status']}")
        log(f"ClickHouse: {health['clickhouse']['status']} ({health['clickhouse']['latency_ms']}ms)")
        log(f"Redis: {health['redis']['status']} ({health['redis']['latency_ms']}ms)")
    except Exception as e:
        log(f"Error: API not responding - {e}")
        return

    all_results = []
    cache_results = []

    for size in SIZES:
        log("")
        log(f"{'='*60}")
        log(f"BENCHMARKING {size:,} ROWS")
        log(f"{'='*60}")

        # Generate and load data
        if not generate_data(size):
            log(f"Failed to generate data for {size} rows, skipping...")
            continue

        load_data(size)

        # Clear cache before benchmarks
        clear_cache()

        # Run benchmarks
        for iteration in range(1, ITERATIONS + 1):
            log(f"Iteration {iteration}/{ITERATIONS}...")
            clear_cache()  # Clear cache between iterations

            # Pivot benchmarks
            pivot_results = run_pivot_benchmarks(size, iteration)
            all_results.extend(pivot_results)

            # Endpoint benchmarks
            endpoint_results = run_endpoint_benchmarks(size, iteration)
            all_results.extend(endpoint_results)

        # Cache test (once per size)
        log("Testing cache effectiveness...")
        cache_test = run_cache_test(size)
        cache_results.extend(cache_test)

        # Print quick summary
        summary = aggregate_results([r for r in all_results if r.data_size == size])
        log(f"Quick Summary for {size:,} rows:")
        for s in summary[:5]:
            log(f"  {s['test_name']}: {s['avg_total_ms']:.1f}ms avg")

    # Generate report
    log("")
    log("Generating report...")
    report = generate_report(all_results, cache_results)

    os.makedirs("docs", exist_ok=True)
    with open("docs/benchmark-report.md", "w") as f:
        f.write(report)

    log(f"Report saved to docs/benchmark-report.md")

    # Save raw results
    with open(f"{DATA_DIR}/results.json", "w") as f:
        json.dump({
            "benchmarks": [asdict(r) for r in all_results],
            "cache_tests": cache_results
        }, f, indent=2)

    log(f"Raw results saved to {DATA_DIR}/results.json")
    log("")
    log("=" * 60)
    log("BENCHMARK COMPLETE")
    log("=" * 60)


if __name__ == "__main__":
    main()
