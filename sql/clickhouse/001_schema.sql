CREATE DATABASE IF NOT EXISTS pivot;

-- Instrument reference data with constituent relationships
CREATE TABLE IF NOT EXISTS pivot.instruments
(
    symbol String,
    name String,
    asset_class LowCardinality(String),
    instrument_type LowCardinality(String),  -- 'Stock', 'ETF', 'ETC', 'Index', 'Future', 'Option'
    currency LowCardinality(String),
    exchange LowCardinality(String),
    sector LowCardinality(String),
    is_composite UInt8,  -- 1 if this instrument has constituents (ETF, ETC, Index)
    created_at DateTime DEFAULT now()
)
ENGINE = ReplacingMergeTree(created_at)
ORDER BY symbol;

-- Constituent mappings for composite instruments (ETFs, ETCs, Indices)
CREATE TABLE IF NOT EXISTS pivot.constituents
(
    parent_symbol String,      -- The ETF/ETC/Index symbol
    constituent_symbol String, -- The underlying stock/commodity symbol
    weight Float64,            -- Weight in the composite (0.0 to 1.0)
    shares_per_unit Float64,   -- Number of shares per unit of parent
    effective_date Date,
    expiry_date Date DEFAULT '2099-12-31',
    created_at DateTime DEFAULT now()
)
ENGINE = ReplacingMergeTree(created_at)
PARTITION BY toYYYYMM(effective_date)
ORDER BY (parent_symbol, constituent_symbol, effective_date);

-- Main trades table
CREATE TABLE IF NOT EXISTS pivot.trades_1d
(
    trade_date Date,
    ts DateTime64(3),
    portfolio_manager_id UInt32,
    fund_id UInt32,
    portfolio_id UInt64,
    account_id UInt64,
    desk LowCardinality(String),
    book LowCardinality(String),
    strategy LowCardinality(String),
    region LowCardinality(String),
    country LowCardinality(String),
    venue LowCardinality(String),
    asset_class LowCardinality(String),
    product LowCardinality(String),
    instrument_type LowCardinality(String),
    symbol String,
    parent_symbol String DEFAULT '',  -- If this is a constituent exposure, the parent ETF/ETC
    currency LowCardinality(String),
    counterparty String,
    risk_bucket LowCardinality(String),
    scenario LowCardinality(String),
    trade_id UInt64,
    order_id UInt64,
    is_constituent_exposure UInt8 DEFAULT 0,  -- 1 if this row is an exploded constituent
    quantity Float64,
    price Float64,
    notional Float64,
    pnl Float64,
    delta Float64,
    gamma Float64,
    vega Float64,
    theta Float64,
    rho Float64,
    margin Float64,
    fees Float64,
    slippage Float64,
    vol Float64,
    rate Float64,
    exposure Float64,
    weight Float64 DEFAULT 1.0,  -- Constituent weight (1.0 for direct trades)
    metric_1 Float64,
    metric_2 Float64,
    metric_3 Float64,
    metric_4 Float64,
    metric_5 Float64,
    metric_6 Float64,
    metric_7 Float64,
    metric_8 Float64,
    metric_9 Float64,
    metric_10 Float64,
    metric_11 Float64,
    metric_12 Float64,
    metric_13 Float64,
    metric_14 Float64
)
ENGINE = MergeTree
PARTITION BY trade_date
ORDER BY (portfolio_manager_id, fund_id, book, asset_class, symbol, ts);

-- View to get trades with constituent breakdown
CREATE VIEW IF NOT EXISTS pivot.trades_with_constituents AS
SELECT
    t.*,
    i.name AS instrument_name,
    i.sector,
    i.is_composite
FROM pivot.trades_1d t
LEFT JOIN pivot.instruments i ON t.symbol = i.symbol;
