use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Metric {
    Quantity,
    Notional,
    Pnl,
    Price,
    Delta,
    Gamma,
    Vega,
    Theta,
    Rho,
    Margin,
    Fees,
    Slippage,
    Exposure,
    TradeCount,
}

impl Metric {
    pub fn to_aggregation(&self) -> &'static str {
        match self {
            Metric::Quantity => "sum(quantity)",
            Metric::Notional => "sum(notional)",
            Metric::Pnl => "sum(pnl)",
            Metric::Price => "avg(price)",
            Metric::Delta => "sum(delta)",
            Metric::Gamma => "sum(gamma)",
            Metric::Vega => "sum(vega)",
            Metric::Theta => "sum(theta)",
            Metric::Rho => "sum(rho)",
            Metric::Margin => "sum(margin)",
            Metric::Fees => "sum(fees)",
            Metric::Slippage => "sum(slippage)",
            Metric::Exposure => "sum(exposure)",
            Metric::TradeCount => "count()",
        }
    }

    pub fn alias(&self) -> &'static str {
        match self {
            Metric::Quantity => "total_quantity",
            Metric::Notional => "total_notional",
            Metric::Pnl => "total_pnl",
            Metric::Price => "avg_price",
            Metric::Delta => "total_delta",
            Metric::Gamma => "total_gamma",
            Metric::Vega => "total_vega",
            Metric::Theta => "total_theta",
            Metric::Rho => "total_rho",
            Metric::Margin => "total_margin",
            Metric::Fees => "total_fees",
            Metric::Slippage => "total_slippage",
            Metric::Exposure => "total_exposure",
            Metric::TradeCount => "trade_count",
        }
    }

    pub fn all() -> &'static [Metric] {
        &[
            Metric::Quantity,
            Metric::Notional,
            Metric::Pnl,
            Metric::Price,
            Metric::Delta,
            Metric::Gamma,
            Metric::Vega,
            Metric::Theta,
            Metric::Rho,
            Metric::Margin,
            Metric::Fees,
            Metric::Slippage,
            Metric::Exposure,
            Metric::TradeCount,
        ]
    }
}
