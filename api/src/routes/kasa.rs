use axum::{
    extract::{Path, State},
    routing::{get, post, put, delete},
    Json, Router,
};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use crate::auth::AuthUser;
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

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct KasaTransfer {
    pub id: i64,
    pub kaynak_kasa_id: i64,
    pub kaynak_kasa_ad: String,
    pub kaynak_kasa_para_birimi: String,
    pub hedef_kasa_id: i64,
    pub hedef_kasa_ad: String,
    pub hedef_kasa_para_birimi: String,
    pub tarih: chrono::NaiveDate,
    pub kaynak_miktar: f64,
    pub hedef_miktar: f64,
    pub kur: Option<f64>,
    pub aciklama: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct TransferInput {
    pub kaynak_kasa_id: i64,
    pub hedef_kasa_id: i64,
    pub tarih: chrono::NaiveDate,
    pub hedef_miktar: f64,
    pub kur: Option<f64>,
    pub aciklama: Option<String>,
}

// --- Router ---

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/", get(get_kasalar).post(create_kasa))
        .route("/transfer", post(kasa_transfer))
        .route("/{id}", get(get_kasa).put(update_kasa).delete(delete_kasa))
        .route("/{id}/hareketler", get(get_hareketler).post(create_hareket))
        .route("/{id}/hareketler/{hareket_id}", delete(delete_hareket))
        .route("/{id}/transferler", get(get_kasa_transferleri))
        .with_state(pool)
}

// --- Handler'lar ---

async fn get_kasalar(user: AuthUser, State(pool): State<PgPool>) -> AppResult<Json<Vec<Kasa>>> {
    user.require_izin(&pool, "kasa.goruntule").await?;
    let kasalar = sqlx::query_as::<_, Kasa>(
        "SELECT id, ad, para_birimi, bakiye, aciklama, aktif, created_at, updated_at
         FROM kasalar ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(kasalar))
}

