use axum::{
    extract::{Query, Request, State},
    http::{HeaderMap, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Instant;

use crate::auth::{token_dogrula, AuthUser};
use crate::errors::AppResult;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct IslemLog {
    pub id: i64,
    pub tarih: chrono::DateTime<chrono::Utc>,
    pub kullanici_id: Option<i64>,
    pub kullanici_email: Option<String>,
    pub rol: Option<String>,
    pub yontem: String,
    pub yol: String,
    pub durum_kodu: i32,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub sure_ms: Option<i32>,
    pub hata: Option<String>,
}

/// Middleware: yazma isteklerini (POST/PUT/PATCH/DELETE) `islem_loglari` tablosuna kaydeder.
/// Giris denemeleri de (basarili/basarisiz) kaydedilir. GET istekleri loglanmaz.
pub async fn audit_middleware(
    State(pool): State<PgPool>,
    req: Request,
    next: Next,
) -> Response {
    let method = req.method().clone();
    // Sadece mutasyon isteklerini logla
    let should_log = matches!(
        method,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    );

    if !should_log {
        return next.run(req).await;
    }

    let path = req.uri().path().to_string();
    let headers = req.headers().clone();

    // Kullanici bilgisi (JWT'den; yoksa None)
    let (user_id, email, rol) = extract_user(&headers);

    // IP ve UA
    let ip = client_ip(&headers);
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.chars().take(300).collect::<String>());

    let started = Instant::now();
    let response = next.run(req).await;
    let elapsed_ms = started.elapsed().as_millis() as i32;
    let status = response.status().as_u16() as i32;

    // Hatalar icin response body'sini okumuyoruz (stream, performans).
    // Sadece status >= 400 ise jenerik mesaj koyuyoruz.
    let hata = if status >= 400 {
        Some(format!("HTTP {}", status))
    } else {
        None
    };

    // Yazmayi fire-and-forget yap, response'u bloklamasin
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        let res = sqlx::query(
            "INSERT INTO islem_loglari
                (kullanici_id, kullanici_email, rol, yontem, yol, durum_kodu, ip, user_agent, sure_ms, hata)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        )
        .bind(user_id)
        .bind(email)
        .bind(rol)
        .bind(method.as_str())
        .bind(path)
        .bind(status)
        .bind(ip)
        .bind(user_agent)
        .bind(elapsed_ms)
        .bind(hata)
        .execute(&pool_clone)
        .await;
        if let Err(e) = res {
            tracing::warn!("Audit log yazilamadi: {}", e);
        }
    });

    response
}

fn extract_user(headers: &HeaderMap) -> (Option<i64>, Option<String>, Option<String>) {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));
    if let Some(tok) = token {
        if let Ok(c) = token_dogrula(tok) {
            return (Some(c.sub), Some(c.email), Some(c.rol));
        }
    }
    (None, None, None)
}

fn client_ip(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
}

// ─── Viewer API ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LogQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub kullanici_id: Option<i64>,
    pub yontem: Option<String>,
    pub min_durum: Option<i32>,
    pub q: Option<String>,
}

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/", get(list_loglar))
        .route("/ozet", get(ozet))
        .with_state(pool)
}

async fn list_loglar(
    user: AuthUser,
    State(pool): State<PgPool>,
    Query(q): Query<LogQuery>,
) -> AppResult<Json<Vec<IslemLog>>> {
    user.require_izin(&pool, "log.goruntule").await?;

    let limit = q.limit.unwrap_or(100).clamp(1, 500);
    let offset = q.offset.unwrap_or(0).max(0);
    let arama = q.q.as_deref().unwrap_or("").trim();

    let rows = sqlx::query_as::<_, IslemLog>(
        "SELECT id, tarih, kullanici_id, kullanici_email, rol,
                yontem, yol, durum_kodu, ip, user_agent, sure_ms, hata
         FROM islem_loglari
         WHERE ($1::BIGINT IS NULL OR kullanici_id = $1)
           AND ($2::TEXT   IS NULL OR yontem = $2)
           AND ($3::INT    IS NULL OR durum_kodu >= $3)
           AND ($4 = '' OR yol ILIKE '%' || $4 || '%' OR kullanici_email ILIKE '%' || $4 || '%')
         ORDER BY tarih DESC, id DESC
         LIMIT $5 OFFSET $6"
    )
    .bind(q.kullanici_id)
    .bind(q.yontem.as_deref())
    .bind(q.min_durum)
    .bind(arama)
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await?;

    Ok(Json(rows))
}

#[derive(Debug, Serialize)]
struct Ozet {
    toplam: i64,
    son_24_saat: i64,
    hata_24_saat: i64,
}

async fn ozet(user: AuthUser, State(pool): State<PgPool>) -> AppResult<Json<Ozet>> {
    user.require_izin(&pool, "log.goruntule").await?;
    let toplam: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM islem_loglari")
        .fetch_one(&pool)
        .await?;
    let son_24: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM islem_loglari WHERE tarih > NOW() - INTERVAL '24 hours'"
    )
    .fetch_one(&pool)
    .await?;
    let hata_24: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM islem_loglari WHERE tarih > NOW() - INTERVAL '24 hours' AND durum_kodu >= 400"
    )
    .fetch_one(&pool)
    .await?;
    Ok(Json(Ozet { toplam, son_24_saat: son_24, hata_24_saat: hata_24 }))
}

// IntoResponse bypass icin (ornek, simdi gerek yok)
#[allow(dead_code)]
fn _bypass() -> impl IntoResponse {
    StatusCode::OK
}
