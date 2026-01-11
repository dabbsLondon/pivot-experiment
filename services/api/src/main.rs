use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use tracing_subscriber::EnvFilter;

use pivot_api::cache;
use pivot_api::config::Config;
use pivot_api::db;
use pivot_api::handlers;
use pivot_api::middleware::create_logger;
use pivot_api::AppState;

fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(handlers::health::handler))
        .service(
            web::scope("/api/v1")
                .route("/pivot", web::post().to(handlers::pivot::handler))
                .route("/instruments", web::get().to(handlers::instruments::handler))
                .route("/constituents", web::get().to(handlers::constituents::handler))
                .route("/exposure", web::get().to(handlers::exposure::handler))
                .route("/pnl", web::get().to(handlers::pnl::handler)),
        );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env file if present
    let _ = dotenvy::dotenv();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("pivot_api=info,actix_web=info")),
        )
        .init();

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");

    tracing::info!(
        "Starting Pivot API server on {}:{}",
        config.server.host,
        config.server.port
    );
    tracing::info!("ClickHouse: {}", config.clickhouse.url);
    tracing::info!("Redis: {}", config.redis.url);
    tracing::info!("Cache enabled: {}", config.cache.enabled);

    // Create ClickHouse client
    let clickhouse = db::create_client(&config.clickhouse);

    // Create Redis connection manager
    let redis = cache::create_client(&config.redis)
        .await
        .expect("Failed to connect to Redis");

    // Create shared application state
    let state = web::Data::new(AppState {
        clickhouse,
        redis,
        config: config.clone(),
    });

    let host = config.server.host.clone();
    let port = config.server.port;

    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(state.clone())
            .wrap(create_logger())
            .wrap(cors)
            .configure(configure_routes)
    })
    .bind((host, port))?
    .run()
    .await
}