async fn get_kasa(user: AuthUser, State(pool): State<PgPool>, Path(id): Path<i64>) -> AppResult<Json<Kasa>> {
    user.require_izin(&pool, "kasa.goruntule").await?;
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
    user: AuthUser,
    State(pool): State<PgPool>,
    Json(input): Json<CreateKasaInput>,
) -> AppResult<Json<Kasa>> {
    user.require_izin(&pool, "kasa.olustur").await?;
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
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateKasaInput>,
) -> AppResult<Json<Kasa>> {
    user.require_izin(&pool, "kasa.duzenle").await?;
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
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_izin(&pool, "kasa.sil").await?;
    sqlx::query("DELETE FROM kasalar WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(Json(serde_json::json!({ "mesaj": "Kasa silindi" })))
}

async fn get_hareketler(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(kasa_id): Path<i64>,
) -> AppResult<Json<Vec<KasaHareketi>>> {
    user.require_izin(&pool, "kasa.goruntule").await?;
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
    user: AuthUser,
    State(pool): State<PgPool>,
    Json(input): Json<CreateHareket>,
) -> AppResult<Json<KasaHareketi>> {
    user.require_izin(&pool, "kasa.hareket").await?;
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
    user: AuthUser,
    State(pool): State<PgPool>,
    Path((kasa_id, hareket_id)): Path<(i64, i64)>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_izin(&pool, "kasa.hareket").await?;
    sqlx::query("DELETE FROM kasa_hareketleri WHERE id = $1 AND kasa_id = $2")
        .bind(hareket_id)
        .bind(kasa_id)
        .execute(&pool)
        .await?;
    Ok(Json(serde_json::json!({ "mesaj": "Hareket silindi" })))
}

// --- Transfer ---

async fn get_kasa_transferleri(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(kasa_id): Path<i64>,
) -> AppResult<Json<Vec<KasaTransfer>>> {
    user.require_izin(&pool, "kasa.goruntule").await?;
    let transferler = sqlx::query_as::<_, KasaTransfer>(
        "SELECT t.id,
                t.kaynak_kasa_id, k1.ad AS kaynak_kasa_ad, k1.para_birimi AS kaynak_kasa_para_birimi,
                t.hedef_kasa_id,  k2.ad AS hedef_kasa_ad,  k2.para_birimi AS hedef_kasa_para_birimi,
                t.tarih, t.kaynak_miktar, t.hedef_miktar, t.kur, t.aciklama, t.created_at
         FROM kasa_transferleri t
         JOIN kasalar k1 ON k1.id = t.kaynak_kasa_id
         JOIN kasalar k2 ON k2.id = t.hedef_kasa_id
         WHERE t.kaynak_kasa_id = $1 OR t.hedef_kasa_id = $1
         ORDER BY t.tarih DESC, t.id DESC",
    )
    .bind(kasa_id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(transferler))
}

async fn kasa_transfer(
    user: AuthUser,
    State(pool): State<PgPool>,
    Json(input): Json<TransferInput>,
) -> AppResult<Json<KasaTransfer>> {
    user.require_izin(&pool, "kasa.transfer").await?;

    if input.kaynak_kasa_id == input.hedef_kasa_id {
        return Err(crate::errors::AppError::BadRequest(
            "Kaynak ve hedef kasa ayni olamaz".into(),
        ));
    }
    if input.hedef_miktar <= 0.0 {
        return Err(crate::errors::AppError::BadRequest(
            "Transfer miktari sifirdan buyuk olmalidir".into(),
        ));
    }

    let mut tx = pool.begin().await?;

    // Kaynak + hedef kasa bilgisi
    let (kaynak_bakiye, kaynak_para_birimi): (f64, String) = sqlx::query_as(
        "SELECT bakiye, para_birimi FROM kasalar WHERE id = $1",
    )
    .bind(input.kaynak_kasa_id)
    .fetch_one(&mut *tx)
    .await?;

    let (_hedef_bakiye, hedef_para_birimi): (f64, String) = sqlx::query_as(
        "SELECT bakiye, para_birimi FROM kasalar WHERE id = $1",
    )
    .bind(input.hedef_kasa_id)
    .fetch_one(&mut *tx)
    .await?;

    // Kaynak miktari hesapla
    let kaynak_miktar = if kaynak_para_birimi == hedef_para_birimi {
        input.hedef_miktar
    } else {
        let kur = input.kur.ok_or_else(|| {
            crate::errors::AppError::BadRequest(
                "Farkli para birimleri icin kur girilmelidir".into(),
            )
        })?;
        if kur <= 0.0 {
            return Err(crate::errors::AppError::BadRequest(
                "Kur sifirdan buyuk olmalidir".into(),
            ));
        }
        input.hedef_miktar * kur
    };

    if kaynak_bakiye < kaynak_miktar {
        return Err(crate::errors::AppError::BadRequest(format!(
            "Yetersiz bakiye. Mevcut: {:.4}, Gereken: {:.4}",
            kaynak_bakiye, kaynak_miktar
        )));
    }

    let hedef_aciklama = input
        .aciklama
        .clone()
        .unwrap_or_else(|| "Transfer".to_string());

    // Kaynak hareket
    let kaynak_son_bakiye: f64 = sqlx::query_scalar::<_, Option<f64>>(
        "SELECT bakiye FROM kasa_hareketleri
         WHERE kasa_id = $1 ORDER BY tarih DESC, id DESC LIMIT 1",
    )
    .bind(input.kaynak_kasa_id)
    .fetch_optional(&mut *tx)
    .await?
    .flatten()
    .unwrap_or(0.0);
    let kaynak_yeni_bakiye = kaynak_son_bakiye - kaynak_miktar;
    let kaynak_aciklama = format!("Transfer -> {} [{}]", hedef_para_birimi, hedef_aciklama);

    sqlx::query(
        "INSERT INTO kasa_hareketleri (kasa_id, tarih, aciklama, giren, cikan, bakiye)
         VALUES ($1, $2, $3, 0.0, $4, $5)",
    )
    .bind(input.kaynak_kasa_id)
    .bind(input.tarih)
    .bind(&kaynak_aciklama)
    .bind(kaynak_miktar)
    .bind(kaynak_yeni_bakiye)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE kasalar SET bakiye = $1, updated_at = NOW() WHERE id = $2")
        .bind(kaynak_yeni_bakiye)
        .bind(input.kaynak_kasa_id)
        .execute(&mut *tx)
        .await?;

    // Hedef hareket
    let hedef_son_bakiye: f64 = sqlx::query_scalar::<_, Option<f64>>(
        "SELECT bakiye FROM kasa_hareketleri
         WHERE kasa_id = $1 ORDER BY tarih DESC, id DESC LIMIT 1",
    )
    .bind(input.hedef_kasa_id)
    .fetch_optional(&mut *tx)
    .await?
    .flatten()
    .unwrap_or(0.0);
    let hedef_yeni_bakiye = hedef_son_bakiye + input.hedef_miktar;
    let hedef_hareket_aciklama =
        format!("Transfer <- {} [{}]", kaynak_para_birimi, hedef_aciklama);

    sqlx::query(
        "INSERT INTO kasa_hareketleri (kasa_id, tarih, aciklama, giren, cikan, bakiye)
         VALUES ($1, $2, $3, $4, 0.0, $5)",
    )
    .bind(input.hedef_kasa_id)
    .bind(input.tarih)
    .bind(&hedef_hareket_aciklama)
    .bind(input.hedef_miktar)
    .bind(hedef_yeni_bakiye)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE kasalar SET bakiye = $1, updated_at = NOW() WHERE id = $2")
        .bind(hedef_yeni_bakiye)
        .bind(input.hedef_kasa_id)
        .execute(&mut *tx)
        .await?;

    // Transfer kaydi
    let transfer_id: i64 = sqlx::query_scalar(
        "INSERT INTO kasa_transferleri
            (kaynak_kasa_id, hedef_kasa_id, tarih, kaynak_miktar, hedef_miktar, kur, aciklama)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id",
    )
    .bind(input.kaynak_kasa_id)
    .bind(input.hedef_kasa_id)
    .bind(input.tarih)
    .bind(kaynak_miktar)
    .bind(input.hedef_miktar)
    .bind(input.kur)
    .bind(&input.aciklama)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    let transfer = sqlx::query_as::<_, KasaTransfer>(
        "SELECT t.id,
                t.kaynak_kasa_id, k1.ad AS kaynak_kasa_ad, k1.para_birimi AS kaynak_kasa_para_birimi,
                t.hedef_kasa_id,  k2.ad AS hedef_kasa_ad,  k2.para_birimi AS hedef_kasa_para_birimi,
                t.tarih, t.kaynak_miktar, t.hedef_miktar, t.kur, t.aciklama, t.created_at
         FROM kasa_transferleri t
         JOIN kasalar k1 ON k1.id = t.kaynak_kasa_id
         JOIN kasalar k2 ON k2.id = t.hedef_kasa_id
         WHERE t.id = $1",
    )
    .bind(transfer_id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(transfer))
}
