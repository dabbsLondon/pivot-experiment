use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use sha2::{Sha256, Digest};
use crate::config::RedisConfig;

pub type CacheClient = ConnectionManager;

pub async fn create_client(config: &RedisConfig) -> Result<CacheClient, redis::RedisError> {
    let client = redis::Client::open(config.url.clone())?;
    client.get_connection_manager().await
}

pub async fn health_check(client: &mut CacheClient) -> Result<(), redis::RedisError> {
    let _: String = redis::cmd("PING").query_async(client).await?;
    Ok(())
}

pub fn generate_cache_key(prefix: &str, data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let hash = hasher.finalize();
    format!("{}:{:x}", prefix, hash)
}

pub async fn get_cached<T: serde::de::DeserializeOwned>(
    client: &mut CacheClient,
    key: &str,
) -> Result<Option<T>, redis::RedisError> {
    let value: Option<String> = client.get(key).await?;
    match value {
        Some(json) => {
            match serde_json::from_str(&json) {
                Ok(parsed) => Ok(Some(parsed)),
                Err(_) => Ok(None),
            }
        }
        None => Ok(None),
    }
}

pub async fn set_cached<T: serde::Serialize>(
    client: &mut CacheClient,
    key: &str,
    value: &T,
    ttl_seconds: u64,
) -> Result<(), redis::RedisError> {
    let json = serde_json::to_string(value).unwrap_or_default();
    client.set_ex(key, json, ttl_seconds).await
}
