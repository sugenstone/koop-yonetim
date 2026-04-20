mod db;
mod models;
mod routes;
mod auth;
mod errors;
mod mail;

use axum::{routing::get, Router, http::Method};
use tower_http::cors::{CorsLayer, Any};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // .env dosyasını yükle
    dotenvy::dotenv().ok();

    // Veritabanı bağlantısı
    let pool = db::create_pool().await?;
    match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        db::run_migrations(&pool),
    ).await {
        Ok(_) => {},
        Err(_) => tracing::warn!("Migration timeout, sunucu başlatılıyor"),
    }

    // CORS ayarları
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any)
        .allow_origin(Any);

    // Router
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .nest("/api", routes::all_routes(pool))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Koop API sunucusu başlatılıyor: http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

