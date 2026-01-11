use serde::{Deserialize, Serialize};
use crate::query::{Dimension, Metric};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivotRequest {
    pub dimensions: Vec<Dimension>,
    pub metrics: Vec<Metric>,
    #[serde(default)]
    pub filters: PivotFilters,
    #[serde(default)]
    pub sort: Option<SortSpec>,
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
    #[serde(default)]
    pub cache_bypass: bool,
}

impl Default for PivotRequest {
    fn default() -> Self {
        Self {
            dimensions: vec![],
            metrics: vec![],
            filters: PivotFilters::default(),
            sort: None,
            limit: default_limit(),
            offset: 0,
            cache_bypass: false,
        }
    }
}

fn default_limit() -> u32 {
    100
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PivotFilters {
    pub trade_date: Option<String>,
    pub trade_date_range: Option<DateRange>,
    pub exposure_type: Option<Vec<ExposureType>>,
    pub portfolio_manager_id: Option<Vec<u32>>,
    pub fund_id: Option<Vec<u32>>,
    pub asset_class: Option<Vec<String>>,
    pub symbol: Option<Vec<String>>,
    pub underlying_symbol: Option<Vec<String>>,
    pub parent_symbol: Option<Vec<String>>,
    pub desk: Option<Vec<String>>,
    pub book: Option<Vec<String>>,
    pub region: Option<Vec<String>>,
    pub country: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortSpec {
    pub field: String,
    #[serde(default)]
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    #[default]
    Desc,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExposureType {
    Direct,
    #[serde(rename = "ETF")]
    Etf,
    #[serde(rename = "ETC")]
    Etc,
    Constituent,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstrumentsQuery {
    pub asset_class: Option<String>,
    pub instrument_type: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConstituentsQuery {
    pub parent_symbol: Option<String>,
    pub constituent_symbol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureQuery {
    pub trade_date: String,
    #[serde(default = "default_group_by")]
    pub group_by: String,
    #[serde(default)]
    pub view: ExposureView,
    #[serde(default)]
    pub cache_bypass: bool,
}

fn default_group_by() -> String {
    "asset_class".to_string()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExposureView {
    #[default]
    TopLevel,
    LookThrough,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnlQuery {
    pub trade_date: String,
    #[serde(default = "default_pnl_group_by")]
    pub group_by: String,
    #[serde(default)]
    pub cache_bypass: bool,
}

fn default_pnl_group_by() -> String {
    "portfolio_manager_id".to_string()
}
