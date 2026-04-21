use axum::{
    extract::{Path, State},
    routing::{get, post, delete},
    Json, Router,
};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use crate::auth::AuthUser;
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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct HisseSatis {
    pub id: i64,
    pub hisse_id: i64,
    pub hissedar_id: i64,
    pub hissedar_adi: Option<String>,
    pub kasa_id: i64,
    pub satis_tutari: f64,
    pub tarih: chrono::NaiveDate,
    pub tamamlandi: bool,
    pub tamamlanma_tarihi: Option<chrono::NaiveDate>,
    pub iptal: bool,
    pub aciklama: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct HisseSatisOdeme {
    pub id: i64,
    pub satis_id: i64,
    pub tutar: f64,
    pub tarih: chrono::NaiveDate,
    pub aciklama: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct HisseBorc {
    pub id: i64,
    pub donem_id: i64,
    pub hissedar_id: i64,
    pub hisse_sayisi: i32,
    pub tutar: f64,
    pub odendi: bool,
    pub odeme_tarihi: Option<chrono::NaiveDate>,
    pub aciklama: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateHisseInput {
    pub aciklama: Option<String>,
    pub atama_hissedar_id: Option<i64>,
    pub atama_tarih: Option<chrono::NaiveDate>,
    pub atama_ucret: Option<f64>,
    pub atama_aciklama: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateHisseTopluInput {
    pub adet: i32,
    pub aciklama: Option<String>,
    pub atama_hissedar_id: Option<i64>,
    pub atama_tarih: Option<chrono::NaiveDate>,
    pub atama_ucret: Option<f64>,
    pub atama_aciklama: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AtamaInput {
    pub hisse_id: i64,
    pub hissedar_id: i64,
    pub tarih: chrono::NaiveDate,
    pub ucret: f64,
    pub aciklama: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TransferInput {
    pub hisse_id: i64,
    pub yeni_hissedar_id: i64,
    pub tarih: chrono::NaiveDate,
    pub ucret: f64,
    pub aciklama: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SatisBaslatInput {
    pub hisse_id: i64,
    pub hissedar_id: i64,
    pub kasa_id: i64,
    pub satis_tutari: f64,
    pub tarih: chrono::NaiveDate,
    pub aciklama: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SatisOdemeInput {
    pub satis_id: i64,
    pub tutar: f64,
    pub tarih: chrono::NaiveDate,
    pub aciklama: Option<String>,
}

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/", get(get_hisseler).post(create_hisse))
        .route("/toplu", post(create_toplu))
        .route("/ata", post(ata))
        .route("/transfer", post(transfer))
        .route("/satis", post(satis_baslat))
        .route("/satis/{satis_id}", delete(satis_iptal))
        .route("/satis/{satis_id}/odeme", post(satis_odeme_ekle))
        .route("/satis/{satis_id}/odemeler", get(satis_odemeleri))
        .route("/atama/{atama_id}", delete(atama_sil))
        .route("/{id}", get(get_hisse).delete(delete_hisse))
        .route("/{id}/atamalar", get(get_atamalari))
        .route("/{id}/borclar", get(get_hisse_borclari))
        .route("/{id}/satis", get(get_hisse_satis_aktif))
        .with_state(pool)
}

async fn get_hisseler(user: AuthUser, State(pool): State<PgPool>) -> AppResult<Json<Vec<Hisse>>> {
    user.require_izin(&pool, "hisse.goruntule").await?;
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

async fn get_hisse(user: AuthUser, State(pool): State<PgPool>, Path(id): Path<i64>) -> AppResult<Json<Hisse>> {
    user.require_izin(&pool, "hisse.goruntule").await?;
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
    user: AuthUser,
    State(pool): State<PgPool>,
    Json(input): Json<CreateHisseInput>,
) -> AppResult<Json<Hisse>> {
    user.require_izin(&pool, "hisse.yonet").await?;
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

    // Hissedar seçildiyse otomatik atama yap
    if let Some(hissedar_id) = input.atama_hissedar_id {
        let tarih = input.atama_tarih.unwrap_or_else(|| chrono::Utc::now().date_naive());
        let ucret = input.atama_ucret.unwrap_or(0.0);
        sqlx::query(
            "INSERT INTO hisse_atamalari (hisse_id, hissedar_id, tarih, ucret, aciklama)
             VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(hisse.id)
        .bind(hissedar_id)
        .bind(tarih)
        .bind(ucret)
        .bind(&input.atama_aciklama)
        .execute(&pool)
        .await?;
        sqlx::query("UPDATE hisseler SET durum = 'atanmis', updated_at = NOW() WHERE id = $1")
            .bind(hisse.id)
            .execute(&pool)
            .await?;
        // Atama bilgisiyle birlikte tekrar çek
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
        .bind(hisse.id)
        .fetch_one(&pool)
        .await?;
        return Ok(Json(hisse));
    }

    Ok(Json(hisse))
}

async fn create_toplu(
    user: AuthUser,
    State(pool): State<PgPool>,
    Json(input): Json<CreateHisseTopluInput>,
) -> AppResult<Json<Vec<Hisse>>> {
    user.require_izin(&pool, "hisse.yonet").await?;
    let mevcut: Option<i64> = sqlx::query_scalar("SELECT COUNT(*) FROM hisseler")
        .fetch_one(&pool)
        .await?;
    let baslangic = mevcut.unwrap_or(0);

    let mut hisseler = Vec::new();
    let mut olusturulan_idler: Vec<i64> = Vec::new();
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
        olusturulan_idler.push(h.id);
        hisseler.push(h);
    }

    // Hissedar seçildiyse tüm oluşturulan hisseleri ata
    if let Some(hissedar_id) = input.atama_hissedar_id {
        let tarih = input.atama_tarih.unwrap_or_else(|| chrono::Utc::now().date_naive());
        let ucret = input.atama_ucret.unwrap_or(0.0);
        for hid in &olusturulan_idler {
            sqlx::query(
                "INSERT INTO hisse_atamalari (hisse_id, hissedar_id, tarih, ucret, aciklama)
                 VALUES ($1, $2, $3, $4, $5)"
            )
            .bind(hid)
            .bind(hissedar_id)
            .bind(tarih)
            .bind(ucret)
            .bind(&input.atama_aciklama)
            .execute(&pool)
            .await?;
        }
        sqlx::query("UPDATE hisseler SET durum = 'atanmis', updated_at = NOW() WHERE id = ANY($1)")
            .bind(&olusturulan_idler)
            .execute(&pool)
            .await?;
        // Güncel bilgilerle tekrar çek
        let guncel = sqlx::query_as::<_, Hisse>(
            "SELECT h.id, h.kod, h.durum, h.aciklama,
                    a.hissedar_id,
                    (SELECT soyad || ' ' || ad FROM hissedarlar WHERE id = a.hissedar_id) AS hissedar_adi,
                    h.created_at, h.updated_at
             FROM hisseler h
             LEFT JOIN LATERAL (
                 SELECT hissedar_id FROM hisse_atamalari
                 WHERE hisse_id = h.id ORDER BY tarih DESC LIMIT 1
             ) a ON true
             WHERE h.id = ANY($1)
             ORDER BY h.id"
        )
        .bind(&olusturulan_idler)
        .fetch_all(&pool)
        .await?;
        return Ok(Json(guncel));
    }

    Ok(Json(hisseler))
}

async fn ata(
    user: AuthUser,
    State(pool): State<PgPool>,
    Json(input): Json<AtamaInput>,
) -> AppResult<Json<HisseAtama>> {
    user.require_izin(&pool, "hisse.yonet").await?;
    let atama = sqlx::query_as::<_, HisseAtama>(
        "INSERT INTO hisse_atamalari (hisse_id, hissedar_id, tarih, ucret, aciklama)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, hisse_id, hissedar_id,
                   (SELECT soyad || ' ' || ad FROM hissedarlar WHERE id = $2) AS hissedar_adi,
                   tarih, ucret, aciklama, created_at"
    )
    .bind(input.hisse_id)
    .bind(input.hissedar_id)
    .bind(input.tarih)
    .bind(input.ucret)
    .bind(input.aciklama)
    .fetch_one(&pool)
    .await?;

    sqlx::query("UPDATE hisseler SET durum = 'atanmis', updated_at = NOW() WHERE id = $1")
        .bind(input.hisse_id)
        .execute(&pool)
        .await?;

    Ok(Json(atama))
}

async fn atama_sil(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(atama_id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_izin(&pool, "hisse.yonet").await?;
    // Hisse'yi musait yap
    sqlx::query(
        "UPDATE hisseler SET durum = 'musait', updated_at = NOW()
         WHERE id = (SELECT hisse_id FROM hisse_atamalari WHERE id = $1)"
    )
    .bind(atama_id)
    .execute(&pool)
    .await?;
    sqlx::query("DELETE FROM hisse_atamalari WHERE id = $1")
        .bind(atama_id)
        .execute(&pool)
        .await?;
    Ok(Json(serde_json::json!({ "mesaj": "Atama silindi" })))
}

async fn transfer(
    user: AuthUser,
    State(pool): State<PgPool>,
    Json(input): Json<TransferInput>,
) -> AppResult<Json<HisseAtama>> {
    user.require_izin(&pool, "hisse.yonet").await?;
    // Eski atamayı sil
    sqlx::query("DELETE FROM hisse_atamalari WHERE hisse_id = $1")
        .bind(input.hisse_id)
        .execute(&pool)
        .await?;
    // Yeni atama ekle
    let atama = sqlx::query_as::<_, HisseAtama>(
        "INSERT INTO hisse_atamalari (hisse_id, hissedar_id, tarih, ucret, aciklama)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, hisse_id, hissedar_id,
                   (SELECT soyad || ' ' || ad FROM hissedarlar WHERE id = $2) AS hissedar_adi,
                   tarih, ucret, aciklama, created_at"
    )
    .bind(input.hisse_id)
    .bind(input.yeni_hissedar_id)
    .bind(input.tarih)
    .bind(input.ucret)
    .bind(input.aciklama)
    .fetch_one(&pool)
    .await?;
    Ok(Json(atama))
}

async fn satis_baslat(
    user: AuthUser,
    State(pool): State<PgPool>,
    Json(input): Json<SatisBaslatInput>,
) -> AppResult<Json<HisseSatis>> {
    user.require_izin(&pool, "hisse.yonet").await?;
    let satis = sqlx::query_as::<_, HisseSatis>(
        "INSERT INTO hisse_satislari (hisse_id, hissedar_id, kasa_id, satis_tutari, tarih, aciklama)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, hisse_id, hissedar_id,
                   (SELECT soyad || ' ' || ad FROM hissedarlar WHERE id = $2) AS hissedar_adi,
                   kasa_id, satis_tutari, tarih, tamamlandi, tamamlanma_tarihi, iptal, aciklama, created_at"
    )
    .bind(input.hisse_id)
    .bind(input.hissedar_id)
    .bind(input.kasa_id)
    .bind(input.satis_tutari)
    .bind(input.tarih)
    .bind(input.aciklama)
    .fetch_one(&pool)
    .await?;
    Ok(Json(satis))
}

async fn satis_odeme_ekle(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(satis_id): Path<i64>,
    Json(input): Json<SatisOdemeInput>,
) -> AppResult<Json<HisseSatisOdeme>> {
    user.require_izin(&pool, "hisse.yonet").await?;
    let odeme = sqlx::query_as::<_, HisseSatisOdeme>(
        "INSERT INTO hisse_satis_odemeleri (satis_id, tutar, tarih, aciklama)
         VALUES ($1, $2, $3, $4)
         RETURNING id, satis_id, tutar, tarih, aciklama, created_at"
    )
    .bind(satis_id)
    .bind(input.tutar)
    .bind(input.tarih)
    .bind(input.aciklama)
    .fetch_one(&pool)
    .await?;
    // Toplam ödeme = satış tutarı ise tamamlandı işaretle
    sqlx::query(
        "UPDATE hisse_satislari SET
             tamamlandi = (SELECT COALESCE(SUM(tutar),0) FROM hisse_satis_odemeleri WHERE satis_id=$1) >= satis_tutari,
             tamamlanma_tarihi = CASE WHEN (SELECT COALESCE(SUM(tutar),0) FROM hisse_satis_odemeleri WHERE satis_id=$1) >= satis_tutari THEN CURRENT_DATE ELSE NULL END
         WHERE id = $1"
    )
    .bind(satis_id)
    .execute(&pool)
    .await?;
    Ok(Json(odeme))
}

async fn satis_odemeleri(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(satis_id): Path<i64>,
) -> AppResult<Json<Vec<HisseSatisOdeme>>> {
    user.require_izin(&pool, "hisse.goruntule").await?;
    let liste = sqlx::query_as::<_, HisseSatisOdeme>(
        "SELECT id, satis_id, tutar, tarih, aciklama, created_at
         FROM hisse_satis_odemeleri WHERE satis_id = $1 ORDER BY tarih"
    )
    .bind(satis_id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(liste))
}

async fn satis_iptal(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(satis_id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_izin(&pool, "hisse.yonet").await?;
    sqlx::query("UPDATE hisse_satislari SET iptal = true WHERE id = $1")
        .bind(satis_id)
        .execute(&pool)
        .await?;
    Ok(Json(serde_json::json!({ "mesaj": "Satış iptal edildi" })))
}

async fn get_hisse_satis_aktif(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(hisse_id): Path<i64>,
) -> AppResult<Json<Option<HisseSatis>>> {
    user.require_izin(&pool, "hisse.goruntule").await?;
    let satis = sqlx::query_as::<_, HisseSatis>(
        "SELECT s.id, s.hisse_id, s.hissedar_id,
                (h.soyad || ' ' || h.ad) AS hissedar_adi,
                s.kasa_id, s.satis_tutari, s.tarih, s.tamamlandi,
                s.tamamlanma_tarihi, s.iptal, s.aciklama, s.created_at
         FROM hisse_satislari s
         JOIN hissedarlar h ON h.id = s.hissedar_id
         WHERE s.hisse_id = $1 AND s.iptal = false
         ORDER BY s.created_at DESC LIMIT 1"
    )
    .bind(hisse_id)
    .fetch_optional(&pool)
    .await?;
    Ok(Json(satis))
}

async fn get_hisse_borclari(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(hisse_id): Path<i64>,
) -> AppResult<Json<Vec<HisseBorc>>> {
    user.require_izin(&pool, "hisse.goruntule").await?;
    // Hisseye atanmış hissedarın borçlarını getir
    let liste = sqlx::query_as::<_, HisseBorc>(
        "SELECT b.id, b.donem_id, b.hissedar_id, b.hisse_sayisi, b.tutar,
                b.odendi, b.odeme_tarihi, b.aciklama, b.created_at
         FROM donem_aidat_borclari b
         WHERE b.hissedar_id = (
             SELECT hissedar_id FROM hisse_atamalari
             WHERE hisse_id = $1 ORDER BY tarih DESC LIMIT 1
         )
         ORDER BY b.created_at DESC"
    )
    .bind(hisse_id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(liste))
}

async fn get_atamalari(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(hisse_id): Path<i64>,
) -> AppResult<Json<Vec<HisseAtama>>> {
    user.require_izin(&pool, "hisse.goruntule").await?;
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
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_izin(&pool, "hisse.yonet").await?;
    sqlx::query("DELETE FROM hisseler WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(Json(serde_json::json!({ "mesaj": "Hisse silindi" })))
}


