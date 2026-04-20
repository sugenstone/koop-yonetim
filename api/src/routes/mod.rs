pub mod kasa;
pub mod hissedar;
pub mod donem;
pub mod hisse;
pub mod gelir_gider;
pub mod kullanici;

use axum::Router;
use sqlx::PgPool;

pub fn all_routes(pool: PgPool) -> Router {
    Router::new()
        .nest("/auth",         crate::auth::router(pool.clone()))
        .nest("/kullanicilar", kullanici::router(pool.clone()))
        .nest("/kasalar",      kasa::router(pool.clone()))
        .nest("/hissedarlar",  hissedar::router(pool.clone()))
        .nest("/donemler",     donem::router(pool.clone()))
        .nest("/hisseler",     hisse::router(pool.clone()))
        .nest("/gelir-gider",  gelir_gider::router(pool.clone()))
}
