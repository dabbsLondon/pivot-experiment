use actix_web::{web, HttpResponse};
use clickhouse::Row;
use serde::Deserialize;
use std::time::Instant;

use crate::error::ApiError;
use crate::models::request::{ExposureQuery, ExposureView};
use crate::models::response::{ExposureResponse, ExposureRow, QueryMetadata};
use crate::cache::redis::{generate_cache_key, get_cached, set_cached};
use crate::AppState;

#[derive(Debug, Row, Deserialize)]
struct ExposureDbRow {
    group_value: String,
    total_notional: f64,
    total_pnl: f64,
    trade_count: u64,
}

// Allowed group_by columns (whitelist)
const ALLOWED_GROUP_BY: &[&str] = &[
    "asset_class",
    "symbol",
    "underlying_symbol",
    "instrument_type",
    "portfolio_manager_id",
    "fund_id",
    "desk",
    "book",
    "region",
    "country",
];

pub async fn handler(
    state: web::Data<AppState>,
    query: web::Query<ExposureQuery>,
) -> Result<HttpResponse, ApiError> {
    let start = Instant::now();

    // Validate group_by column
    let group_by = &query.group_by;
    if !ALLOWED_GROUP_BY.contains(&group_by.as_str()) {
        return Err(ApiError::QueryValidation(format!(
            "Invalid group_by: '{}'. Allowed values: {:?}",
            group_by, ALLOWED_GROUP_BY
        )));
    }

    // Check cache first
    if !query.cache_bypass && state.config.cache.enabled {
        let cache_key = generate_cache_key("exposure", &serde_json::to_string(&query.0)?);
        let mut redis = state.redis.clone();

        if let Ok(Some(cached)) = get_cached::<ExposureResponse>(&mut redis, &cache_key).await {
            let mut response = cached;
            response.metadata.cached = true;
            response.metadata.query_time_ms = start.elapsed().as_millis() as u64;
            return Ok(HttpResponse::Ok().json(response));
        }
    }

    // Build exposure type filter based on view
    let exposure_filter = match query.view {
        ExposureView::TopLevel => "exposure_type IN ('Direct', 'ETF', 'ETC')",
        ExposureView::LookThrough => "exposure_type IN ('Direct', 'Constituent')",
        ExposureView::All => "1=1",
    };

    let sql = format!(
        "SELECT
            toString({}) AS group_value,
            sum(notional) AS total_notional,
            sum(pnl) AS total_pnl,
            count() AS trade_count
         FROM pivot.trades_1d
         WHERE trade_date = '{}' AND {}
         GROUP BY {}
         ORDER BY total_notional DESC
         LIMIT 100",
        group_by,
        escape_string(&query.trade_date),
        exposure_filter,
        group_by
    );

    tracing::debug!("Executing exposure query: {}", sql);

    let rows: Vec<ExposureDbRow> = state
        .clickhouse
        .query(&sql)
        .fetch_all()
        .await?;

    let data: Vec<ExposureRow> = rows
        .into_iter()
        .map(|r| ExposureRow {
            group: r.group_value,
            total_notional: r.total_notional,
            total_pnl: r.total_pnl,
            trade_count: r.trade_count,
        })
        .collect();

    let response = ExposureResponse {
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
        let cache_key = generate_cache_key("exposure", &serde_json::to_string(&query.0)?);
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
