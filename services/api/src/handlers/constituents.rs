use actix_web::{web, HttpResponse};
use std::time::Instant;

use crate::db::models::ConstituentRow;
use crate::error::ApiError;
use crate::models::request::ConstituentsQuery;
use crate::models::response::{Constituent, ConstituentsResponse};
use crate::cache::redis::{get_cached, set_cached};
use crate::AppState;

const CACHE_KEY_ALL: &str = "constituents:all";
const CACHE_TTL: u64 = 3600; // 1 hour

pub async fn handler(
    state: web::Data<AppState>,
    query: web::Query<ConstituentsQuery>,
) -> Result<HttpResponse, ApiError> {
    let start = Instant::now();

    // Check cache first (only for unfiltered requests)
    let use_cache = query.parent_symbol.is_none() && query.constituent_symbol.is_none();

    if use_cache && state.config.cache.enabled {
        let mut redis = state.redis.clone();
        if let Ok(Some(cached)) = get_cached::<ConstituentsResponse>(&mut redis, CACHE_KEY_ALL).await {
            tracing::debug!("Constituents cache hit, returned in {}ms", start.elapsed().as_millis());
            return Ok(HttpResponse::Ok().json(cached));
        }
    }

    // Build query
    let mut sql = "SELECT parent_symbol, constituent_symbol, weight, shares_per_unit, toString(effective_date) as effective_date FROM pivot.constituents".to_string();
    let mut conditions = Vec::new();

    if let Some(ref ps) = query.parent_symbol {
        conditions.push(format!("parent_symbol = '{}'", escape_string(ps)));
    }
    if let Some(ref cs) = query.constituent_symbol {
        conditions.push(format!("constituent_symbol = '{}'", escape_string(cs)));
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" ORDER BY parent_symbol, weight DESC");

    tracing::debug!("Executing constituents query: {}", sql);

    let rows: Vec<ConstituentRow> = state
        .clickhouse
        .query(&sql)
        .fetch_all()
        .await?;

    let constituents: Vec<Constituent> = rows
        .into_iter()
        .map(|r| Constituent {
            parent_symbol: r.parent_symbol,
            constituent_symbol: r.constituent_symbol,
            weight: r.weight,
            shares_per_unit: r.shares_per_unit,
            effective_date: r.effective_date,
        })
        .collect();

    let response = ConstituentsResponse {
        count: constituents.len(),
        constituents,
    };

    // Cache unfiltered response
    if use_cache && state.config.cache.enabled {
        let mut redis = state.redis.clone();
        let _ = set_cached(&mut redis, CACHE_KEY_ALL, &response, CACHE_TTL).await;
    }

    Ok(HttpResponse::Ok().json(response))
}

fn escape_string(s: &str) -> String {
    s.replace('\'', "''")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == ' ')
        .collect()
}
