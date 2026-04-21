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
pub struct Donem {
    pub id: i64,
    pub ay: i32,
    pub yil: i32,
    pub hisse_basi_aidat: f64,
    pub aktif: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDonemInput {
    pub ay: i32,
    pub yil: i32,
    pub hisse_basi_aidat: f64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Toplanti {
    pub id: i64,
    pub donem_id: i64,
    pub tarih: chrono::NaiveDate,
    pub konu: String,
    pub yer: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateToplantiInput {
    pub donem_id: i64,
    pub tarih: chrono::NaiveDate,
    pub konu: String,
    pub yer: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Karar {
    pub id: i64,
    pub toplanti_id: i64,
    pub karar_no: Option<i32>,
    pub aciklama: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateKararInput {
    pub toplanti_id: i64,
    pub karar_no: Option<i32>,
    pub aciklama: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AidatBorc {
    pub id: i64,
    pub donem_id: i64,
    pub hissedar_id: i64,
    pub hissedar_ad: Option<String>,
    pub hisse_sayisi: i32,
    pub tutar: f64,
    pub odendi: bool,
    pub odeme_tarihi: Option<chrono::NaiveDate>,
    pub aciklama: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/", get(get_donemler).post(create_donem))
        .route("/{id}", get(get_donem).put(update_donem).delete(delete_donem))
        .route("/{id}/toplantilar", get(get_toplantilar).post(create_toplanti))
        .route("/{id}/borclar", get(get_donem_borclari).post(donem_borc_olustur))
        .with_state(pool)
}

/// Toplantı ve Karar route'ları (/api/toplantilar/*, /api/kararlar/*)
pub fn toplanti_router(pool: PgPool) -> Router {
    Router::new()
        .route("/{id}", put(update_toplanti).delete(delete_toplanti))
        .route("/{id}/kararlar", get(get_kararlar).post(create_karar))
        .with_state(pool)
}

pub fn karar_router(pool: PgPool) -> Router {
    Router::new()
        .route("/{id}", put(update_karar).delete(delete_karar))
        .with_state(pool)
}

async fn get_donemler(user: AuthUser, State(pool): State<PgPool>) -> AppResult<Json<Vec<Donem>>> {
    user.require_izin(&pool, "donem.goruntule").await?;
    let liste = sqlx::query_as::<_, Donem>(
        "SELECT id, ay, yil, hisse_basi_aidat, aktif, created_at, updated_at
         FROM donemler ORDER BY yil DESC, ay DESC"
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(liste))
}

async fn get_donem(user: AuthUser, State(pool): State<PgPool>, Path(id): Path<i64>) -> AppResult<Json<Donem>> {
    user.require_izin(&pool, "donem.goruntule").await?;
    let donem = sqlx::query_as::<_, Donem>(
        "SELECT id, ay, yil, hisse_basi_aidat, aktif, created_at, updated_at
         FROM donemler WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(donem))
}

async fn create_donem(
    user: AuthUser,
    State(pool): State<PgPool>,
    Json(input): Json<CreateDonemInput>,
) -> AppResult<Json<Donem>> {
    user.require_izin(&pool, "donem.yonet").await?;
    let donem = sqlx::query_as::<_, Donem>(
        "INSERT INTO donemler (ay, yil, hisse_basi_aidat)
         VALUES ($1, $2, $3)
         RETURNING id, ay, yil, hisse_basi_aidat, aktif, created_at, updated_at"
    )
    .bind(input.ay)
    .bind(input.yil)
    .bind(input.hisse_basi_aidat)
    .fetch_one(&pool)
    .await?;
    Ok(Json(donem))
}

async fn update_donem(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(input): Json<CreateDonemInput>,
) -> AppResult<Json<Donem>> {
    user.require_izin(&pool, "donem.yonet").await?;
    let donem = sqlx::query_as::<_, Donem>(
        "UPDATE donemler SET ay=$1, yil=$2, hisse_basi_aidat=$3, updated_at=NOW()
         WHERE id = $4
         RETURNING id, ay, yil, hisse_basi_aidat, aktif, created_at, updated_at"
    )
    .bind(input.ay)
    .bind(input.yil)
    .bind(input.hisse_basi_aidat)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(donem))
}

async fn delete_donem(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_izin(&pool, "donem.yonet").await?;
    sqlx::query("DELETE FROM donemler WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(Json(serde_json::json!({ "mesaj": "Dönem silindi" })))
}

// ─── Toplantı Handlers ──────────────────────────────────────────────────────

async fn get_toplantilar(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<Vec<Toplanti>>> {
    user.require_izin(&pool, "donem.goruntule").await?;
    let liste = sqlx::query_as::<_, Toplanti>(
        "SELECT id, donem_id, tarih, konu, yer, created_at, updated_at
         FROM toplantilar WHERE donem_id = $1 ORDER BY tarih DESC"
    )
    .bind(id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(liste))
}

async fn create_toplanti(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(donem_id): Path<i64>,
    Json(input): Json<CreateToplantiInput>,
) -> AppResult<Json<Toplanti>> {
    user.require_izin(&pool, "toplanti.yonet").await?;
    let toplanti = sqlx::query_as::<_, Toplanti>(
        "INSERT INTO toplantilar (donem_id, tarih, konu, yer)
         VALUES ($1, $2, $3, $4)
         RETURNING id, donem_id, tarih, konu, yer, created_at, updated_at"
    )
    .bind(donem_id)
    .bind(input.tarih)
    .bind(input.konu)
    .bind(input.yer)
    .fetch_one(&pool)
    .await?;
    Ok(Json(toplanti))
}

async fn update_toplanti(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(input): Json<CreateToplantiInput>,
) -> AppResult<Json<Toplanti>> {
    user.require_izin(&pool, "toplanti.yonet").await?;
    let toplanti = sqlx::query_as::<_, Toplanti>(
        "UPDATE toplantilar SET tarih=$1, konu=$2, yer=$3, updated_at=NOW()
         WHERE id = $4
         RETURNING id, donem_id, tarih, konu, yer, created_at, updated_at"
    )
    .bind(input.tarih)
    .bind(input.konu)
    .bind(input.yer)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(toplanti))
}

async fn delete_toplanti(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_izin(&pool, "toplanti.yonet").await?;
    sqlx::query("DELETE FROM toplantilar WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(Json(serde_json::json!({ "mesaj": "Toplantı silindi" })))
}

// ─── Karar Handlers ─────────────────────────────────────────────────────────

async fn get_kararlar(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(toplanti_id): Path<i64>,
) -> AppResult<Json<Vec<Karar>>> {
    user.require_izin(&pool, "donem.goruntule").await?;
    let liste = sqlx::query_as::<_, Karar>(
        "SELECT id, toplanti_id, karar_no, aciklama, created_at
         FROM kararlar WHERE toplanti_id = $1 ORDER BY karar_no ASC"
    )
    .bind(toplanti_id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(liste))
}

async fn create_karar(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(toplanti_id): Path<i64>,
    Json(input): Json<CreateKararInput>,
) -> AppResult<Json<Karar>> {
    user.require_izin(&pool, "toplanti.yonet").await?;
    let karar = sqlx::query_as::<_, Karar>(
        "INSERT INTO kararlar (toplanti_id, karar_no, aciklama)
         VALUES ($1, $2, $3)
         RETURNING id, toplanti_id, karar_no, aciklama, created_at"
    )
    .bind(toplanti_id)
    .bind(input.karar_no)
    .bind(input.aciklama)
    .fetch_one(&pool)
    .await?;
    Ok(Json(karar))
}

async fn update_karar(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(input): Json<CreateKararInput>,
) -> AppResult<Json<Karar>> {
    user.require_izin(&pool, "toplanti.yonet").await?;
    let karar = sqlx::query_as::<_, Karar>(
        "UPDATE kararlar SET karar_no=$1, aciklama=$2
         WHERE id = $3
         RETURNING id, toplanti_id, karar_no, aciklama, created_at"
    )
    .bind(input.karar_no)
    .bind(input.aciklama)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(karar))
}

async fn delete_karar(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_izin(&pool, "toplanti.yonet").await?;
    sqlx::query("DELETE FROM kararlar WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(Json(serde_json::json!({ "mesaj": "Karar silindi" })))
}

// ─── Aidat Borç Handlers ────────────────────────────────────────────────────

async fn get_donem_borclari(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<Vec<AidatBorc>>> {
    user.require_izin(&pool, "donem.goruntule").await?;
    let liste = sqlx::query_as::<_, AidatBorc>(
        "SELECT b.id, b.donem_id, b.hissedar_id,
                h.ad || ' ' || h.soyad AS hissedar_ad,
                b.hisse_sayisi, b.tutar, b.odendi, b.odeme_tarihi, b.aciklama, b.created_at
         FROM donem_aidat_borclari b
         JOIN hissedarlar h ON h.id = b.hissedar_id
         WHERE b.donem_id = $1
         ORDER BY h.ad, h.soyad"
    )
    .bind(id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(liste))
}

async fn donem_borc_olustur(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(donem_id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_izin(&pool, "donem.yonet").await?;
    // Dönemin aidat miktarını al
    let donem = sqlx::query_as::<_, Donem>(
        "SELECT id, ay, yil, hisse_basi_aidat, aktif, created_at, updated_at
         FROM donemler WHERE id = $1"
    )
    .bind(donem_id)
    .fetch_one(&pool)
    .await?;

    let donem_adi = format!("{}/{}", donem.ay, donem.yil);
    let tarih = chrono::Utc::now().date_naive();

    // Borcu olmayan aktif hissedarları + hisse sayılarını topla
    #[derive(sqlx::FromRow)]
    struct Hedef {
        id: i64,
        hisse_sayisi: i64,
    }
    let hedefler: Vec<Hedef> = sqlx::query_as(
        "SELECT h.id,
                COALESCE((SELECT COUNT(*) FROM hisse_atamalari ha
                          JOIN hisseler hs ON hs.id = ha.hisse_id
                          WHERE ha.hissedar_id = h.id AND hs.durum = 'atanmis'), 0) AS hisse_sayisi
         FROM hissedarlar h
         WHERE h.aktif = true
           AND NOT EXISTS (
               SELECT 1 FROM donem_aidat_borclari b
               WHERE b.donem_id = $1 AND b.hissedar_id = h.id
           )"
    )
    .bind(donem_id)
    .fetch_all(&pool)
    .await?;

    let mut eklenen: u64 = 0;
    for h in hedefler {
        if h.hisse_sayisi <= 0 { continue; }
        let tutar = (h.hisse_sayisi as f64) * donem.hisse_basi_aidat;

        // Borç kaydı
        sqlx::query(
            "INSERT INTO donem_aidat_borclari (donem_id, hissedar_id, hisse_sayisi, tutar)
             VALUES ($1, $2, $3, $4)"
        )
        .bind(donem_id)
        .bind(h.id)
        .bind(h.hisse_sayisi)
        .bind(tutar)
        .execute(&pool)
        .await?;

        // Cüzdana borç yansıt
        let onceki: Option<f64> = sqlx::query_scalar(
            "SELECT bakiye FROM hissedar_cuzdanlari
             WHERE hissedar_id = $1 ORDER BY id DESC LIMIT 1"
        )
        .bind(h.id)
        .fetch_optional(&pool)
        .await?;
        let yeni = onceki.unwrap_or(0.0) - tutar;
        let bilgi = format!("{} aidatı - {} hisse", donem_adi, h.hisse_sayisi);
        sqlx::query(
            "INSERT INTO hissedar_cuzdanlari (hissedar_id, donem_id, tarih, bilgi, borc, alacak, bakiye)
             VALUES ($1, $2, $3, $4, $5, 0.0, $6)"
        )
        .bind(h.id)
        .bind(donem_id)
        .bind(tarih)
        .bind(&bilgi)
        .bind(tutar)
        .bind(yeni)
        .execute(&pool)
        .await?;

        eklenen += 1;
    }

    Ok(Json(serde_json::json!({
        "mesaj": format!("{} hissedar için borç oluşturuldu", eklenen),
        "eklenen": eklenen
    })))
}
