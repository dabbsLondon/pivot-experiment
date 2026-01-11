# Getting Started: Data Generation Guide

A step-by-step guide to generating financial trade data and loading it into ClickHouse.

## Overview

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           DATA GENERATION WORKFLOW                               │
└─────────────────────────────────────────────────────────────────────────────────┘

  STEP 1              STEP 2              STEP 3              STEP 4
  ──────              ──────              ──────              ──────

┌──────────┐      ┌──────────────┐      ┌──────────────┐      ┌──────────────┐
│  Setup   │─────▶│   Generate   │─────▶│   Validate   │─────▶│  Load to DB  │
│ Infra    │      │     Data     │      │    Files     │      │              │
└──────────┘      └──────────────┘      └──────────────┘      └──────────────┘
     │                   │                    │                     │
     ▼                   ▼                    ▼                     ▼
  Docker            CSV Files            Check rows            ClickHouse
  ClickHouse        trades.csv           columns               Ready for
  Redis             instruments.csv      integrity             queries
                    constituents.csv
```

---

## Prerequisites

Before you begin, ensure you have:

| Requirement | Version | Check Command |
|-------------|---------|---------------|
| Docker | 20+ | `docker --version` |
| Docker Compose | v2+ | `docker compose version` |
| Rust | 1.70+ | `rustc --version` |
| Cargo | 1.70+ | `cargo --version` |
| pnpm | 9.0+ | `pnpm --version` |
| Node.js | 18+ | `node --version` |

---

## Step 1: Setup Infrastructure

Start the required services (ClickHouse and Redis).

```bash
# Clone the repository (if not already done)
git clone <repository-url>
cd pivot-experiment

# Start Docker services
docker compose up -d
```

**What this starts:**

```
┌─────────────────────────────────────────────────────────────────┐
│                    Docker Compose Services                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌───────────────────────────────┐    ┌──────────────────────┐ │
│   │        ClickHouse 24.2        │    │      Redis 7.2       │ │
│   │                               │    │                      │ │
│   │   Port 8123 (HTTP API)        │    │   Port 6379          │ │
│   │   Port 9000 (Native TCP)      │    │                      │ │
│   │                               │    │   Used for caching   │ │
│   │   OLAP database for           │    │   API responses      │ │
│   │   trade analytics             │    │                      │ │
│   └───────────────────────────────┘    └──────────────────────┘ │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Verify services are running:**

```bash
docker compose ps

# Expected output:
# NAME        STATUS
# clickhouse  running
# redis       running
```

---

## Step 2: Initialize Database Schema

Create the database tables and rollups.

```bash
# Create database and tables
pnpm db:reset

# Create pre-aggregated rollup tables
pnpm db:rollups
```

**Schema created:**

```
┌─────────────────────────────────────────────────────────────────┐
│                       pivot Database                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌───────────────────────┐    ┌─────────────────────────────┐  │
│   │  pivot.instruments    │    │     pivot.constituents      │  │
│   │                       │    │                             │  │
│   │  Reference data for   │───▶│  Maps ETFs/ETCs to their    │  │
│   │  29 instruments       │    │  underlying holdings        │  │
│   │  (stocks, ETFs, etc.) │    │  (24 mappings)              │  │
│   └───────────────────────┘    └─────────────────────────────┘  │
│              │                              │                    │
│              │         JOIN ON symbol       │                    │
│              ▼                              ▼                    │
│   ┌──────────────────────────────────────────────────────────┐  │
│   │                   pivot.trades_1d                         │  │
│   │                                                           │  │
│   │   Main fact table with 55 columns                         │  │
│   │   Partitioned by trade_date                               │  │
│   │   Optimized for analytical queries                        │  │
│   └──────────────────────────────────────────────────────────┘  │
│              │                                                   │
│              │  Materialized View                                │
│              ▼                                                   │
│   ┌──────────────────────────────────────────────────────────┐  │
│   │               pivot.trades_1d_rollup                      │  │
│   │                                                           │  │
│   │   Pre-aggregated summaries for fast dashboard queries     │  │
│   └──────────────────────────────────────────────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Verify schema:**

```bash
docker compose exec clickhouse clickhouse-client \
  --query "SHOW TABLES FROM pivot"

# Expected output:
# constituents
# instruments
# trades_1d
# trades_1d_rollup
# trades_1d_rollup_mv
```

---

## Step 3: Build the Data Generator

Compile the data generator for optimal performance.

```bash
# Build in release mode (optimized)
cargo build -p pivot-data-gen --release
```

This compiles the Rust data generator with optimizations enabled.

---

## Step 4: Generate Data

Choose a generation scenario based on your needs:

### Option A: Quick Test (1K rows)

```bash
cargo run -p pivot-data-gen --release -- \
  --rows 1000 \
  --output data/trades.csv \
  --instruments-output data/instruments.csv \
  --constituents-output data/constituents.csv
