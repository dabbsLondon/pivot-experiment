# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

### Rust

```bash
# Build entire workspace
cargo build --workspace

# Release build (optimized, required for large data generation)
cargo build --release

# Run tests
cargo test --workspace --no-fail-fast

# Run a single test
cargo test test_name

# Format and lint
cargo fmt --check
cargo clippy --workspace
```

### Data Generation

```bash
# Basic usage
cargo run -p pivot-data-gen -- --rows 1000 --output trades.csv

# With ETF/ETC constituent explosion
cargo run -p pivot-data-gen -- --rows 1000 --explode-constituents --output trades.csv

# Full reference data export
cargo run -p pivot-data-gen -- \
  --rows 5000 \
  --portfolio-managers 20 \
  --explode-constituents \
  --output trades.csv \
  --instruments-output instruments.csv \
  --constituents-output constituents.csv
```

### Infrastructure and Database

```bash
# Start local infrastructure (ClickHouse + Redis)
docker compose up -d

# Initialize database schema
pnpm db:reset

# Create rollup tables
pnpm db:rollups

# Run API server
pnpm dev:api
```

## Architecture

This is a financial data pivot table platform for analyzing trade data across multiple dimensions.

### Workspace Structure

- **Rust workspace** with two packages:
  - `services/api` (`pivot-api`) - REST API server (scaffold phase)
  - `tools/data-gen` (`pivot-data-gen`) - Synthetic trade data generator

- **JavaScript monorepo** (pnpm workspaces):
  - `apps/web` - React web application (scaffold phase)

### Data Model

Three main ClickHouse tables in `sql/clickhouse/`:

1. **`trades_1d`** - Main fact table with 55 columns including organizational hierarchy, geography, instrument details, greeks, and custom metrics. Partitioned by `trade_date`.

2. **`instruments`** - Reference data for tradeable instruments (stocks, ETFs, ETCs, commodities). Contains `is_composite` flag for composite instruments.

3. **`constituents`** - Maps composite instruments (ETFs/ETCs) to underlying holdings with weights.

Pre-computed aggregations via `trades_1d_rollup` materialized view.

### Pivoting Dimensions

- **Organizational**: Portfolio Manager → Fund → Portfolio → Desk → Book
- **Instrument**: Asset Class → Product Type → Symbol
- **Geographic**: Region → Country → Venue
- **Exposure type**: Direct, ETF, ETC, Constituent (enables look-through analytics)

### ETF/ETC Constituent Explosion

The `--explode-constituents` flag expands composite instrument trades into constituent rows, enabling look-through analysis. The `exposure_type` field differentiates between direct trades and constituent exposures.
