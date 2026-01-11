use clickhouse::Client;
use crate::config::ClickHouseConfig;

pub fn create_client(config: &ClickHouseConfig) -> Client {
    Client::default()
        .with_url(&config.url)
        .with_database(&config.database)
}

pub async fn health_check(client: &Client) -> Result<(), clickhouse::error::Error> {
    client.query("SELECT 1").execute().await?;
    Ok(())
}
