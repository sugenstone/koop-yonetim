use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;

pub async fn create_pool() -> anyhow::Result<PgPool> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL ortam değişkeni ayarlanmamış");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    // Migration'lar VDS'de zaten uygulandı, advisory lock sorununu önlemek için
    // sadece migrate!() yerine doğrudan çalıştır ama hataları yoksay
    match sqlx::migrate!("./migrations").run(pool).await {
        Ok(_) => tracing::info!("Migration'lar başarıyla çalıştırıldı"),
        Err(e) => tracing::warn!("Migration atlandı: {}", e),
    }
    Ok(())
}
