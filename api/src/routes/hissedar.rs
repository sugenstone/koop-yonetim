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

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CuzdanHareketi {
    pub id: i64,
    pub hissedar_id: i64,
    pub donem_id: Option<i64>,
    pub donem_adi: Option<String>,
    pub tarih: chrono::NaiveDate,
    pub bilgi: String,
    pub borc: f64,
    pub alacak: f64,
    pub bakiye: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ParaEkleInput {
    pub hissedar_id: i64,
    pub tutar: f64,
    pub aciklama: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ParaEkleSonuc {
    pub yeni_bakiye: f64,
    pub tahsil_edilen_borc_sayisi: i64,
    pub tahsil_edilen_toplam: f64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct HisseAtamaOzet {
    pub id: i64,
    pub hisse_id: i64,
    pub hisse_kod: Option<String>,
    pub tarih: chrono::NaiveDate,
    pub ucret: f64,
    pub aciklama: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct HissedarBorc {
    pub id: i64,
    pub donem_id: i64,
    pub hisse_sayisi: i32,
    pub tutar: f64,
    pub odendi: bool,
    pub odeme_tarihi: Option<chrono::NaiveDate>,
    pub aciklama: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/", get(get_hissedarlar).post(create_hissedar))
        .route("/{id}", get(get_hissedar).put(update_hissedar).delete(delete_hissedar))
        .route("/{id}/cuzdan", get(get_cuzdan))
        .route("/{id}/cuzdan/para", post(cuzdan_para_ekle))
        .route("/{id}/atamalar", get(get_hissedar_atamalari))
        .route("/{id}/borclar", get(get_hissedar_borclari))
        .with_state(pool)
}

async fn get_hissedarlar(user: AuthUser, State(pool): State<PgPool>) -> AppResult<Json<Vec<Hissedar>>> {
    user.require_izin(&pool, "hissedar.goruntule").await?;
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
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<Hissedar>> {
    user.require_izin(&pool, "hissedar.goruntule").await?;
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
    user: AuthUser,
    State(pool): State<PgPool>,
    Json(input): Json<CreateHissedarInput>,
) -> AppResult<Json<Hissedar>> {
    user.require_izin(&pool, "hissedar.olustur").await?;
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
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateHissedarInput>,
) -> AppResult<Json<Hissedar>> {
    user.require_izin(&pool, "hissedar.duzenle").await?;
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
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_izin(&pool, "hissedar.sil").await?;
    sqlx::query("DELETE FROM hissedarlar WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(Json(serde_json::json!({ "mesaj": "Hissedar silindi" })))
}

async fn get_cuzdan(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<Vec<CuzdanHareketi>>> {
    user.require_izin(&pool, "hissedar.goruntule").await?;
    let liste = sqlx::query_as::<_, CuzdanHareketi>(
        "SELECT c.id, c.hissedar_id, c.donem_id,
                CASE WHEN d.id IS NOT NULL THEN CAST(d.ay AS TEXT) || '/' || CAST(d.yil AS TEXT) ELSE NULL END AS donem_adi,
                c.tarih, c.bilgi, c.borc, c.alacak, c.bakiye, c.created_at
         FROM hissedar_cuzdanlari c
         LEFT JOIN donemler d ON d.id = c.donem_id
         WHERE c.hissedar_id = $1
         ORDER BY c.id DESC"
    )
    .bind(id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(liste))
}

async fn cuzdan_para_ekle(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(input): Json<ParaEkleInput>,
) -> AppResult<Json<ParaEkleSonuc>> {
    user.require_izin(&pool, "hissedar.cuzdan").await?;

    let mut tx = pool.begin().await?;

    // Mevcut bakiyeyi hesapla
    let mevcut_bakiye: f64 = sqlx::query_scalar(
        "SELECT COALESCE(bakiye, 0) FROM hissedar_cuzdanlari
          WHERE hissedar_id = $1 ORDER BY id DESC LIMIT 1"
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .unwrap_or(0.0);

    let yeni_bakiye_sonrasi_yatirma = mevcut_bakiye + input.tutar;
    let bilgi = input.aciklama.as_deref().unwrap_or("Para yatırma");

    // Yatırma kaydı ekle
    sqlx::query(
        "INSERT INTO hissedar_cuzdanlari (hissedar_id, tarih, bilgi, borc, alacak, bakiye)
          VALUES ($1, CURRENT_DATE, $2, 0, $3, $4)"
    )
    .bind(id)
    .bind(bilgi)
    .bind(input.tutar)
    .bind(yeni_bakiye_sonrasi_yatirma)
    .execute(&mut *tx)
    .await?;

    // Ödenmemiş aidat borçlarını sıraya al
    #[derive(sqlx::FromRow)]
    struct AidatBorc { id: i64, tutar: f64, donem_id: Option<i64> }
    let borclar = sqlx::query_as::<_, AidatBorc>(
        "SELECT id, tutar, donem_id FROM donem_aidat_borclari
          WHERE hissedar_id = $1 AND odendi = false
          ORDER BY created_at ASC"
    )
    .bind(id)
    .fetch_all(&mut *tx)
    .await?;

    let mut kalan = yeni_bakiye_sonrasi_yatirma;
    let mut tahsil_sayisi: i64 = 0;
    let mut tahsil_toplam: f64 = 0.0;

    for borc in &borclar {
        if kalan < borc.tutar {
            break;
        }
        kalan -= borc.tutar;
        tahsil_sayisi += 1;
        tahsil_toplam += borc.tutar;

        // Borç kaydını ödenmiş yap
        sqlx::query(
            "UPDATE donem_aidat_borclari SET odendi = true, odeme_tarihi = CURRENT_DATE WHERE id = $1"
        )
        .bind(borc.id)
        .execute(&mut *tx)
        .await?;

        // Cüzdana borç hareketi ekle
        let bilgi_borc = format!("Aidat tahsilat — dönem #{}", borc.donem_id.map(|d| d.to_string()).unwrap_or_default());
        sqlx::query(
            "INSERT INTO hissedar_cuzdanlari (hissedar_id, donem_id, tarih, bilgi, borc, alacak, bakiye)
              VALUES ($1, $2, CURRENT_DATE, $3, $4, 0, $5)"
        )
        .bind(id)
        .bind(borc.donem_id)
        .bind(bilgi_borc)
        .bind(borc.tutar)
        .bind(kalan)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(Json(ParaEkleSonuc {
        yeni_bakiye: kalan,
        tahsil_edilen_borc_sayisi: tahsil_sayisi,
        tahsil_edilen_toplam: tahsil_toplam,
    }))
}

async fn get_hissedar_atamalari(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(hissedar_id): Path<i64>,
) -> AppResult<Json<Vec<HisseAtamaOzet>>> {
    user.require_izin(&pool, "hissedar.goruntule").await?;
    let liste = sqlx::query_as::<_, HisseAtamaOzet>(
        "SELECT a.id, a.hisse_id, hs.kod AS hisse_kod,
                a.tarih, a.ucret, a.aciklama, a.created_at
         FROM hisse_atamalari a
         JOIN hisseler hs ON hs.id = a.hisse_id
         WHERE a.hissedar_id = $1
         ORDER BY a.tarih DESC"
    )
    .bind(hissedar_id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(liste))
}

async fn get_hissedar_borclari(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(hissedar_id): Path<i64>,
) -> AppResult<Json<Vec<HissedarBorc>>> {
    user.require_izin(&pool, "hissedar.goruntule").await?;
    let liste = sqlx::query_as::<_, HissedarBorc>(
        "SELECT id, donem_id, hisse_sayisi, tutar,
                odendi, odeme_tarihi, aciklama, created_at
         FROM donem_aidat_borclari
         WHERE hissedar_id = $1
         ORDER BY created_at DESC"
    )
    .bind(hissedar_id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(liste))
}
