use axum::{
    extract::{Path, State},
    routing::{get, post, delete},
    Json, Router,
};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use crate::errors::AppResult;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Hisse {
    pub id: i64,
    pub kod: String,
    pub durum: String,
    pub aciklama: Option<String>,
    pub hissedar_id: Option<i64>,
    pub hissedar_adi: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct HisseAtama {
    pub id: i64,
    pub hisse_id: i64,
    pub hissedar_id: i64,
    pub hissedar_adi: Option<String>,
    pub tarih: chrono::NaiveDate,
    pub ucret: f64,
    pub aciklama: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateHisseInput {
    pub aciklama: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateHisseTopluInput {
    pub adet: i32,
    pub aciklama: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AtamaInput {
    pub hissedar_id: i64,
    pub tarih: chrono::NaiveDate,
    pub ucret: f64,
    pub aciklama: Option<String>,
}

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/", get(get_hisseler).post(create_hisse))
        .route("/toplu", post(create_toplu))
        .route("/{id}", get(get_hisse).delete(delete_hisse))
        .route("/{id}/ata", post(ata))
        .route("/{id}/atamalar", get(get_atamalari))
        .with_state(pool)
}

async fn get_hisseler(State(pool): State<PgPool>) -> AppResult<Json<Vec<Hisse>>> {
    let liste = sqlx::query_as::<_, Hisse>(
        "SELECT h.id, h.kod, h.durum, h.aciklama,
                a.hissedar_id,
                (SELECT soyad || ' ' || ad FROM hissedarlar WHERE id = a.hissedar_id) AS hissedar_adi,
                h.created_at, h.updated_at
         FROM hisseler h
         LEFT JOIN LATERAL (
             SELECT hissedar_id FROM hisse_atamalari
             WHERE hisse_id = h.id ORDER BY tarih DESC LIMIT 1
         ) a ON true
         ORDER BY h.kod"
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(liste))
}

async fn get_hisse(State(pool): State<PgPool>, Path(id): Path<i64>) -> AppResult<Json<Hisse>> {
    let hisse = sqlx::query_as::<_, Hisse>(
        "SELECT h.id, h.kod, h.durum, h.aciklama,
                a.hissedar_id,
                (SELECT soyad || ' ' || ad FROM hissedarlar WHERE id = a.hissedar_id) AS hissedar_adi,
                h.created_at, h.updated_at
         FROM hisseler h
         LEFT JOIN LATERAL (
             SELECT hissedar_id FROM hisse_atamalari
             WHERE hisse_id = h.id ORDER BY tarih DESC LIMIT 1
         ) a ON true
         WHERE h.id = $1"
    )
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(hisse))
}

async fn create_hisse(
    State(pool): State<PgPool>,
    Json(input): Json<CreateHisseInput>,
) -> AppResult<Json<Hisse>> {
    let son_no: Option<i64> = sqlx::query_scalar(
        "SELECT COUNT(*) FROM hisseler"
    )
    .fetch_one(&pool)
    .await?;

    let yeni_no = son_no.unwrap_or(0) + 1;
    let kod = format!("H{:04}", yeni_no);

    let hisse = sqlx::query_as::<_, Hisse>(
        "INSERT INTO hisseler (kod, aciklama)
         VALUES ($1, $2)
         RETURNING id, kod, durum, aciklama,
                   NULL::BIGINT AS hissedar_id, NULL::TEXT AS hissedar_adi,
                   created_at, updated_at"
    )
    .bind(kod)
    .bind(input.aciklama)
    .fetch_one(&pool)
    .await?;
    Ok(Json(hisse))
}

async fn create_toplu(
    State(pool): State<PgPool>,
    Json(input): Json<CreateHisseTopluInput>,
) -> AppResult<Json<Vec<Hisse>>> {
    let mevcut: Option<i64> = sqlx::query_scalar("SELECT COUNT(*) FROM hisseler")
        .fetch_one(&pool)
        .await?;
    let baslangic = mevcut.unwrap_or(0);

    let mut hisseler = Vec::new();
    for i in 0..input.adet {
        let kod = format!("H{:04}", baslangic + (i as i64) + 1);
        let h = sqlx::query_as::<_, Hisse>(
            "INSERT INTO hisseler (kod, aciklama)
             VALUES ($1, $2)
             RETURNING id, kod, durum, aciklama,
                       NULL::BIGINT AS hissedar_id, NULL::TEXT AS hissedar_adi,
                       created_at, updated_at"
        )
        .bind(kod)
        .bind(input.aciklama.clone())
        .fetch_one(&pool)
        .await?;
        hisseler.push(h);
    }
    Ok(Json(hisseler))
}

async fn ata(
    State(pool): State<PgPool>,
    Path(hisse_id): Path<i64>,
    Json(input): Json<AtamaInput>,
) -> AppResult<Json<HisseAtama>> {
    let atama = sqlx::query_as::<_, HisseAtama>(
        "INSERT INTO hisse_atamalari (hisse_id, hissedar_id, tarih, ucret, aciklama)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, hisse_id, hissedar_id,
                   (SELECT soyad || ' ' || ad FROM hissedarlar WHERE id = $2) AS hissedar_adi,
                   tarih, ucret, aciklama, created_at"
    )
    .bind(hisse_id)
    .bind(input.hissedar_id)
    .bind(input.tarih)
    .bind(input.ucret)
    .bind(input.aciklama)
    .fetch_one(&pool)
    .await?;

    sqlx::query("UPDATE hisseler SET durum = 'atanmis', updated_at = NOW() WHERE id = $1")
        .bind(hisse_id)
        .execute(&pool)
        .await?;

    Ok(Json(atama))
}

async fn get_atamalari(
    State(pool): State<PgPool>,
    Path(hisse_id): Path<i64>,
) -> AppResult<Json<Vec<HisseAtama>>> {
    let liste = sqlx::query_as::<_, HisseAtama>(
        "SELECT a.id, a.hisse_id, a.hissedar_id,
                (h.soyad || ' ' || h.ad) AS hissedar_adi,
                a.tarih, a.ucret, a.aciklama, a.created_at
         FROM hisse_atamalari a
         JOIN hissedarlar h ON h.id = a.hissedar_id
         WHERE a.hisse_id = $1 ORDER BY a.tarih DESC"
    )
    .bind(hisse_id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(liste))
}

async fn delete_hisse(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    sqlx::query("DELETE FROM hisseler WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(Json(serde_json::json!({ "mesaj": "Hisse silindi" })))
}


