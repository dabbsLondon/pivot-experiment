use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Dimension {
    TradeDate,
    PortfolioManagerId,
    FundId,
    PortfolioId,
    AccountId,
    Desk,
    Book,
    Strategy,
    Region,
    Country,
    Venue,
    AssetClass,
    Product,
    InstrumentType,
    Symbol,
    UnderlyingSymbol,
    ParentSymbol,
    ExposureType,
    Currency,
    Counterparty,
    RiskBucket,
    Scenario,
}

impl Dimension {
    pub fn to_column(&self) -> &'static str {
        match self {
            Dimension::TradeDate => "trade_date",
            Dimension::PortfolioManagerId => "portfolio_manager_id",
            Dimension::FundId => "fund_id",
            Dimension::PortfolioId => "portfolio_id",
            Dimension::AccountId => "account_id",
            Dimension::Desk => "desk",
            Dimension::Book => "book",
            Dimension::Strategy => "strategy",
            Dimension::Region => "region",
            Dimension::Country => "country",
            Dimension::Venue => "venue",
            Dimension::AssetClass => "asset_class",
            Dimension::Product => "product",
            Dimension::InstrumentType => "instrument_type",
            Dimension::Symbol => "symbol",
            Dimension::UnderlyingSymbol => "underlying_symbol",
            Dimension::ParentSymbol => "parent_symbol",
            Dimension::ExposureType => "exposure_type",
            Dimension::Currency => "currency",
            Dimension::Counterparty => "counterparty",
            Dimension::RiskBucket => "risk_bucket",
            Dimension::Scenario => "scenario",
        }
    }

    pub fn all() -> &'static [Dimension] {
        &[
            Dimension::TradeDate,
            Dimension::PortfolioManagerId,
            Dimension::FundId,
            Dimension::PortfolioId,
            Dimension::AccountId,
            Dimension::Desk,
            Dimension::Book,
            Dimension::Strategy,
            Dimension::Region,
            Dimension::Country,
            Dimension::Venue,
            Dimension::AssetClass,
            Dimension::Product,
            Dimension::InstrumentType,
            Dimension::Symbol,
            Dimension::UnderlyingSymbol,
            Dimension::ParentSymbol,
            Dimension::ExposureType,
            Dimension::Currency,
            Dimension::Counterparty,
            Dimension::RiskBucket,
            Dimension::Scenario,
        ]
    }
}
