use axum::{
    extract::{Path, State},
    routing::{get, post, put, delete},
    Json, Router,
};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use crate::errors::AppResult;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Hissedar {
    pub id: i64,
    pub ad: String,
    pub soyad: String,
    pub kasa_id: i64,
    pub kasa_ad: Option<String>,
    pub aile_sira_no: Option<i64>,
    pub tcno: Option<String>,
    pub tel: Option<String>,
    pub yakin_adi: Option<String>,
    pub yakinlik_derecesi: Option<String>,
    pub aktif: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateHissedarInput {
    pub ad: String,
    pub soyad: String,
    pub kasa_id: i64,
    pub aile_sira_no: Option<i64>,
    pub tcno: Option<String>,
    pub tel: Option<String>,
    pub yakin_adi: Option<String>,
    pub yakinlik_derecesi: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateHissedarInput {
    pub ad: Option<String>,
    pub soyad: Option<String>,
    pub kasa_id: Option<i64>,
    pub aile_sira_no: Option<i64>,
    pub tcno: Option<String>,
    pub tel: Option<String>,
    pub yakin_adi: Option<String>,
    pub yakinlik_derecesi: Option<String>,
    pub aktif: Option<bool>,
}

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/", get(get_hissedarlar).post(create_hissedar))
        .route("/{id}", get(get_hissedar).put(update_hissedar).delete(delete_hissedar))
        .with_state(pool)
}

async fn get_hissedarlar(State(pool): State<PgPool>) -> AppResult<Json<Vec<Hissedar>>> {
    let liste = sqlx::query_as::<_, Hissedar>(
        "SELECT h.id, h.ad, h.soyad, h.kasa_id, k.ad AS kasa_ad,
                h.aile_sira_no, h.tcno, h.tel, h.yakin_adi, h.yakinlik_derecesi,
                h.aktif, h.created_at, h.updated_at
         FROM hissedarlar h
         LEFT JOIN kasalar k ON k.id = h.kasa_id
         ORDER BY h.soyad, h.ad"
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(liste))
}

async fn get_hissedar(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<Hissedar>> {
    let hissedar = sqlx::query_as::<_, Hissedar>(
        "SELECT h.id, h.ad, h.soyad, h.kasa_id, k.ad AS kasa_ad,
                h.aile_sira_no, h.tcno, h.tel, h.yakin_adi, h.yakinlik_derecesi,
                h.aktif, h.created_at, h.updated_at
         FROM hissedarlar h
         LEFT JOIN kasalar k ON k.id = h.kasa_id
         WHERE h.id = $1"
    )
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(hissedar))
}

async fn create_hissedar(
    State(pool): State<PgPool>,
    Json(input): Json<CreateHissedarInput>,
) -> AppResult<Json<Hissedar>> {
    let hissedar = sqlx::query_as::<_, Hissedar>(
        "INSERT INTO hissedarlar (ad, soyad, kasa_id, aile_sira_no, tcno, tel, yakin_adi, yakinlik_derecesi)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id, ad, soyad, kasa_id, NULL::TEXT AS kasa_ad,
                   aile_sira_no, tcno, tel, yakin_adi, yakinlik_derecesi,
                   aktif, created_at, updated_at"
    )
    .bind(input.ad)
    .bind(input.soyad)
    .bind(input.kasa_id)
    .bind(input.aile_sira_no)
    .bind(input.tcno)
    .bind(input.tel)
    .bind(input.yakin_adi)
    .bind(input.yakinlik_derecesi)
    .fetch_one(&pool)
    .await?;
    Ok(Json(hissedar))
}

async fn update_hissedar(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateHissedarInput>,
) -> AppResult<Json<Hissedar>> {
    let hissedar = sqlx::query_as::<_, Hissedar>(
        "UPDATE hissedarlar SET
            ad                = COALESCE($1, ad),
            soyad             = COALESCE($2, soyad),
            kasa_id           = COALESCE($3, kasa_id),
            aile_sira_no      = COALESCE($4, aile_sira_no),
            tcno              = COALESCE($5, tcno),
            tel               = COALESCE($6, tel),
            yakin_adi         = COALESCE($7, yakin_adi),
            yakinlik_derecesi = COALESCE($8, yakinlik_derecesi),
            aktif             = COALESCE($9, aktif),
            updated_at        = NOW()
         WHERE id = $10
         RETURNING id, ad, soyad, kasa_id, NULL::TEXT AS kasa_ad,
                   aile_sira_no, tcno, tel, yakin_adi, yakinlik_derecesi,
                   aktif, created_at, updated_at"
    )
    .bind(input.ad)
    .bind(input.soyad)
    .bind(input.kasa_id)
    .bind(input.aile_sira_no)
    .bind(input.tcno)
    .bind(input.tel)
    .bind(input.yakin_adi)
    .bind(input.yakinlik_derecesi)
    .bind(input.aktif)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(hissedar))
}

async fn delete_hissedar(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    sqlx::query("DELETE FROM hissedarlar WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(Json(serde_json::json!({ "mesaj": "Hissedar silindi" })))
}


