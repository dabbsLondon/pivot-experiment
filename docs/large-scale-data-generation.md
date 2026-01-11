# Large-Scale Data Generation Plan

This document outlines the process for generating a 20 million row test dataset for performance testing and benchmarking.

## Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     Large-Scale Data Generation Pipeline                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌───────────┐ │
│  │   Configure  │───▶│   Generate   │───▶│   Validate   │───▶│   Load    │ │
│  │  Parameters  │    │     Data     │    │    Output    │    │   to DB   │ │
│  └──────────────┘    └──────────────┘    └──────────────┘    └───────────┘ │
│                                                                             │
│  Parameters:          Output:             Checks:            Target:        │
│  • 20M trades         • trades.csv        • Row count        • ClickHouse  │
│  • 100 PMs            • instruments.csv   • Column count     • Partitioned │
│  • Explode ETFs       • constituents.csv  • File integrity   • Indexed     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Dataset Specifications

| Parameter | Value | Notes |
|-----------|-------|-------|
| Base Trades | 20,000,000 | Before constituent explosion |
| Portfolio Managers | 100 | Realistic enterprise scale |
| Constituent Explosion | Enabled | ETF/ETC look-through |
| Expected Total Rows | ~37,000,000 | 1.86x multiplier from ETF explosion |
| Columns | 55 | Full schema with exposure_type |
| Random Seed | 42 | For reproducibility |
| Trade Date | 2024-01-15 | Single day dataset |

## Output Files

| File | Expected Size | Description |
|------|---------------|-------------|
| `trades_20m.csv` | ~26 GB | Main trade data with constituents |
| `instruments.csv` | ~3 KB | 29 instrument definitions |
| `constituents.csv` | ~2 KB | 24 constituent mappings |

## Time Estimates

Based on benchmarks (M1 Mac, release build):

| Rows | Time | File Size |
|------|------|-----------|
| 100,000 | 0.6s | 130 MB |
| 1,000,000 | 6s | 1.3 GB |
| 10,000,000 | 60s | 13 GB |
| **20,000,000** | **~2 min** | **~26 GB** |

## Generation Command

```bash
# Build optimized binary
cargo build -p pivot-data-gen --release

# Generate 20M row dataset
time cargo run -p pivot-data-gen --release -- \
  --rows 20000000 \
  --portfolio-managers 100 \
  --explode-constituents \
  --seed 42 \
  --trade-date 2024-01-15 \
  --output data/trades_20m.csv \
  --instruments-output data/instruments.csv \
  --constituents-output data/constituents.csv

# Verify output
wc -l data/trades_20m.csv
head -1 data/trades_20m.csv | tr ',' '\n' | wc -l
ls -lh data/
```

## Expected Output

```
Generated 37200000 rows in 122000ms
  - 17200000 constituent exposure rows
Output size: 27234567890 bytes

data/
├── trades_20m.csv      (26 GB)
├── instruments.csv     (3 KB)
└── constituents.csv    (2 KB)
```

## Data Distribution

### By Exposure Type

| Type | % of Rows | Description |
|------|-----------|-------------|
| Direct | ~40% | Direct stock/commodity trades |
| ETF | ~10% | ETF wrapper trades |
| ETC | ~10% | ETC wrapper trades |
| Constituent | ~40% | Exploded underlying exposures |

### By Asset Class

| Class | % of Rows |
|-------|-----------|
| Equity | ~65% |
| Commodity | ~35% |

### By Instrument Type

| Type | Count |
|------|-------|
| Stocks | 15 |
| ETFs | 4 (SPY, QQQ, XLF, XLE) |
| ETCs | 4 (GLD, SLV, USO, PMET) |
| Commodities | 6 |

## Validation Checklist

After generation, verify:

- [ ] Row count is ~37M (20M × 1.86)
- [ ] Column count is 55
- [ ] File size is ~26 GB
- [ ] All exposure_type values present: Direct, ETF, ETC, Constituent
- [ ] Constituent rows have valid parent_symbol
- [ ] No NULL or empty required fields