```

**Output:** ~500KB, generates in <1 second

### Option B: Development Dataset (10K rows)

```bash
cargo run -p pivot-data-gen --release -- \
  --rows 10000 \
  --portfolio-managers 10 \
  --output data/trades.csv \
  --instruments-output data/instruments.csv \
  --constituents-output data/constituents.csv
```

**Output:** ~5MB, generates in <1 second

### Option C: With Constituent Explosion (10K rows + look-through)

```bash
cargo run -p pivot-data-gen --release -- \
  --rows 10000 \
  --portfolio-managers 10 \
  --explode-constituents \
  --output data/trades.csv \
  --instruments-output data/instruments.csv \
  --constituents-output data/constituents.csv
```

**Output:** ~10MB (~18K rows after explosion)

### Option D: Production Test (100K rows)

```bash
cargo run -p pivot-data-gen --release -- \
  --rows 100000 \
  --portfolio-managers 50 \
  --explode-constituents \
  --output data/trades.csv \
  --instruments-output data/instruments.csv \
  --constituents-output data/constituents.csv
```

**Output:** ~130MB, generates in ~1 second

### Option E: Large Scale (20M rows)

See [Large-Scale Data Generation](./large-scale-data-generation.md) for detailed instructions.

---

## Understanding Constituent Explosion

When `--explode-constituents` is enabled, ETF/ETC trades generate additional rows:

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                          CONSTITUENT EXPLOSION                                   │
└─────────────────────────────────────────────────────────────────────────────────┘

  WITHOUT --explode-constituents          WITH --explode-constituents
  ─────────────────────────────           ─────────────────────────────

  Trade: Buy 1000 SPY                     Trade: Buy 1000 SPY
                                                    │
  ┌────────────────────────┐              ┌────────┴────────┐
  │  1 row                 │              │                 │
  │                        │              ▼                 ▼
  │  symbol: SPY           │        ┌──────────┐    ┌──────────────────┐
  │  exposure_type: ETF    │        │ ETF row  │    │ 8 Constituent    │
  │  quantity: 1000        │        │          │    │ rows             │
  │                        │        │ SPY      │    │                  │
  └────────────────────────┘        │ Direct   │    │ AAPL (7.0%)      │
                                    │ 1000 qty │    │ MSFT (6.5%)      │
                                    └──────────┘    │ GOOGL (4.0%)     │
                                                    │ AMZN (3.5%)      │
                                                    │ META (3.0%)      │
                                                    │ NVDA (2.5%)      │
                                                    │ TSLA (2.0%)      │
                                                    │ V (1.5%)         │
                                                    └──────────────────┘

  1 row total                       9 rows total (1 ETF + 8 constituents)
```

**Data multiplier by instrument:**

| Instrument | Type | Constituents | Rows per Trade |
|------------|------|--------------|----------------|
| AAPL, MSFT | Stock | 0 | 1 |
| SPY | ETF | 8 | 9 |
| QQQ | ETF | 7 | 8 |
| XLF | ETF | 3 | 4 |
| XLE | ETF | 1 | 2 |
| GLD | ETC | 1 | 2 |
| SLV | ETC | 1 | 2 |
| USO | ETC | 1 | 2 |
| PMET | ETC | 3 | 4 |

**Average multiplier:** ~1.86x (10K trades → ~18.6K rows)

---

## Step 5: Validate Generated Data

Before loading, verify the generated files:

```bash
# Check file sizes
ls -lh data/

# Count rows (including header)
wc -l data/trades.csv

# Count columns
head -1 data/trades.csv | tr ',' '\n' | wc -l
# Expected: 55

# Preview first few rows
head -3 data/trades.csv

# Check exposure type distribution (if using explode-constituents)
cut -d',' -f19 data/trades.csv | sort | uniq -c | sort -rn
```

**Expected output structure:**

```
data/
├── trades.csv           # Main trade data (55 columns)
├── instruments.csv      # 29 instrument definitions
└── constituents.csv     # 24 ETF/ETC to constituent mappings
```

---

## Step 6: Load Data into ClickHouse

Load the generated CSV files into the database.

```bash
# Create data directory if needed
mkdir -p data

# Load instruments (reference data)
docker compose exec -T clickhouse clickhouse-client \
  --query "INSERT INTO pivot.instruments FORMAT CSVWithNames" \
  < data/instruments.csv

# Load constituents (ETF/ETC mappings)
docker compose exec -T clickhouse clickhouse-client \
  --query "INSERT INTO pivot.constituents FORMAT CSVWithNames" \
  < data/constituents.csv

# Load trades (main fact table)
docker compose exec -T clickhouse clickhouse-client \
  --query "INSERT INTO pivot.trades_1d FORMAT CSVWithNames" \
  < data/trades.csv
```

