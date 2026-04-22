use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::AuthUser;
use crate::errors::{AppError, AppResult};

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/yedekle", post(yedekle))
        .route("/sifirla", post(sifirla))
        .with_state(pool)
}

#[derive(Debug, Deserialize)]
struct SifirlaInput {
    sifre: String,
}

#[derive(Debug, Serialize)]
struct MesajResponse {
    mesaj: String,
}

// ─── Yedekleme ────────────────────────────────────────────────────────────────

async fn yedekle(
    user: AuthUser,
    _state: State<PgPool>,
) -> Result<Response, AppError> {
    user.require_rol(&["admin"])?;

    let db_url = std::env::var("DATABASE_URL")
        .map_err(|_| AppError::Internal(anyhow::anyhow!("DATABASE_URL bulunamadi")))?;

    let output = tokio::process::Command::new("pg_dump")
        .arg("--no-password")
        .arg("--dbname")
        .arg(&db_url)
        .output()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("pg_dump calismadi: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Internal(anyhow::anyhow!("pg_dump hatasi: {}", stderr)));
    }

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("koop_yedek_{}.sql", timestamp);
    let disposition = format!("attachment; filename=\"{}\"", filename);

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/octet-stream"),
            (header::CONTENT_DISPOSITION, disposition.as_str()),
        ],
        output.stdout,
    )
        .into_response())
}

// ─── Sıfırlama ────────────────────────────────────────────────────────────────

async fn sifirla(
    user: AuthUser,
    State(pool): State<PgPool>,
    Json(input): Json<SifirlaInput>,
) -> AppResult<Json<MesajResponse>> {
    user.require_rol(&["admin"])?;

    // Admin şifresini doğrula
    let sifre_hash: String =
        sqlx::query_scalar("SELECT sifre_hash FROM kullanicilar WHERE id = $1")
            .bind(user.id())
            .fetch_one(&pool)
            .await?;

    let ok = bcrypt::verify(&input.sifre, &sifre_hash)
        .map_err(|e| AppError::Internal(e.into()))?;
    if !ok {
        return Err(AppError::BadRequest("Şifre hatalı".into()));
    }

    // Tüm veri tablolarını temizle (kullanicilar hariç)
    sqlx::query(
        "TRUNCATE TABLE
            kararlar,
            toplantilar,
            hisse_satis_odemeleri,
            hisse_satislari,
            hisse_atamalari,
            donem_aidat_borclari,
            hissedar_cuzdanlari,
            gelir_gider_kayitlari,
            kasa_transferleri,
            kasa_hareketleri,
            gelir_gider_kategorileri,
            donemler,
            hissedarlar,
            hisseler,
            kasalar
        RESTART IDENTITY CASCADE",
    )
    .execute(&pool)
    .await?;

    tracing::warn!("VERİTABANI SIFIRLANDI - admin id={}", user.id());

    Ok(Json(MesajResponse {
        mesaj: "Veritabanı başarıyla sıfırlandı. Tüm veriler silindi.".into(),
    }))
}
