use axum::{
    extract::{Path, State},
    routing::{get, post, put, delete},
    Json, Router,
};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use crate::auth::AuthUser;
use crate::errors::AppResult;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GelirGiderKategori {
    pub id: i64,
    pub ad: String,
    pub tip: String,
    pub aciklama: Option<String>,
    pub aktif: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GelirGiderKayit {
    pub id: i64,
    pub kasa_id: i64,
    pub kategori_id: i64,
    pub kategori_ad: Option<String>,
    pub tarih: chrono::NaiveDate,
    pub tutar: f64,
    pub aciklama: String,
    pub kasa_hareketi_id: Option<i64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateKategoriInput {
    pub ad: String,
    pub tip: String,
    pub aciklama: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateKayitInput {
    pub kasa_id: i64,
    pub kategori_id: i64,
    pub tarih: chrono::NaiveDate,
    pub tutar: f64,
    pub aciklama: String,
}

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/kategoriler", get(get_kategoriler).post(create_kategori))
        .route("/kategoriler/{id}", put(update_kategori).delete(delete_kategori))
        .route("/kayitlar", get(get_kayitlar).post(create_kayit))
        .route("/kayitlar/{id}", delete(delete_kayit))
        .with_state(pool)
}

async fn get_kategoriler(user: AuthUser, State(pool): State<PgPool>) -> AppResult<Json<Vec<GelirGiderKategori>>> {
    user.require_izin(&pool, "gelir_gider.goruntule").await?;
    let liste = sqlx::query_as::<_, GelirGiderKategori>(
        "SELECT id, ad, tip, aciklama, aktif, created_at, updated_at
         FROM gelir_gider_kategorileri ORDER BY tip, ad"
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(liste))
}

async fn create_kategori(
    user: AuthUser,
    State(pool): State<PgPool>,
    Json(input): Json<CreateKategoriInput>,
) -> AppResult<Json<GelirGiderKategori>> {
    user.require_izin(&pool, "gelir_gider.yonet").await?;
    let k = sqlx::query_as::<_, GelirGiderKategori>(
        "INSERT INTO gelir_gider_kategorileri (ad, tip, aciklama)
         VALUES ($1, $2, $3)
         RETURNING id, ad, tip, aciklama, aktif, created_at, updated_at"
    )
    .bind(input.ad)
    .bind(input.tip)
    .bind(input.aciklama)
    .fetch_one(&pool)
    .await?;
    Ok(Json(k))
}

async fn update_kategori(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(input): Json<CreateKategoriInput>,
) -> AppResult<Json<GelirGiderKategori>> {
    user.require_izin(&pool, "gelir_gider.yonet").await?;
    let k = sqlx::query_as::<_, GelirGiderKategori>(
        "UPDATE gelir_gider_kategorileri SET ad=$1, tip=$2, aciklama=$3, updated_at=NOW()
         WHERE id=$4
         RETURNING id, ad, tip, aciklama, aktif, created_at, updated_at"
    )
    .bind(input.ad)
    .bind(input.tip)
    .bind(input.aciklama)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(k))
}

async fn delete_kategori(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_izin(&pool, "gelir_gider.yonet").await?;
    sqlx::query("DELETE FROM gelir_gider_kategorileri WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(Json(serde_json::json!({ "mesaj": "Kategori silindi" })))
}

async fn get_kayitlar(user: AuthUser, State(pool): State<PgPool>) -> AppResult<Json<Vec<GelirGiderKayit>>> {
    user.require_izin(&pool, "gelir_gider.goruntule").await?;
    let liste = sqlx::query_as::<_, GelirGiderKayit>(
        "SELECT g.id, g.kasa_id, g.kategori_id, k.ad AS kategori_ad,
                g.tarih, g.tutar, g.aciklama, g.kasa_hareketi_id, g.created_at
         FROM gelir_gider_kayitlari g
         LEFT JOIN gelir_gider_kategorileri k ON k.id = g.kategori_id
         ORDER BY g.tarih DESC, g.id DESC"
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(liste))
}

async fn create_kayit(
    user: AuthUser,
    State(pool): State<PgPool>,
    Json(input): Json<CreateKayitInput>,
) -> AppResult<Json<GelirGiderKayit>> {
    user.require_izin(&pool, "gelir_gider.yonet").await?;
    let mut tx = pool.begin().await?;

    // Kategori tipini Ã¶ÄŸren (gelir/gider)
    let tip: Option<String> = sqlx::query_scalar(
        "SELECT tip FROM gelir_gider_kategorileri WHERE id = $1"
    )
    .bind(input.kategori_id)
    .fetch_optional(&mut *tx)
    .await?;

    let (giren, cikan) = match tip.as_deref() {
        Some("gelir") => (input.tutar, 0.0_f64),
        _             => (0.0_f64, input.tutar),
    };

    // Son bakiyeyi bul
    let son_bakiye: f64 = sqlx::query_scalar::<_, Option<f64>>(
        "SELECT bakiye FROM kasa_hareketleri
         WHERE kasa_id = $1 ORDER BY tarih DESC, id DESC LIMIT 1"
    )
    .bind(input.kasa_id)
    .fetch_optional(&mut *tx)
    .await?
    .flatten()
    .unwrap_or(0.0);

    let yeni_bakiye = son_bakiye + giren - cikan;

    // Kasa hareketi ekle
    let hareket_id: i64 = sqlx::query_scalar(
        "INSERT INTO kasa_hareketleri (kasa_id, tarih, aciklama, giren, cikan, bakiye)
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"
    )
    .bind(input.kasa_id)
    .bind(input.tarih)
    .bind(input.aciklama.clone())
    .bind(giren)
    .bind(cikan)
    .bind(yeni_bakiye)
    .fetch_one(&mut *tx)
    .await?;

    // Kasa bakiyesini gÃ¼ncelle
    sqlx::query("UPDATE kasalar SET bakiye = $1, updated_at = NOW() WHERE id = $2")
        .bind(yeni_bakiye)
        .bind(input.kasa_id)
        .execute(&mut *tx)
        .await?;

    // KayÄ±t ekle
    let kayit = sqlx::query_as::<_, GelirGiderKayit>(
        "INSERT INTO gelir_gider_kayitlari (kasa_id, kategori_id, tarih, tutar, aciklama, kasa_hareketi_id)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, kasa_id, kategori_id, NULL::TEXT AS kategori_ad,
                   tarih, tutar, aciklama, kasa_hareketi_id, created_at"
    )
    .bind(input.kasa_id)
    .bind(input.kategori_id)
    .bind(input.tarih)
    .bind(input.tutar)
    .bind(input.aciklama)
    .bind(hareket_id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(kayit))
}

async fn delete_kayit(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_izin(&pool, "gelir_gider.yonet").await?;
    sqlx::query("DELETE FROM gelir_gider_kayitlari WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(Json(serde_json::json!({ "mesaj": "KayÄ±t silindi" })))
}
