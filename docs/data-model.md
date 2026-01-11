# Data Model

This document describes the data model used by the Pivot Experiment platform for financial trade analytics.

## Overview

The system stores trade data in ClickHouse, optimized for high-performance analytical queries across large datasets. The data model supports:

- Trade-level granularity with full attribution
- Composite instruments (ETFs, ETCs) with constituent breakdown
- Multi-dimensional pivoting across organizational and instrument hierarchies
- Pre-aggregated rollups for common query patterns

## Tables

### `pivot.instruments`

Reference data for all tradeable instruments.

| Column | Type | Description |
|--------|------|-------------|
| `symbol` | String | Primary identifier (e.g., "AAPL", "SPY") |
| `name` | String | Full instrument name |
| `asset_class` | LowCardinality(String) | Asset classification (Equity, Commodity, etc.) |
| `instrument_type` | LowCardinality(String) | Type: Stock, ETF, ETC, Index, Future, Option, Commodity |
| `currency` | LowCardinality(String) | Trading currency |
| `exchange` | LowCardinality(String) | Primary exchange |
| `sector` | LowCardinality(String) | Industry sector |
| `is_composite` | UInt8 | 1 if instrument has constituents (ETF/ETC/Index) |

### `pivot.constituents`

Maps composite instruments to their underlying holdings.

| Column | Type | Description |
|--------|------|-------------|
| `parent_symbol` | String | The ETF/ETC/Index symbol |
| `constituent_symbol` | String | Underlying instrument symbol |
| `weight` | Float64 | Portfolio weight (0.0 to 1.0) |
| `shares_per_unit` | Float64 | Shares of constituent per unit of parent |
| `effective_date` | Date | When this mapping became effective |
| `expiry_date` | Date | When this mapping expires (default: 2099-12-31) |

#### Example: ETF Constituents

```
SPY (S&P 500 ETF)
├── AAPL (7.0% weight)
├── MSFT (6.5% weight)
├── GOOGL (4.0% weight)
├── AMZN (3.5% weight)
└── ... (500+ constituents)

GLD (Gold ETC)
└── GOLD (100% weight, backed by physical gold)

PMET (Precious Metals Basket)
├── GOLD (50% weight)
├── SILVER (30% weight)
└── PLAT (20% weight)
```

### `pivot.trades_1d`

Main fact table containing trade-level data.

#### Identification & Timing

| Column | Type | Description |
|--------|------|-------------|
| `trade_date` | Date | Trade date (partition key) |
| `ts` | DateTime64(3) | Timestamp with millisecond precision |
| `trade_id` | UInt64 | Unique trade identifier |
| `order_id` | UInt64 | Parent order identifier |

#### Organizational Hierarchy

| Column | Type | Description |
|--------|------|-------------|
| `portfolio_manager_id` | UInt32 | Portfolio manager identifier |
| `fund_id` | UInt32 | Fund identifier |
| `portfolio_id` | UInt64 | Portfolio identifier |
| `account_id` | UInt64 | Account identifier |
| `desk` | LowCardinality(String) | Trading desk |
| `book` | LowCardinality(String) | Trading book |
| `strategy` | LowCardinality(String) | Trading strategy |

#### Geography

| Column | Type | Description |
|--------|------|-------------|
| `region` | LowCardinality(String) | AMER, EMEA, APAC |
| `country` | LowCardinality(String) | Country code |
| `venue` | LowCardinality(String) | Execution venue |

#### Instrument Details

| Column | Type | Description |
|--------|------|-------------|
| `asset_class` | LowCardinality(String) | Equity, Commodity, etc. |
| `product` | LowCardinality(String) | Product type |
| `instrument_type` | LowCardinality(String) | Stock, ETF, ETC, Constituent |
| `symbol` | String | Instrument symbol |
| `parent_symbol` | String | Parent ETF/ETC if constituent exposure |
| `currency` | LowCardinality(String) | Trading currency |
| `counterparty` | String | Counterparty name |

#### Risk Classification

| Column | Type | Description |
|--------|------|-------------|
| `risk_bucket` | LowCardinality(String) | Low, Medium, High, VeryHigh |
| `scenario` | LowCardinality(String) | Base, Stress, Historical, MonteCarlo |

#### Exposure Type (Key for Pivoting)

| Column | Type | Description |
|--------|------|-------------|
| `underlying_symbol` | String | The actual underlying instrument (AAPL, GOLD, etc.) |
| `parent_symbol` | String | For constituents: the parent ETF/ETC symbol |
| `exposure_type` | LowCardinality(String) | **Key field for pivot filtering** |
| `weight` | Float64 | Constituent weight (1.0 for direct trades) |

**exposure_type values:**

| Value | Description | Use Case |
|-------|-------------|----------|
| `Direct` | Direct stock/commodity trade | Always include |
| `ETF` | Trade in an ETF wrapper (SPY, QQQ) | Include for top-level view |
| `ETC` | Trade in an ETC wrapper (GLD, SLV) | Include for top-level view |
| `Constituent` | Exploded constituent exposure | Include for look-through view |

