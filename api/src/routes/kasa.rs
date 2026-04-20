use axum::{
    extract::{Path, State},
    routing::{get, post, put, delete},
    Json, Router,
};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use crate::errors::AppResult;

// --- Modeller ---

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Kasa {
    pub id: i64,
    pub ad: String,
    pub para_birimi: String,
    pub bakiye: f64,
    pub aciklama: Option<String>,
    pub aktif: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateKasaInput {
    pub ad: String,
    pub para_birimi: String,
    pub aciklama: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateKasaInput {
    pub ad: Option<String>,
    pub para_birimi: Option<String>,
    pub aciklama: Option<String>,
    pub aktif: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct KasaHareketi {
    pub id: i64,
    pub kasa_id: i64,
    pub tarih: chrono::NaiveDate,
    pub aciklama: String,
    pub giren: f64,
    pub cikan: f64,
    pub bakiye: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateHareket {
    pub kasa_id: i64,
    pub tarih: chrono::NaiveDate,
    pub aciklama: String,
    pub giren: f64,
    pub cikan: f64,
}

// --- Router ---

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/", get(get_kasalar).post(create_kasa))
        .route("/{id}", get(get_kasa).put(update_kasa).delete(delete_kasa))
        .route("/{id}/hareketler", get(get_hareketler).post(create_hareket))
        .route("/{id}/hareketler/{hareket_id}", delete(delete_hareket))
        .with_state(pool)
}

// --- Handler'lar ---

async fn get_kasalar(State(pool): State<PgPool>) -> AppResult<Json<Vec<Kasa>>> {
    let kasalar = sqlx::query_as::<_, Kasa>(
        "SELECT id, ad, para_birimi, bakiye, aciklama, aktif, created_at, updated_at
         FROM kasalar ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(kasalar))
}

async fn get_kasa(State(pool): State<PgPool>, Path(id): Path<i64>) -> AppResult<Json<Kasa>> {
    let kasa = sqlx::query_as::<_, Kasa>(
        "SELECT id, ad, para_birimi, bakiye, aciklama, aktif, created_at, updated_at
         FROM kasalar WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(kasa))
}

async fn create_kasa(
    State(pool): State<PgPool>,
    Json(input): Json<CreateKasaInput>,
) -> AppResult<Json<Kasa>> {
    let kasa = sqlx::query_as::<_, Kasa>(
        "INSERT INTO kasalar (ad, para_birimi, aciklama)
         VALUES ($1, $2, $3)
         RETURNING id, ad, para_birimi, bakiye, aciklama, aktif, created_at, updated_at",
    )
    .bind(input.ad)
    .bind(input.para_birimi)
    .bind(input.aciklama)
    .fetch_one(&pool)
    .await?;
    Ok(Json(kasa))
}

async fn update_kasa(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateKasaInput>,
) -> AppResult<Json<Kasa>> {
    let kasa = sqlx::query_as::<_, Kasa>(
        "UPDATE kasalar SET
            ad          = COALESCE($1, ad),
            para_birimi = COALESCE($2, para_birimi),
            aciklama    = COALESCE($3, aciklama),
            aktif       = COALESCE($4, aktif),
            updated_at  = NOW()
         WHERE id = $5
         RETURNING id, ad, para_birimi, bakiye, aciklama, aktif, created_at, updated_at",
    )
    .bind(input.ad)
    .bind(input.para_birimi)
    .bind(input.aciklama)
    .bind(input.aktif)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(kasa))
}

async fn delete_kasa(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    sqlx::query("DELETE FROM kasalar WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(Json(serde_json::json!({ "mesaj": "Kasa silindi" })))
}

async fn get_hareketler(
    State(pool): State<PgPool>,
    Path(kasa_id): Path<i64>,
) -> AppResult<Json<Vec<KasaHareketi>>> {
    let hareketler = sqlx::query_as::<_, KasaHareketi>(
        "SELECT id, kasa_id, tarih, aciklama, giren, cikan, bakiye, created_at
         FROM kasa_hareketleri WHERE kasa_id = $1 ORDER BY tarih DESC, id DESC",
    )
    .bind(kasa_id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(hareketler))
}

async fn create_hareket(
    State(pool): State<PgPool>,
    Json(input): Json<CreateHareket>,
) -> AppResult<Json<KasaHareketi>> {
    let son_bakiye: f64 = sqlx::query_scalar::<_, Option<f64>>(
        "SELECT bakiye FROM kasa_hareketleri
         WHERE kasa_id = $1 ORDER BY tarih DESC, id DESC LIMIT 1",
    )
    .bind(input.kasa_id)
    .fetch_optional(&pool)
    .await?
    .flatten()
    .unwrap_or(0.0);

    let yeni_bakiye = son_bakiye + input.giren - input.cikan;

    let hareket = sqlx::query_as::<_, KasaHareketi>(
        "INSERT INTO kasa_hareketleri (kasa_id, tarih, aciklama, giren, cikan, bakiye)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, kasa_id, tarih, aciklama, giren, cikan, bakiye, created_at",
    )
    .bind(input.kasa_id)
    .bind(input.tarih)
    .bind(input.aciklama)
    .bind(input.giren)
    .bind(input.cikan)
    .bind(yeni_bakiye)
    .fetch_one(&pool)
    .await?;

    sqlx::query("UPDATE kasalar SET bakiye = $1, updated_at = NOW() WHERE id = $2")
        .bind(yeni_bakiye)
        .bind(input.kasa_id)
        .execute(&pool)
        .await?;

    Ok(Json(hareket))
}

async fn delete_hareket(
    State(pool): State<PgPool>,
    Path((kasa_id, hareket_id)): Path<(i64, i64)>,
) -> AppResult<Json<serde_json::Value>> {
    sqlx::query("DELETE FROM kasa_hareketleri WHERE id = $1 AND kasa_id = $2")
        .bind(hareket_id)
        .bind(kasa_id)
        .execute(&pool)
        .await?;
    Ok(Json(serde_json::json!({ "mesaj": "Hareket silindi" })))
}
