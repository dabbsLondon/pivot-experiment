use actix_web::{web, HttpResponse};
use clickhouse::Row;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Instant;

use crate::error::ApiError;
use crate::models::request::PnlQuery;
use crate::models::response::{PnlResponse, PnlRow, QueryMetadata};
use crate::cache::redis::{generate_cache_key, get_cached, set_cached};
use crate::AppState;

#[derive(Debug, Row, Deserialize)]
struct PnlDbRow {
    #[serde(flatten)]
    values: HashMap<String, serde_json::Value>,
}

// Allowed group_by columns (whitelist)
const ALLOWED_GROUP_BY: &[&str] = &[
    "portfolio_manager_id",
    "fund_id",
    "desk",
    "book",
    "asset_class",
    "symbol",
    "exposure_type",
    "region",
    "country",
];

pub async fn handler(
    state: web::Data<AppState>,
    query: web::Query<PnlQuery>,
) -> Result<HttpResponse, ApiError> {
    let start = Instant::now();

    // Validate all group_by columns
    for col in &query.group_by {
        if !ALLOWED_GROUP_BY.contains(&col.as_str()) {
            return Err(ApiError::QueryValidation(format!(
                "Invalid group_by column: '{}'. Allowed values: {:?}",
                col, ALLOWED_GROUP_BY
            )));
        }
    }

    if query.group_by.is_empty() {
        return Err(ApiError::QueryValidation(
            "At least one group_by column is required".to_string(),
        ));
    }

    // Check cache first
    if !query.cache_bypass && state.config.cache.enabled {
        let cache_key = generate_cache_key("pnl", &serde_json::to_string(&query.0)?);
        let mut redis = state.redis.clone();

        if let Ok(Some(cached)) = get_cached::<PnlResponse>(&mut redis, &cache_key).await {
            let mut response = cached;
            response.metadata.cached = true;
            response.metadata.query_time_ms = start.elapsed().as_millis() as u64;
            return Ok(HttpResponse::Ok().json(response));
        }
    }

    // Build query
    let group_cols = query.group_by.join(", ");
    let sql = format!(
        "SELECT
            {},
            sum(pnl) AS total_pnl,
            sum(notional) AS total_notional,
            count() AS trade_count
         FROM pivot.trades_1d
         WHERE trade_date = '{}'
         GROUP BY {}
         ORDER BY total_pnl DESC
         LIMIT 100",
        group_cols,
        escape_string(&query.trade_date),
        group_cols
    );

    tracing::debug!("Executing P&L query: {}", sql);

    let rows: Vec<PnlDbRow> = state
        .clickhouse
        .query(&sql)
        .fetch_all()
        .await?;

    let data: Vec<PnlRow> = rows
        .into_iter()
        .map(|row| {
            let mut groups = HashMap::new();
            let mut total_pnl = 0.0;
            let mut total_notional = 0.0;
            let mut trade_count = 0u64;

            for (key, value) in row.values {
                match key.as_str() {
                    "total_pnl" => {
                        total_pnl = value.as_f64().unwrap_or(0.0);
                    }
                    "total_notional" => {
                        total_notional = value.as_f64().unwrap_or(0.0);
                    }
                    "trade_count" => {
                        trade_count = value.as_u64().unwrap_or(0);
                    }
                    _ => {
                        groups.insert(key, value);
                    }
                }
            }

            PnlRow {
                groups,
                total_pnl,
                total_notional,
                trade_count,
            }
        })
        .collect();

    let response = PnlResponse {
        metadata: QueryMetadata {
            total_rows: data.len() as u64,
            returned_rows: data.len(),
            query_time_ms: start.elapsed().as_millis() as u64,
            cached: false,
        },
        data,
    };

    // Cache the response
    if !query.cache_bypass && state.config.cache.enabled {
        let cache_key = generate_cache_key("pnl", &serde_json::to_string(&query.0)?);
        let mut redis = state.redis.clone();
        let _ = set_cached(&mut redis, &cache_key, &response, state.config.cache.ttl_seconds).await;
    }

    Ok(HttpResponse::Ok().json(response))
}

fn escape_string(s: &str) -> String {
    s.replace('\'', "''")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}