#### Quantities & Values

| Column | Type | Description |
|--------|------|-------------|
| `quantity` | Float64 | Trade quantity |
| `price` | Float64 | Execution price |
| `notional` | Float64 | Notional value (quantity × price) |
| `pnl` | Float64 | Profit/Loss |

#### Greeks (Options Risk)

| Column | Type | Description |
|--------|------|-------------|
| `delta` | Float64 | Price sensitivity (-1.0 to 1.0) |
| `gamma` | Float64 | Delta sensitivity (0.0 to 0.1) |
| `vega` | Float64 | Volatility sensitivity |
| `theta` | Float64 | Time decay (always ≤ 0) |
| `rho` | Float64 | Interest rate sensitivity |

#### Additional Metrics

| Column | Type | Description |
|--------|------|-------------|
| `margin` | Float64 | Required margin |
| `fees` | Float64 | Transaction fees |
| `slippage` | Float64 | Execution slippage |
| `vol` | Float64 | Implied volatility |
| `rate` | Float64 | Interest rate |
| `exposure` | Float64 | Risk exposure |
| `metric_1` to `metric_14` | Float64 | Custom metrics |

## Constituent Explosion

When the `--explode-constituents` flag is enabled, trades in composite instruments (ETFs, ETCs) generate additional rows for each constituent holding.

### Example

A trade of 1000 shares of SPY generates:

1. **Parent row**: The original SPY trade
   - `symbol`: SPY
   - `is_constituent_exposure`: 0
   - `weight`: 1.0
   - `quantity`: 1000

2. **Constituent rows**: One per holding
   - `symbol`: AAPL (constituent)
   - `parent_symbol`: SPY
   - `is_constituent_exposure`: 1
   - `weight`: 0.07
   - `quantity`: 450 (1000 × 0.45 shares_per_unit)
   - `notional`: 7% of parent notional

This enables queries like:
- "What is my total AAPL exposure across direct holdings AND ETF holdings?"
- "How much of my portfolio is exposed to Technology sector through ETFs?"

## Rollup Tables

### `pivot.trades_1d_rollup`

Pre-aggregated daily summaries for fast dashboard queries.

```sql
SELECT
    trade_date,
    portfolio_manager_id,
    fund_id,
    book,
    asset_class,
    symbol,
    sumMerge(quantity_state) AS total_quantity,
    sumMerge(notional_state) AS total_notional,
    sumMerge(pnl_state) AS total_pnl
FROM pivot.trades_1d_rollup
GROUP BY trade_date, portfolio_manager_id, fund_id, book, asset_class, symbol
```

## Query Patterns

### Top-Level View (No Double Counting)

Use this when you want to see ETFs as single positions without exploding to constituents:

```sql
SELECT
    symbol,
    exposure_type,
    sum(notional) AS total_notional,
    sum(pnl) AS total_pnl
FROM pivot.trades_1d
WHERE trade_date = '2024-01-15'
  AND exposure_type IN ('Direct', 'ETF', 'ETC')  -- Exclude constituent rows
GROUP BY symbol, exposure_type
ORDER BY total_notional DESC
```

### Look-Through View (Actual Underlying Exposure)

Use this to see what you actually own through ETFs:

```sql
SELECT
    underlying_symbol,
    exposure_type,
    sum(notional) AS total_notional,
    sum(pnl) AS total_pnl
FROM pivot.trades_1d
WHERE trade_date = '2024-01-15'
  AND exposure_type IN ('Direct', 'Constituent')  -- Direct + ETF constituents
GROUP BY underlying_symbol, exposure_type
ORDER BY total_notional DESC
```

### Total AAPL Exposure (Direct + Via ETFs)

```sql
SELECT
    underlying_symbol,
    sumIf(notional, exposure_type = 'Direct') AS direct_exposure,
    sumIf(notional, exposure_type = 'Constituent') AS etf_exposure,
    sum(notional) AS total_exposure
FROM pivot.trades_1d
WHERE underlying_symbol = 'AAPL'
  AND trade_date = '2024-01-15'
GROUP BY underlying_symbol
```

### ETF Decomposition (What's Inside Each ETF)

```sql
SELECT
    parent_symbol AS etf,
    underlying_symbol,
    weight,
    sum(notional) AS constituent_notional
FROM pivot.trades_1d
WHERE exposure_type = 'Constituent'
  AND trade_date = '2024-01-15'
GROUP BY parent_symbol, underlying_symbol, weight
ORDER BY parent_symbol, weight DESC
```

### Portfolio Manager P&L by Exposure Type

```sql
SELECT
    portfolio_manager_id,
    exposure_type,
    sum(pnl) AS total_pnl,
    count() AS trade_count
FROM pivot.trades_1d
WHERE trade_date = '2024-01-15'
GROUP BY portfolio_manager_id, exposure_type
ORDER BY portfolio_manager_id, total_pnl DESC
```