**Data flow:**

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           DATA LOADING FLOW                                      │
└─────────────────────────────────────────────────────────────────────────────────┘

    Local Filesystem                      Docker Container
    ────────────────                      ────────────────

┌────────────────────┐                 ┌──────────────────────────────────┐
│ data/              │                 │          ClickHouse              │
│ ├── instruments.csv│────stdin───────▶│  ┌────────────────────────────┐ │
│ ├── constituents.csv────stdin───────▶│  │      pivot database        │ │
│ └── trades.csv     │────stdin───────▶│  │                            │ │
└────────────────────┘                 │  │  instruments ← 29 rows     │ │
                                       │  │  constituents ← 24 rows    │ │
                                       │  │  trades_1d ← N rows        │ │
                                       │  │                            │ │
                                       │  │  trades_1d_rollup          │ │
                                       │  │  (auto-populated via MV)   │ │
                                       │  └────────────────────────────┘ │
                                       └──────────────────────────────────┘
```

---

## Step 7: Verify Data Load

Confirm data was loaded correctly.

```bash
# Count rows in each table
docker compose exec clickhouse clickhouse-client --query \
  "SELECT 'instruments' as table, count() as rows FROM pivot.instruments
   UNION ALL
   SELECT 'constituents', count() FROM pivot.constituents
   UNION ALL
   SELECT 'trades_1d', count() FROM pivot.trades_1d
   UNION ALL
   SELECT 'rollup', count() FROM pivot.trades_1d_rollup"

# Check exposure type distribution
docker compose exec clickhouse clickhouse-client --query \
  "SELECT exposure_type, count() as cnt, round(sum(notional), 2) as total_notional
   FROM pivot.trades_1d
   GROUP BY exposure_type
   ORDER BY cnt DESC"

# Sample query: P&L by portfolio manager
docker compose exec clickhouse clickhouse-client --query \
  "SELECT portfolio_manager_id, sum(pnl) as total_pnl
   FROM pivot.trades_1d
   GROUP BY portfolio_manager_id
   ORDER BY total_pnl DESC
   LIMIT 5"
```

---

## Complete Workflow Diagram

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                       END-TO-END DATA PIPELINE                                   │
└─────────────────────────────────────────────────────────────────────────────────┘

                                       YOU ARE HERE
                                            │
    ┌───────┐   ┌────────┐   ┌──────────┐  ▼  ┌──────────┐   ┌─────────────────┐
    │ Clone │──▶│ Docker │──▶│ DB       │────▶│ Generate │──▶│ Load & Query    │
    │ Repo  │   │ Up     │   │ Schema   │     │ Data     │   │                 │
    └───────┘   └────────┘   └──────────┘     └──────────┘   └─────────────────┘
        │           │             │                │                 │
        ▼           ▼             ▼                ▼                 ▼
    git clone   docker       pnpm db:reset    cargo run -p     clickhouse-client
                compose up   pnpm db:rollups  pivot-data-gen   INSERT INTO

                                                    │
                                                    ▼
                                            ┌──────────────┐
                                            │   3 Files    │
                                            │              │
                                            │ trades.csv   │
                                            │ instruments  │
                                            │ constituents │
                                            └──────────────┘
```

---

## Troubleshooting

### Docker services not starting

```bash
# Check logs
docker compose logs clickhouse
docker compose logs redis

# Restart services
docker compose down
docker compose up -d
```

### Schema errors during load

```bash
# Reset and recreate schema
docker compose exec clickhouse clickhouse-client \
  --query "DROP DATABASE IF EXISTS pivot"

pnpm db:reset
pnpm db:rollups
```

### "Table not found" errors

```bash
# Verify tables exist
docker compose exec clickhouse clickhouse-client \
  --query "SHOW DATABASES"

docker compose exec clickhouse clickhouse-client \
  --query "SHOW TABLES FROM pivot"
```

### Data generation is slow

```bash
# Ensure you're using release build
cargo build -p pivot-data-gen --release

# Check if using --release flag
cargo run -p pivot-data-gen --release -- --rows 1000
```

---

## Next Steps

- [Data Model](./data-model.md) - Understand the schema and query patterns
- [Data Generator CLI Reference](./data-generator.md) - All CLI options
- [Large-Scale Generation](./large-scale-data-generation.md) - Generating 20M+ rows
- [Project Roadmap](./roadmap.md) - What's coming next
