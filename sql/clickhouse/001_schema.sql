CREATE DATABASE IF NOT EXISTS pivot;

-- Instrument reference data with constituent relationships
CREATE TABLE IF NOT EXISTS pivot.instruments
(
    symbol String,
    name String,
    asset_class LowCardinality(String),
    instrument_type LowCardinality(String),  -- 'Stock', 'ETF', 'ETC', 'Index', 'Future', 'Option', 'Commodity'
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
    -- Identification & Timing
    trade_date Date,
    ts DateTime64(3),
    trade_id UInt64,
    order_id UInt64,

    -- Organizational Hierarchy
    portfolio_manager_id UInt32,
    fund_id UInt32,
    portfolio_id UInt64,
    account_id UInt64,
    desk LowCardinality(String),
    book LowCardinality(String),
    strategy LowCardinality(String),

    -- Geography
    region LowCardinality(String),
    country LowCardinality(String),
    venue LowCardinality(String),

    -- Instrument Details
    asset_class LowCardinality(String),
    product LowCardinality(String),
    instrument_type LowCardinality(String),
    symbol String,
    underlying_symbol String,              -- The actual underlying (AAPL, GOLD, etc.)
    parent_symbol String DEFAULT '',       -- For constituents: the parent ETF/ETC
    exposure_type LowCardinality(String),  -- 'Direct', 'ETF', 'ETC', 'Constituent'
    currency LowCardinality(String),
    counterparty String,

    -- Risk Classification
    risk_bucket LowCardinality(String),
    scenario LowCardinality(String),

    -- Quantities & Values
    quantity Float64,
    price Float64,
    notional Float64,
    pnl Float64,

    -- Greeks (Options Risk)
    delta Float64,
    gamma Float64,
    vega Float64,
    theta Float64,
    rho Float64,

    -- Additional Metrics
    margin Float64,
    fees Float64,
    slippage Float64,
    vol Float64,
    rate Float64,
    exposure Float64,
    weight Float64 DEFAULT 1.0,  -- Constituent weight (1.0 for direct trades)

    -- Custom Metrics
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

-- ============================================================================
-- PIVOT QUERY EXAMPLES
-- ============================================================================

-- Example 1: Top-level view (no double counting)
-- Use this to see ETF trades as single line items
-- SELECT * FROM pivot.trades_1d WHERE exposure_type IN ('Direct', 'ETF', 'ETC')

-- Example 2: Look-through view (see actual underlying exposure)
-- Use this to see what you actually own through ETFs
-- SELECT * FROM pivot.trades_1d WHERE exposure_type IN ('Direct', 'Constituent')

-- Example 3: Aggregate by underlying across all exposure types
-- SELECT
--     underlying_symbol,
--     exposure_type,
--     sum(notional) AS total_notional,
--     sum(pnl) AS total_pnl
-- FROM pivot.trades_1d
-- GROUP BY underlying_symbol, exposure_type
-- ORDER BY total_notional DESC

-- Example 4: Total AAPL exposure (direct + via ETFs)
-- SELECT
--     underlying_symbol,
--     sumIf(notional, exposure_type = 'Direct') AS direct_exposure,
--     sumIf(notional, exposure_type = 'Constituent') AS etf_exposure,
--     sum(notional) AS total_exposure
-- FROM pivot.trades_1d
-- WHERE underlying_symbol = 'AAPL'
-- GROUP BY underlying_symbol