```bash
# Validation commands
echo "Row count:"
wc -l data/trades_20m.csv

echo "Column count:"
head -1 data/trades_20m.csv | tr ',' '\n' | wc -l

echo "Exposure type distribution:"
cut -d',' -f19 data/trades_20m.csv | sort | uniq -c | sort -rn

echo "Sample rows by type:"
grep ",Direct," data/trades_20m.csv | head -2
grep ",ETF," data/trades_20m.csv | head -2
grep ",Constituent," data/trades_20m.csv | head -2
```

## Loading into ClickHouse

```bash
# Start ClickHouse
docker compose up -d clickhouse

# Wait for startup
sleep 5

# Initialize schema
docker compose exec clickhouse clickhouse-client \
  --multiquery \
  --queries-file /sql/clickhouse/001_schema.sql

# Load instruments
docker compose exec -T clickhouse clickhouse-client \
  --query "INSERT INTO pivot.instruments FORMAT CSVWithNames" \
  < data/instruments.csv

# Load constituents
docker compose exec -T clickhouse clickhouse-client \
  --query "INSERT INTO pivot.constituents FORMAT CSVWithNames" \
  < data/constituents.csv

# Load trades (this will take a few minutes)
echo "Loading 37M rows into ClickHouse..."
time docker compose exec -T clickhouse clickhouse-client \
  --query "INSERT INTO pivot.trades_1d FORMAT CSVWithNames" \
  < data/trades_20m.csv

# Verify load
docker compose exec clickhouse clickhouse-client \
  --query "SELECT count() FROM pivot.trades_1d"

docker compose exec clickhouse clickhouse-client \
  --query "SELECT exposure_type, count() as cnt FROM pivot.trades_1d GROUP BY exposure_type ORDER BY cnt DESC"
```

## Storage Considerations

### Git LFS Required

The 26GB file is too large for regular Git. Use Git LFS:

```bash
# Install Git LFS
git lfs install

# Track large CSV files
git lfs track "data/*.csv"
git add .gitattributes

# Add and commit data
git add data/
git commit -m "Add 20M row test dataset"
git push
```

### Alternative: Compressed Storage

```bash
# Compress for storage (reduces to ~3-4 GB)
gzip -k data/trades_20m.csv

# Decompress for use
gunzip -k data/trades_20m.csv.gz
```

## Process Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          Data Generation Flow                                │
└─────────────────────────────────────────────────────────────────────────────┘

     ┌─────────────────────────────────────────────────────────────────┐
     │                      pivot-data-gen                              │
     │                                                                  │
     │  ┌──────────────┐   ┌──────────────┐   ┌──────────────────────┐ │
     │  │ Instruments  │   │ Constituents │   │    Trade Generator   │ │
     │  │  (29 items)  │   │  (24 items)  │   │     (20M trades)     │ │
     │  └──────┬───────┘   └──────┬───────┘   └──────────┬───────────┘ │
     │         │                  │                      │             │
     └─────────┼──────────────────┼──────────────────────┼─────────────┘
               │                  │                      │
               ▼                  ▼                      ▼
     ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────────────┐
     │ instruments.csv │ │constituents.csv │ │    trades_20m.csv       │
     │     (3 KB)      │ │     (2 KB)      │ │       (26 GB)           │
     └─────────────────┘ └─────────────────┘ └─────────────────────────┘
                                                        │
                                                        │ ETF/ETC trades
                                                        │ get exploded
                                                        ▼
                                             ┌─────────────────────────┐
                                             │   Constituent Rows      │
                                             │   (17M additional)      │
                                             │                         │
                                             │ SPY trade → 8 rows      │
                                             │ QQQ trade → 7 rows      │
                                             │ GLD trade → 1 row       │
                                             │ PMET trade → 3 rows     │
                                             └─────────────────────────┘
                                                        │
                                                        ▼
                                             ┌─────────────────────────┐
                                             │   Total: ~37M rows      │
                                             │                         │
                                             │ • 20M base trades       │
                                             │ • 17M constituent rows  │
                                             └─────────────────────────┘
```

## Next Steps After Generation

1. **Verify data integrity** using validation commands above
2. **Load into ClickHouse** for query testing
3. **Run benchmark queries** to establish baseline performance
4. **Test pivot operations** across different dimensions
5. **Profile query performance** with EXPLAIN and system tables
