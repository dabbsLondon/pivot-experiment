pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod query;

use cache::CacheClient;
use clickhouse::Client;
use config::Config;

pub struct AppState {
    pub clickhouse: Client,
    pub redis: CacheClient,
    pub config: Config,
}
