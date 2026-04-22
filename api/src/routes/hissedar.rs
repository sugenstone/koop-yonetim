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
    pub operation_id: Option<uuid::Uuid>,
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
        .route("/{id}/cuzdan/para-geri-cek", post(cuzdan_para_geri_cek))
        .route("/{id}/cuzdan/tahsilat-iptal/{hareket_id}", post(cuzdan_tahsilat_iptal))
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
                c.tarih, c.bilgi, c.borc, c.alacak, c.bakiye, c.operation_id, c.created_at
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
    use super::common::{
        cuzdan_alacak_ekle_op, cuzdan_son_bakiye, hissedar_bilgi, kasa_giren_ekle_op,
        tahsilat_aciklamasi, donem_adi,
    };
    user.require_izin(&pool, "hissedar.cuzdan").await?;

    if input.tutar <= 0.0 {
        return Err(crate::errors::AppError::BadRequest(
            "Tutar pozitif olmalı".into()
        ));
    }

    let h = hissedar_bilgi(&pool, id).await?;
    let tarih = chrono::Utc::now().date_naive();
    let op_id = uuid::Uuid::new_v4();

    // 1) Cüzdana alacak (para yatırma)
    let yatirma_ac = input.aciklama.clone().unwrap_or_else(|| "Para yatırma".to_string());
    cuzdan_alacak_ekle_op(&pool, id, None, tarih, &yatirma_ac, input.tutar, op_id).await?;

    // 2) Ödenmemiş aidat borçlarını sıralı getir
    #[derive(sqlx::FromRow)]
    struct AidatBorc {
        id: i64,
        hisse_sayisi: i32,
        tutar: f64,
        ay: i32,
        yil: i32,
    }
    let borclar: Vec<AidatBorc> = sqlx::query_as(
        "SELECT b.id, b.hisse_sayisi, b.tutar, d.ay, d.yil
         FROM donem_aidat_borclari b
         JOIN donemler d ON d.id = b.donem_id
         WHERE b.hissedar_id = $1 AND b.odendi = false
         ORDER BY d.yil ASC, d.ay ASC, b.id ASC"
    )
    .bind(id)
    .fetch_all(&pool)
    .await?;

    // Harcanabilir nakit = cüzdan_bakiye + ödenmemiş IOU toplamı
    let unpaid_iou_sum: f64 = borclar.iter().map(|b| b.tutar).sum();
    let bakiye_after_deposit = cuzdan_son_bakiye(&pool, id).await?;
    let mut nakit_var: f64 = bakiye_after_deposit + unpaid_iou_sum;
    const EPS: f64 = 0.005;

    let mut tahsil_sayisi: i64 = 0;
    let mut tahsil_toplam: f64 = 0.0;

    for borc in &borclar {
        if nakit_var + EPS < borc.tutar { break; }

        // Borcu ödenmiş işaretle + operation_id damgala
        sqlx::query(
            "UPDATE donem_aidat_borclari
             SET odendi = true, odeme_tarihi = $1, odeme_operation_id = $2
             WHERE id = $3"
        )
        .bind(tarih)
        .bind(op_id)
        .bind(borc.id)
        .execute(&pool)
        .await?;

        let d_ad = donem_adi(borc.ay, borc.yil);
        let kasa_ac = tahsilat_aciklamasi(
            &d_ad,
            borc.hisse_sayisi as i64,
            &h.ad,
            &h.soyad,
            &h.yakin_adi,
            &h.yakinlik_derecesi,
        );
        kasa_giren_ekle_op(&pool, h.kasa_id, tarih, &kasa_ac, borc.tutar, op_id).await?;

        nakit_var -= borc.tutar;
        tahsil_sayisi += 1;
        tahsil_toplam += borc.tutar;
    }

    // 3) Hisse satın alma borçları
    let hisse_borc_toplam: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(borc), 0)::double precision FROM hissedar_cuzdanlari
         WHERE hissedar_id = $1 AND borc > 0 AND bilgi LIKE 'Hisse satın alma:%'"
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap_or(0.0);
    let hisse_tahsil_toplam: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(alacak), 0)::double precision FROM hissedar_cuzdanlari
         WHERE hissedar_id = $1 AND alacak > 0 AND bilgi LIKE 'Hisse satın alma tahsilatı:%'"
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap_or(0.0);
    let odenmemis_hisse_borc = hisse_borc_toplam - hisse_tahsil_toplam;

    if odenmemis_hisse_borc > EPS && nakit_var + EPS >= odenmemis_hisse_borc {
        let kasa_ac = format!("Hisse satın alma tahsilatı: {} {}", h.ad, h.soyad);
        kasa_giren_ekle_op(&pool, h.kasa_id, tarih, &kasa_ac, odenmemis_hisse_borc, op_id).await?;
        cuzdan_alacak_ekle_op(&pool, id, None, tarih, "Hisse satın alma tahsilatı", odenmemis_hisse_borc, op_id).await?;
        tahsil_toplam += odenmemis_hisse_borc;
    }

    let yeni_bakiye = cuzdan_son_bakiye(&pool, id).await?;

    Ok(Json(ParaEkleSonuc {
        yeni_bakiye,
        tahsil_edilen_borc_sayisi: tahsil_sayisi,
        tahsil_edilen_toplam: tahsil_toplam,
    }))
}

#[derive(Debug, Deserialize)]
pub struct ParaGeriCekInput {
    pub tutar: f64,
    pub aciklama: Option<String>,
}

