use actix_web::{web, HttpResponse};
use serde::Serialize;
use std::time::Instant;

use crate::AppState;
use crate::db::clickhouse::health_check as ch_health;
use crate::cache::redis::health_check as redis_health;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub clickhouse: ServiceHealth,
    pub redis: ServiceHealth,
    pub version: String,
}

#[derive(Serialize)]
pub struct ServiceHealth {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub async fn handler(state: web::Data<AppState>) -> HttpResponse {
    let mut overall_healthy = true;

    // Check ClickHouse
    let ch_start = Instant::now();
    let clickhouse = match ch_health(&state.clickhouse).await {
        Ok(_) => ServiceHealth {
            status: "connected".to_string(),
            latency_ms: Some(ch_start.elapsed().as_millis() as u64),
            error: None,
        },
        Err(e) => {
            overall_healthy = false;
            ServiceHealth {
                status: "error".to_string(),
                latency_ms: None,
                error: Some(e.to_string()),
            }
        }
    };

    // Check Redis
    let redis_start = Instant::now();
    let mut redis_conn = state.redis.clone();
    let redis = match redis_health(&mut redis_conn).await {
        Ok(_) => ServiceHealth {
            status: "connected".to_string(),
            latency_ms: Some(redis_start.elapsed().as_millis() as u64),
            error: None,
        },
        Err(e) => {
            overall_healthy = false;
            ServiceHealth {
                status: "error".to_string(),
                latency_ms: None,
                error: Some(e.to_string()),
            }
        }
    };

    let response = HealthResponse {
        status: if overall_healthy { "healthy" } else { "degraded" }.to_string(),
        clickhouse,
        redis,
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    if overall_healthy {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::ServiceUnavailable().json(response)
    }
}
