use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct PivotResponse {
    pub data: Vec<PivotRow>,
    pub metadata: QueryMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PivotRow {
    pub dimensions: HashMap<String, serde_json::Value>,
    pub metrics: HashMap<String, f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryMetadata {
    pub total_rows: u64,
    pub returned_rows: usize,
    pub query_time_ms: u64,
    pub cached: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstrumentsResponse {
    pub instruments: Vec<Instrument>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub symbol: String,
    pub name: String,
    pub asset_class: String,
    pub instrument_type: String,
    pub currency: String,
    pub exchange: String,
    pub sector: String,
    pub is_composite: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConstituentsResponse {
    pub constituents: Vec<Constituent>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constituent {
    pub parent_symbol: String,
    pub constituent_symbol: String,
    pub weight: f64,
    pub shares_per_unit: f64,
    pub effective_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExposureResponse {
    pub data: Vec<ExposureRow>,
    pub metadata: QueryMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExposureRow {
    pub group: String,
    pub total_notional: f64,
    pub total_pnl: f64,
    pub trade_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PnlResponse {
    pub data: Vec<PnlRow>,
    pub metadata: QueryMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PnlRow {
    pub groups: HashMap<String, serde_json::Value>,
    pub total_pnl: f64,
    pub total_notional: f64,
    pub trade_count: u64,
}