async fn cuzdan_para_geri_cek(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(input): Json<ParaGeriCekInput>,
) -> AppResult<Json<serde_json::Value>> {
    use super::common::{
        cuzdan_borc_ekle, cuzdan_son_bakiye, hissedar_bilgi, kasa_cikan_ekle, kasa_son_bakiye,
    };
    user.require_izin(&pool, "hissedar.cuzdan").await?;

    if input.tutar <= 0.0 {
        return Err(crate::errors::AppError::BadRequest("Tutar pozitif olmalı".into()));
    }

    let bakiye = cuzdan_son_bakiye(&pool, id).await?;
    if bakiye < input.tutar {
        return Err(crate::errors::AppError::BadRequest(format!(
            "Cüzdan bakiyesi yetersiz (mevcut: {:.2})", bakiye
        )));
    }

    let h = hissedar_bilgi(&pool, id).await?;
    let kasa_bak = kasa_son_bakiye(&pool, h.kasa_id).await?;
    if kasa_bak < input.tutar {
        return Err(crate::errors::AppError::BadRequest(format!(
            "Kasa bakiyesi yetersiz (mevcut: {:.2})", kasa_bak
        )));
    }

    let tarih = chrono::Utc::now().date_naive();
    let bilgi = input.aciklama.clone().unwrap_or_else(|| "Para geri çekme".to_string());

    // Cüzdana borç (alacak azalır)
    cuzdan_borc_ekle(&pool, id, None, tarih, &bilgi, input.tutar).await?;

    // Kasadan çıkış
    let kasa_ac = format!("Hissedar para çekme: {} {}", h.ad, h.soyad);
    kasa_cikan_ekle(&pool, h.kasa_id, tarih, &kasa_ac, input.tutar).await?;

    let yeni_bakiye = cuzdan_son_bakiye(&pool, id).await?;
    Ok(Json(serde_json::json!({
        "mesaj": "Para çekildi",
        "yeni_bakiye": yeni_bakiye
    })))
}

/// Tahsilat iptali: bir cüzdan hareketinin operation_id'sine bağlı tüm
/// yan etkileri (cüzdan, kasa, dönem borç ödemeleri) geri alır.
async fn cuzdan_tahsilat_iptal(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path((hissedar_id, hareket_id)): Path<(i64, i64)>,
) -> AppResult<Json<serde_json::Value>> {
    use super::common::{recompute_cuzdan_bakiyeleri, recompute_kasa_bakiyeleri, hissedar_bilgi};

    user.require_izin(&pool, "hissedar.cuzdan.iptal").await?;

    // Hareketi bul
    #[derive(sqlx::FromRow)]
    struct Hareket {
        id: i64,
        hissedar_id: i64,
        operation_id: Option<uuid::Uuid>,
        alacak: f64,
        borc: f64,
    }
    let hareket: Option<Hareket> = sqlx::query_as(
        "SELECT id, hissedar_id, operation_id, alacak, borc
         FROM hissedar_cuzdanlari WHERE id = $1"
    )
    .bind(hareket_id)
    .fetch_optional(&pool)
    .await?;

    let hareket = hareket.ok_or_else(|| {
        crate::errors::AppError::BadRequest("Cüzdan hareketi bulunamadı".into())
    })?;

    if hareket.hissedar_id != hissedar_id {
        return Err(crate::errors::AppError::BadRequest(
            "Hareket bu hissedara ait değil".into()
        ));
    }

    let op_id = hareket.operation_id.ok_or_else(|| {
        crate::errors::AppError::BadRequest(
            "Bu kayıt eski bir işlemdir ve iptal edilemez (operation_id yok)".into()
        )
    })?;

    // Güvenlik: sadece "alacak" (para yatırma) hareketlerine izin ver
    if hareket.alacak <= 0.0 || hareket.borc > 0.0 {
        return Err(crate::errors::AppError::BadRequest(
            "Sadece para yatırma (tahsilat) kayıtları iptal edilebilir".into()
        ));
    }

    let h = hissedar_bilgi(&pool, hissedar_id).await?;

    // Etkilenen kasaları topla (recompute için)
    let etkilenen_kasalar: Vec<i64> = sqlx::query_scalar(
        "SELECT DISTINCT kasa_id FROM kasa_hareketleri WHERE operation_id = $1"
    )
    .bind(op_id)
    .fetch_all(&pool)
    .await?;

    // 1) Borçları geri aç
    let geri_acilan_borc: i64 = sqlx::query_scalar(
        "WITH upd AS (
            UPDATE donem_aidat_borclari
            SET odendi = false, odeme_tarihi = NULL, odeme_operation_id = NULL
            WHERE odeme_operation_id = $1
            RETURNING 1
        )
        SELECT COUNT(*) FROM upd"
    )
    .bind(op_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    // 2) Kasa hareketlerini sil
    let silinen_kasa = sqlx::query(
        "DELETE FROM kasa_hareketleri WHERE operation_id = $1"
    )
    .bind(op_id)
    .execute(&pool)
    .await?
    .rows_affected();

    // 3) Cüzdan hareketlerini sil
    let silinen_cuzdan = sqlx::query(
        "DELETE FROM hissedar_cuzdanlari WHERE operation_id = $1"
    )
    .bind(op_id)
    .execute(&pool)
    .await?
    .rows_affected();

    // 4) Cüzdan bakiyesini yeniden hesapla
    recompute_cuzdan_bakiyeleri(&pool, hissedar_id).await?;

    // 5) Etkilenen kasaların bakiyelerini yeniden hesapla
    for k_id in &etkilenen_kasalar {
        recompute_kasa_bakiyeleri(&pool, *k_id).await?;
    }

    let _ = (user, h); // unused warning susturmak için

    Ok(Json(serde_json::json!({
        "mesaj": "Tahsilat iptal edildi",
        "silinen_cuzdan_kayit": silinen_cuzdan,
        "silinen_kasa_kayit": silinen_kasa,
        "geri_acilan_borc": geri_acilan_borc,
    })))
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
