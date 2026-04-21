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
    use super::common::{
        cuzdan_alacak_ekle, cuzdan_borc_ekle, cuzdan_son_bakiye, donem_adi as format_donem_adi,
        kasa_giren_ekle, tahsilat_aciklamasi,
    };
    user.require_izin(&pool, "donem.yonet").await?;
    // Dönemin aidat miktarını al
    let donem = sqlx::query_as::<_, Donem>(
        "SELECT id, ay, yil, hisse_basi_aidat, aktif, created_at, updated_at
         FROM donemler WHERE id = $1"
    )
    .bind(donem_id)
    .fetch_one(&pool)
    .await?;

    if donem.hisse_basi_aidat <= 0.0 {
        return Err(crate::errors::AppError::BadRequest(
            "Hisse başı aidat tutarı 0 veya negatif, borç oluşturulamaz".into()
        ));
    }

    let donem_ad = format_donem_adi(donem.ay, donem.yil);
    let tarih = chrono::Utc::now().date_naive();

    // Atanmış hisseleri toplayan hissedar bazlı gruplama (Tauri ile aynı mantık)
    #[derive(sqlx::FromRow)]
    struct Hedef {
        id: i64,
        ad: String,
        soyad: String,
        kasa_id: i64,
        yakin_adi: Option<String>,
        yakinlik_derecesi: Option<String>,
        hisse_sayisi: i64,
        hisse_kodlari: Option<String>,
    }
    let hedefler: Vec<Hedef> = sqlx::query_as(
        "SELECT h.id, h.ad, h.soyad, h.kasa_id, h.yakin_adi, h.yakinlik_derecesi,
                COUNT(hs.id) AS hisse_sayisi,
                STRING_AGG(hs.kod, ', ' ORDER BY hs.kod) AS hisse_kodlari
         FROM hissedarlar h
         JOIN hisse_atamalari ha ON ha.hissedar_id = h.id
         JOIN hisseler hs ON hs.id = ha.hisse_id AND hs.durum = 'atanmis'
         WHERE h.aktif = true
           AND ha.id = (
               SELECT id FROM hisse_atamalari
               WHERE hisse_id = hs.id
               ORDER BY created_at DESC, id DESC LIMIT 1
           )
           AND NOT EXISTS (
               SELECT 1 FROM donem_aidat_borclari b
               WHERE b.donem_id = $1 AND b.hissedar_id = h.id
           )
         GROUP BY h.id, h.ad, h.soyad, h.kasa_id, h.yakin_adi, h.yakinlik_derecesi"
    )
    .bind(donem_id)
    .fetch_all(&pool)
    .await?;

    let mut olusturulan: u64 = 0;
    let mut otomatik_tahsil: u64 = 0;
    let mut tahsil_edilemeyen: u64 = 0;

    for h in &hedefler {
        if h.hisse_sayisi <= 0 { continue; }

        let toplam = (h.hisse_sayisi as f64) * donem.hisse_basi_aidat;
        let hisse_kodlari_str = h.hisse_kodlari.as_deref().unwrap_or("");
        let borc_aciklama = format!(
            "{} aidatı - {} hisse ({}) ({} {})",
            donem_ad, h.hisse_sayisi, hisse_kodlari_str, h.ad, h.soyad
        );

        let onceki_bakiye = cuzdan_son_bakiye(&pool, h.id).await?;
        let yeterli = onceki_bakiye >= toplam;

        // donem_aidat_borclari INSERT
        sqlx::query(
            "INSERT INTO donem_aidat_borclari
                 (donem_id, hissedar_id, hisse_sayisi, tutar, odendi, odeme_tarihi, aciklama)
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(donem_id)
        .bind(h.id)
        .bind(h.hisse_sayisi as i32)
        .bind(toplam)
        .bind(yeterli)
        .bind(if yeterli { Some(tarih) } else { None })
        .bind(&borc_aciklama)
        .execute(&pool)
        .await?;
        olusturulan += 1;

        // Cüzdana borç
        let cuzdan_borc_bilgi = format!("{} aidatı - {} hisse", donem_ad, h.hisse_sayisi);
        cuzdan_borc_ekle(&pool, h.id, Some(donem_id), tarih, &cuzdan_borc_bilgi, toplam).await?;

        if yeterli {
            // Kasaya giren
            let kasa_ac = tahsilat_aciklamasi(
                &donem_ad, h.hisse_sayisi, &h.ad, &h.soyad, &h.yakin_adi, &h.yakinlik_derecesi
            );
            kasa_giren_ekle(&pool, h.kasa_id, tarih, &kasa_ac, toplam).await?;

            // Cüzdana alacak (mahsup)
            let tahsil_bilgi = format!("Tahsilat: {} - {} hisse", donem_ad, h.hisse_sayisi);
            cuzdan_alacak_ekle(&pool, h.id, Some(donem_id), tarih, &tahsil_bilgi, toplam).await?;
            otomatik_tahsil += 1;
        } else {
            tahsil_edilemeyen += 1;
        }
    }

    Ok(Json(serde_json::json!({
        "mesaj": format!("{} hissedar için borç oluşturuldu, {} otomatik tahsil edildi",
            olusturulan, otomatik_tahsil),
        "olusturulan": olusturulan,
        "otomatik_tahsil": otomatik_tahsil,
        "tahsil_edilemeyen": tahsil_edilemeyen
    })))
}
