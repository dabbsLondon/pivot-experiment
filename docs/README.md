# Documentation

Technical documentation for the Pivot Experiment platform.

## Contents

| Document | Description |
|----------|-------------|
| [Getting Started](./getting-started.md) | Step-by-step guide to generate data and load into ClickHouse |
| [Data Model](./data-model.md) | Schema design, tables, constituent relationships, and query patterns |
| [Data Generator](./data-generator.md) | CLI tool usage, options, and loading data into ClickHouse |
| [Large-Scale Generation](./large-scale-data-generation.md) | Plan for generating 20M+ row datasets |
| [Project Roadmap](./roadmap.md) | Detailed delivery plan with phases and action items |

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Pivot Platform                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────┐        ┌─────────────┐        ┌─────────────────────┐    │
│   │   Web App   │──HTTP──│  API Server │──────▶│     ClickHouse      │    │
│   │  apps/web   │        │ services/api│        │   (OLAP Storage)    │    │
│   └─────────────┘        └──────┬──────┘        └─────────────────────┘    │
│                                 │                         ▲                 │
│                                 │                         │                 │
│                          ┌──────▼──────┐        ┌─────────┴─────────┐      │
│                          │    Redis    │        │   Data Generator  │      │
│                          │   (Cache)   │        │  tools/data-gen   │      │
│                          └─────────────┘        └───────────────────┘      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Data Flow

1. **Data Generation**: The `pivot-data-gen` tool creates synthetic trade data with realistic financial characteristics
2. **Storage**: Trade data is loaded into ClickHouse, partitioned by trade date
3. **API Layer**: The Rust API service queries ClickHouse and caches results in Redis
4. **Web Interface**: React-based UI for interactive pivot table exploration

## Key Concepts

### Composite Instruments

ETFs (Exchange-Traded Funds) and ETCs (Exchange-Traded Commodities) are "composite" instruments that hold baskets of underlying securities or commodities.

**Example**: Buying SPY (S&P 500 ETF) gives you exposure to 500+ stocks

The platform supports "look-through" analysis by exploding trades in composite instruments into their constituent exposures:

```
Portfolio View (Aggregated):
├── SPY:     $1,000,000
├── QQQ:     $500,000
└── AAPL:    $200,000

Look-Through View (Exploded):
├── AAPL:    $200,000 (direct) + $70,000 (via SPY) + $60,000 (via QQQ) = $330,000
├── MSFT:    $65,000 (via SPY) + $50,000 (via QQQ) = $115,000
├── GOOGL:   $40,000 (via SPY) + $40,000 (via QQQ) = $80,000
└── ...
```

### Multi-Dimensional Pivoting

The schema supports pivoting across multiple dimensions:

- **Organizational**: Portfolio Manager → Fund → Portfolio → Account → Desk → Book
- **Geographic**: Region → Country → Venue
- **Instrument**: Asset Class → Product → Instrument Type → Symbol
- **Time**: Date → Timestamp

## Quick Start

```bash
# Start infrastructure
docker compose up -d

# Initialize database
pnpm db:reset
pnpm db:rollups

# Generate sample data
cargo run -p pivot-data-gen -- --rows 10000 --output sample.csv

# Run API
pnpm dev:api

# Run web app
pnpm dev:web
```
