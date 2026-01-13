use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Database(String),
    Cache(String),
    QueryValidation(String),
    Internal(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            ApiError::Database(msg) => write!(f, "Database error: {}", msg),
            ApiError::Cache(msg) => write!(f, "Cache error: {}", msg),
            ApiError::QueryValidation(msg) => write!(f, "Query validation error: {}", msg),
            ApiError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::QueryValidation(_) => StatusCode::BAD_REQUEST,
            ApiError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Cache(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let message = match self {
            ApiError::BadRequest(msg) => msg.clone(),
            ApiError::QueryValidation(msg) => msg.clone(),
            ApiError::Database(_) => "Database error".to_string(),
            ApiError::Cache(_) => "Cache error".to_string(),
            ApiError::Internal(_) => "Internal server error".to_string(),
        };

        HttpResponse::build(self.status_code())
            .json(ErrorResponse { error: message })
    }
}

impl From<clickhouse::error::Error> for ApiError {
    fn from(err: clickhouse::error::Error) -> Self {
        tracing::error!("ClickHouse error: {:?}", err);
        ApiError::Database(err.to_string())
    }
}

impl From<redis::RedisError> for ApiError {
    fn from(err: redis::RedisError) -> Self {
        tracing::error!("Redis error: {:?}", err);
        ApiError::Cache(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::BadRequest(format!("JSON error: {}", err))
    }
}
