CREATE TABLE IF NOT EXISTS pivot.trades_1d_rollup
(
    trade_date Date,
    portfolio_manager_id UInt32,
    fund_id UInt32,
    book LowCardinality(String),
    asset_class LowCardinality(String),
    symbol String,
    quantity_state AggregateFunction(sum, Float64),
    notional_state AggregateFunction(sum, Float64),
    pnl_state AggregateFunction(sum, Float64)
)
ENGINE = AggregatingMergeTree
PARTITION BY trade_date
ORDER BY (portfolio_manager_id, fund_id, book, asset_class, symbol);

CREATE MATERIALIZED VIEW IF NOT EXISTS pivot.trades_1d_rollup_mv
TO pivot.trades_1d_rollup
AS
SELECT
    trade_date,
    portfolio_manager_id,
    fund_id,
    book,
    asset_class,
    symbol,
    sumState(quantity) AS quantity_state,
    sumState(notional) AS notional_state,
    sumState(pnl) AS pnl_state
FROM pivot.trades_1d
GROUP BY
    trade_date,
    portfolio_manager_id,
    fund_id,
    book,
    asset_class,
    symbol;
