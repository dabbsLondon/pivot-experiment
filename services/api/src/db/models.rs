use clickhouse::Row;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct InstrumentRow {
    pub symbol: String,
    pub name: String,
    pub asset_class: String,
    pub instrument_type: String,
    pub currency: String,
    pub exchange: String,
    pub sector: String,
    #[serde(deserialize_with = "deserialize_bool")]
    pub is_composite: bool,
}

#[derive(Debug, Clone, Row, Serialize, Deserialize)]
pub struct ConstituentRow {
    pub parent_symbol: String,
    pub constituent_symbol: String,
    pub weight: f64,
    pub shares_per_unit: f64,
    pub effective_date: String,
}

fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: u8 = Deserialize::deserialize(deserializer)?;
    Ok(value != 0)
}
