pub mod kasa;
pub mod hissedar;
pub mod donem;
pub mod hisse;
pub mod gelir_gider;
pub mod kullanici;
pub mod izin;

use axum::{middleware, Router, extract::Request, response::Response, middleware::Next};
use sqlx::PgPool;

use crate::auth::AuthUser;
use crate::errors::AppError;

/// Tum korumali endpoint'ler icin JWT kontrolu.
/// AuthUser extractor'unu zorla cagirir; basarisizsa 401 doner.
async fn require_auth(req: Request, next: Next) -> Result<Response, AppError> {
    let (mut parts, body) = req.into_parts();
    // AuthUser extractor state gerektirmiyor (JWT_SECRET env'den).
    let _user = <AuthUser as axum::extract::FromRequestParts<()>>::from_request_parts(&mut parts, &())
        .await?;
    let req = Request::from_parts(parts, body);
    Ok(next.run(req).await)
}

pub fn all_routes(pool: PgPool) -> Router {
    // Korumali route'lar (JWT zorunlu)
    let korumali = Router::new()
        .nest("/kullanicilar", kullanici::router(pool.clone()))
        .nest("/kasalar",      kasa::router(pool.clone()))
        .nest("/hissedarlar",  hissedar::router(pool.clone()))
        .nest("/donemler",     donem::router(pool.clone()))
        .nest("/hisseler",     hisse::router(pool.clone()))
        .nest("/gelir-gider",  gelir_gider::router(pool.clone()))
        .nest("/izinler",      izin::router(pool.clone()))
        .layer(middleware::from_fn(require_auth));

    // Public: sadece giris
    Router::new()
        .nest("/auth", crate::auth::router(pool.clone()))
        .merge(korumali)
}
