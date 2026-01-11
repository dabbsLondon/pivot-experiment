use actix_web::{web, HttpResponse};
use clickhouse::Row;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Instant;

use crate::error::ApiError;
use crate::models::request::PivotRequest;
use crate::models::response::{PivotResponse, PivotRow, QueryMetadata};
use crate::query::PivotQueryBuilder;
use crate::cache::redis::{generate_cache_key, get_cached, set_cached};
use crate::AppState;

#[derive(Debug, Row, Deserialize)]
struct DynamicRow {
    #[serde(flatten)]
    values: HashMap<String, serde_json::Value>,
}

pub async fn handler(
    state: web::Data<AppState>,
    body: web::Json<PivotRequest>,
) -> Result<HttpResponse, ApiError> {
    let start = Instant::now();
    let request = body.into_inner();

    // Check cache first (if not bypassed)
    if !request.cache_bypass && state.config.cache.enabled {
        let cache_key = generate_cache_key("pivot:query", &serde_json::to_string(&request)?);
        let mut redis = state.redis.clone();

        if let Ok(Some(cached)) = get_cached::<PivotResponse>(&mut redis, &cache_key).await {
            let mut response = cached;
            response.metadata.cached = true;
            response.metadata.query_time_ms = start.elapsed().as_millis() as u64;
            return Ok(HttpResponse::Ok().json(response));
        }
    }

    // Build and execute query
    let builder = PivotQueryBuilder::from_request(&request)?;
    let sql = builder.build();

    tracing::debug!("Executing pivot query: {}", sql);

    let rows: Vec<DynamicRow> = state
        .clickhouse
        .query(&sql)
        .fetch_all()
        .await?;

    // Transform rows into response format
    let data: Vec<PivotRow> = rows
        .into_iter()
        .map(|row| {
            let mut dimensions = HashMap::new();
            let mut metrics = HashMap::new();

            for (key, value) in row.values {
                // Check if this is a metric (starts with total_ or ends with _count)
                if key.starts_with("total_") || key.ends_with("_count") || key == "avg_price" {
                    if let Some(num) = value.as_f64() {
                        metrics.insert(key, num);
                    } else if let Some(num) = value.as_i64() {
                        metrics.insert(key, num as f64);
                    }
                } else {
                    dimensions.insert(key, value);
                }
            }

            PivotRow { dimensions, metrics }
        })
        .collect();

    let response = PivotResponse {
        metadata: QueryMetadata {
            total_rows: data.len() as u64,
            returned_rows: data.len(),
            query_time_ms: start.elapsed().as_millis() as u64,
            cached: false,
        },
        data,
    };

    // Cache the response
    if !request.cache_bypass && state.config.cache.enabled {
        let cache_key = generate_cache_key("pivot:query", &serde_json::to_string(&request)?);
        let mut redis = state.redis.clone();
        let _ = set_cached(&mut redis, &cache_key, &response, state.config.cache.ttl_seconds).await;
    }

    Ok(HttpResponse::Ok().json(response))
}
