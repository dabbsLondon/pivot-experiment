# Data Generator

The `pivot-data-gen` tool generates synthetic financial trade data for testing and development.

## Installation

```bash
cargo build -p pivot-data-gen --release
```

## Usage

```bash
# Basic usage - generate 1000 trades to stdout
cargo run -p pivot-data-gen

# Generate 10,000 trades to a file
cargo run -p pivot-data-gen -- --rows 10000 --output trades.csv

# Generate with constituent explosion (ETF/ETC look-through)
cargo run -p pivot-data-gen -- --rows 1000 --explode-constituents --output trades.csv

# Generate all reference data files
cargo run -p pivot-data-gen -- \
  --rows 5000 \
  --portfolio-managers 20 \
  --output trades.csv \
  --instruments-output instruments.csv \
  --constituents-output constituents.csv \
  --explode-constituents
```

## Options

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--rows` | `-r` | 1000 | Number of trades to generate |
| `--portfolio-managers` | `-p` | 10 | Number of portfolio managers |
| `--output` | `-o` | stdout | Output file for trades CSV |
| `--instruments-output` | | none | Output file for instruments reference data |
| `--constituents-output` | | none | Output file for constituent mappings |
| `--seed` | `-s` | 42 | Random seed for reproducibility |
| `--trade-date` | | 2024-01-15 | Trade date (YYYY-MM-DD) |
| `--explode-constituents` | | false | Generate constituent exposure rows for ETFs/ETCs |

## Output Files

### Trades CSV

The main output containing trade records with 54 columns:

- **Identification**: trade_date, ts, trade_id, order_id
- **Organization**: portfolio_manager_id, fund_id, portfolio_id, account_id, desk, book, strategy
- **Geography**: region, country, venue
- **Instrument**: asset_class, product, instrument_type, symbol, parent_symbol, currency, counterparty
- **Risk**: risk_bucket, scenario, is_constituent_exposure, weight
- **Values**: quantity, price, notional, pnl
- **Greeks**: delta, gamma, vega, theta, rho
- **Other**: margin, fees, slippage, vol, rate, exposure, metric_1 through metric_14

### Instruments CSV

Reference data for all instruments (29 instruments):

| Type | Count | Examples |
|------|-------|----------|
| Stocks | 15 | AAPL, MSFT, GOOGL, AMZN, META, NVDA, TSLA, JPM, V, JNJ, WMT, XOM, BAC, PG, HD |
| Commodities | 6 | GOLD, SILVER, PLAT, COPPER, CRUDEOIL, NATGAS |
| ETFs | 4 | SPY, QQQ, XLF, XLE |
| ETCs | 4 | GLD, SLV, USO, PMET |

### Constituents CSV

Mappings from composite instruments to their holdings:

```csv
parent_symbol,constituent_symbol,weight,shares_per_unit,effective_date
SPY,AAPL,0.07,0.45,2024-01-15
SPY,MSFT,0.065,0.25,2024-01-15
QQQ,AAPL,0.12,0.78,2024-01-15
GLD,GOLD,1.0,0.093,2024-01-15
PMET,GOLD,0.50,0.05,2024-01-15
PMET,SILVER,0.30,0.50,2024-01-15
PMET,PLAT,0.20,0.02,2024-01-15
```

## Constituent Explosion

When `--explode-constituents` is enabled, trades in composite instruments generate additional rows:

```
Trade: Buy 1000 SPY
├── Row 1: SPY trade (is_constituent_exposure=0, weight=1.0)
├── Row 2: AAPL exposure (is_constituent_exposure=1, weight=0.07, parent_symbol=SPY)
├── Row 3: MSFT exposure (is_constituent_exposure=1, weight=0.065, parent_symbol=SPY)
├── Row 4: GOOGL exposure (is_constituent_exposure=1, weight=0.04, parent_symbol=SPY)
└── ... (8 constituent rows for SPY)
```

This increases the total row count but enables look-through analytics.

## Deterministic Output

The `--seed` option ensures reproducible output:

```bash
# These produce identical output
cargo run -p pivot-data-gen -- --seed 42 --rows 100 > a.csv
cargo run -p pivot-data-gen -- --seed 42 --rows 100 > b.csv
diff a.csv b.csv  # No differences
```

## Loading into ClickHouse

```bash
# Start ClickHouse
docker compose up -d clickhouse

# Initialize schema
pnpm db:reset

# Generate and load data
cargo run -p pivot-data-gen --release -- \
  --rows 100000 \
  --output /tmp/trades.csv \
  --instruments-output /tmp/instruments.csv \
  --constituents-output /tmp/constituents.csv

# Load via clickhouse-client
docker compose exec -T clickhouse clickhouse-client --query \
  "INSERT INTO pivot.instruments FORMAT CSVWithNames" < /tmp/instruments.csv

docker compose exec -T clickhouse clickhouse-client --query \
  "INSERT INTO pivot.constituents FORMAT CSVWithNames" < /tmp/constituents.csv

docker compose exec -T clickhouse clickhouse-client --query \
  "INSERT INTO pivot.trades_1d FORMAT CSVWithNames" < /tmp/trades.csv
```

## Performance

Approximate generation times on M1 Mac:

| Rows | Time | File Size |
|------|------|-----------|
| 1,000 | <10ms | ~500KB |
| 10,000 | ~50ms | ~5MB |
| 100,000 | ~400ms | ~50MB |
| 1,000,000 | ~4s | ~500MB |

With `--explode-constituents`, expect 2-3x more rows due to ETF/ETC constituent expansion.
