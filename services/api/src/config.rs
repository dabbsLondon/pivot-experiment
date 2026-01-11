use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub clickhouse: ClickHouseConfig,
    pub redis: RedisConfig,
    pub cache: CacheConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct ClickHouseConfig {
    pub url: String,
    pub database: String,
}

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl_seconds: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Config {
            server: ServerConfig {
                host: env::var("PIVOT_API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PIVOT_API_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .unwrap_or(8080),
            },
            clickhouse: ClickHouseConfig {
                url: env::var("CLICKHOUSE_URL")
                    .unwrap_or_else(|_| "http://localhost:8123".to_string()),
                database: env::var("CLICKHOUSE_DATABASE")
                    .unwrap_or_else(|_| "pivot".to_string()),
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            },
            cache: CacheConfig {
                enabled: env::var("CACHE_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                ttl_seconds: env::var("CACHE_TTL_SECONDS")
                    .unwrap_or_else(|_| "300".to_string())
                    .parse()
                    .unwrap_or(300),
            },
        })
    }
}
