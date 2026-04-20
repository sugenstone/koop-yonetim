use axum::{
    extract::{Path, State},
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::AuthUser;
use crate::errors::{AppError, AppResult};

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/", get(list_izinler))
        .route("/roller/{rol}", get(get_rol_izinleri).put(set_rol_izinleri))
        .route("/benim", get(benim_izinlerim))
        .with_state(pool)
}

// ─── Modeller ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Izin {
    pub id: i64,
    pub anahtar: String,
    pub kategori: String,
    pub aciklama: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SetRolIzinleriInput {
    pub izin_ids: Vec<i64>,
}

#[derive(Debug, Serialize)]
struct MesajResponse {
    mesaj: String,
}

const GECERLI_ROLLER: &[&str] = &["admin", "muhasebe", "uye", "izleyici"];

fn rol_kontrol(rol: &str) -> AppResult<()> {
    if !GECERLI_ROLLER.contains(&rol) {
        return Err(AppError::BadRequest(format!("Gecersiz rol: {}", rol)));
    }
    Ok(())
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

/// Tüm izinleri listele (herkes okuyabilir - yetki kontrolü backend'de zaten var).
async fn list_izinler(
    _user: AuthUser,
    State(pool): State<PgPool>,
) -> AppResult<Json<Vec<Izin>>> {
    let rows = sqlx::query_as::<_, Izin>(
        "SELECT id, anahtar, kategori, aciklama, created_at
         FROM izinler ORDER BY kategori, anahtar",
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

/// Belirli bir rolün sahip olduğu izin id'lerini döndür.
async fn get_rol_izinleri(
    _user: AuthUser,
    State(pool): State<PgPool>,
    Path(rol): Path<String>,
) -> AppResult<Json<Vec<i64>>> {
    rol_kontrol(&rol)?;
    let ids: Vec<i64> = sqlx::query_scalar(
        "SELECT izin_id FROM rol_izinleri WHERE rol = $1 ORDER BY izin_id",
    )
    .bind(&rol)
    .fetch_all(&pool)
    .await?;
    Ok(Json(ids))
}

/// Bir rolün izinlerini toplu olarak güncelle (sadece admin).
/// Mevcut tüm izinler silinir, yeni liste eklenir.
async fn set_rol_izinleri(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(rol): Path<String>,
    Json(input): Json<SetRolIzinleriInput>,
) -> AppResult<Json<MesajResponse>> {
    user.require_rol(&["admin"])?;
    rol_kontrol(&rol)?;

    // Admin rolünün izinleri SİLİNEMEZ — kendi kendini kilitlememek için.
    if rol == "admin" {
        return Err(AppError::BadRequest(
            "Admin rolünün izinleri değiştirilemez (sistem güvenliği).".into(),
        ));
    }

    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM rol_izinleri WHERE rol = $1")
        .bind(&rol)
        .execute(&mut *tx)
        .await?;

    for izin_id in &input.izin_ids {
        sqlx::query("INSERT INTO rol_izinleri (rol, izin_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(&rol)
            .bind(izin_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    Ok(Json(MesajResponse {
        mesaj: format!("{} rolü için {} izin kaydedildi", rol, input.izin_ids.len()),
    }))
}

/// Giris yapmış kullanıcının izin anahtarlarını döndür.
/// Frontend, UI'da buton/link gösterip gizlemek için kullanır.
async fn benim_izinlerim(
    user: AuthUser,
    State(pool): State<PgPool>,
) -> AppResult<Json<Vec<String>>> {
    let anahtarlar: Vec<String> = sqlx::query_scalar(
        "SELECT i.anahtar FROM izinler i
         JOIN rol_izinleri ri ON ri.izin_id = i.id
         WHERE ri.rol = $1
         ORDER BY i.anahtar",
    )
    .bind(user.rol())
    .fetch_all(&pool)
    .await?;
    Ok(Json(anahtarlar))
}
