use actix_web::{web, HttpResponse};
use std::time::Instant;

use crate::db::models::InstrumentRow;
use crate::error::ApiError;
use crate::models::request::InstrumentsQuery;
use crate::models::response::{Instrument, InstrumentsResponse};
use crate::cache::redis::{get_cached, set_cached};
use crate::AppState;

const CACHE_KEY: &str = "instruments:all";
const CACHE_TTL: u64 = 3600; // 1 hour

pub async fn handler(
    state: web::Data<AppState>,
    query: web::Query<InstrumentsQuery>,
) -> Result<HttpResponse, ApiError> {
    let start = Instant::now();

    // Check cache first (only for unfiltered requests)
    let use_cache = query.asset_class.is_none() && query.instrument_type.is_none();

    if use_cache && state.config.cache.enabled {
        let mut redis = state.redis.clone();
        if let Ok(Some(cached)) = get_cached::<InstrumentsResponse>(&mut redis, CACHE_KEY).await {
            tracing::debug!("Instruments cache hit, returned in {}ms", start.elapsed().as_millis());
            return Ok(HttpResponse::Ok().json(cached));
        }
    }

    // Build query
    let mut sql = "SELECT symbol, name, asset_class, instrument_type, currency, exchange, sector, is_composite FROM pivot.instruments".to_string();
    let mut conditions = Vec::new();

    if let Some(ref ac) = query.asset_class {
        conditions.push(format!("asset_class = '{}'", escape_string(ac)));
    }
    if let Some(ref it) = query.instrument_type {
        conditions.push(format!("instrument_type = '{}'", escape_string(it)));
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" ORDER BY symbol");

    tracing::debug!("Executing instruments query: {}", sql);

    let rows: Vec<InstrumentRow> = state
        .clickhouse
        .query(&sql)
        .fetch_all()
        .await?;

    let instruments: Vec<Instrument> = rows
        .into_iter()
        .map(|r| Instrument {
            symbol: r.symbol,
            name: r.name,
            asset_class: r.asset_class,
            instrument_type: r.instrument_type,
            currency: r.currency,
            exchange: r.exchange,
            sector: r.sector,
            is_composite: r.is_composite,
        })
        .collect();

    let response = InstrumentsResponse {
        count: instruments.len(),
        instruments,
    };

    // Cache unfiltered response
    if use_cache && state.config.cache.enabled {
        let mut redis = state.redis.clone();
        let _ = set_cached(&mut redis, CACHE_KEY, &response, CACHE_TTL).await;
    }

    Ok(HttpResponse::Ok().json(response))
}

fn escape_string(s: &str) -> String {
    s.replace('\'', "''")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == ' ')
        .collect()
}
