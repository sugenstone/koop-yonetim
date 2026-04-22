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
    pub kasa_ad: Option<String>,
    pub kasa_para_birimi: Option<String>,
    pub kategori_id: i64,
    pub kategori_ad: Option<String>,
    pub kategori_tip: Option<String>,
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
pub struct UpdateKategoriInput {
    pub ad: String,
    pub tip: Option<String>,
    pub aciklama: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateKayitInput {
    pub kasa_id: i64,
    pub kategori_id: i64,
    pub tarih: chrono::NaiveDate,
    pub tutar: f64,
    pub aciklama: String,
    /// Kullanicinin bilerek onayladigi para birimi. Kasa ile uyusmazsa istek reddedilir.
    pub para_birimi: String,
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
    Json(input): Json<UpdateKategoriInput>,
) -> AppResult<Json<GelirGiderKategori>> {
    user.require_izin(&pool, "gelir_gider.yonet").await?;
    // tip gonderilmezse mevcut deger korunur (tarihi bozmamak icin)
    let k = sqlx::query_as::<_, GelirGiderKategori>(
        "UPDATE gelir_gider_kategorileri
            SET ad = $1,
                tip = COALESCE($2, tip),
                aciklama = $3,
                updated_at = NOW()
          WHERE id = $4
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
        "SELECT g.id, g.kasa_id, ks.ad AS kasa_ad, ks.para_birimi AS kasa_para_birimi,
                g.kategori_id, k.ad AS kategori_ad, k.tip AS kategori_tip,
                g.tarih, g.tutar, g.aciklama, g.kasa_hareketi_id, g.created_at
         FROM gelir_gider_kayitlari g
         LEFT JOIN gelir_gider_kategorileri k ON k.id = g.kategori_id
         LEFT JOIN kasalar ks ON ks.id = g.kasa_id
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

    // Tutar pozitif olmali
    if !(input.tutar > 0.0) {
        return Err(crate::errors::AppError::BadRequest(
            "Tutar sifirdan buyuk olmalidir".to_string(),
        ));
    }

    let mut tx = pool.begin().await?;

    // Kasa para birimini getir ve kullanicinin secimi ile karsilastir
    let kasa_para_birimi: Option<String> = sqlx::query_scalar(
        "SELECT para_birimi FROM kasalar WHERE id = $1"
    )
    .bind(input.kasa_id)
    .fetch_optional(&mut *tx)
    .await?;
    let kasa_pb = kasa_para_birimi.ok_or_else(|| {
        crate::errors::AppError::BadRequest("Kasa bulunamadi".to_string())
    })?;
    let secilen_pb = input.para_birimi.trim().to_uppercase();
    if secilen_pb.is_empty() {
        return Err(crate::errors::AppError::BadRequest(
            "Para birimi secilmelidir".to_string(),
        ));
    }
    if secilen_pb != kasa_pb.to_uppercase() {
        return Err(crate::errors::AppError::BadRequest(format!(
            "Para birimi uyusmazligi: secilen kasa '{}' biriminde, gonderilen '{}'. Ayni birimde bir kasa seciniz veya tutari ilgili birime cevirin.",
            kasa_pb, secilen_pb
        )));
    }

    // Kategori tipini ve adini ogren (gelir/gider + aciklama icin ad)
    let kategori: Option<(String, String)> = sqlx::query_as(
        "SELECT tip, ad FROM gelir_gider_kategorileri WHERE id = $1"
    )
    .bind(input.kategori_id)
    .fetch_optional(&mut *tx)
    .await?;

    let (tip, kategori_ad): (Option<String>, Option<String>) = match kategori {
        Some((t, a)) => (Some(t), Some(a)),
        None => (None, None),
    };

    let (giren, cikan) = match tip.as_deref() {
        Some("gelir") => (input.tutar, 0.0_f64),
        _             => (0.0_f64, input.tutar),
    };

    // Kasa hareketi aciklamasi: "[Kategori] kullanici aciklamasi"
    let hareket_aciklama = match &kategori_ad {
        Some(ad) if !input.aciklama.trim().is_empty() => format!("[{}] {}", ad, input.aciklama),
        Some(ad) => format!("[{}]", ad),
        None => input.aciklama.clone(),
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
    .bind(hareket_aciklama)
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

    // Kayit ekle + kasa/kategori bilgilerini JOIN ile geri dondur
    let kayit = sqlx::query_as::<_, GelirGiderKayit>(
        "WITH ins AS (
            INSERT INTO gelir_gider_kayitlari
                (kasa_id, kategori_id, tarih, tutar, aciklama, kasa_hareketi_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, kasa_id, kategori_id, tarih, tutar, aciklama,
                      kasa_hareketi_id, created_at
         )
         SELECT ins.id, ins.kasa_id, ks.ad AS kasa_ad, ks.para_birimi AS kasa_para_birimi,
                ins.kategori_id, kt.ad AS kategori_ad, kt.tip AS kategori_tip,
                ins.tarih, ins.tutar, ins.aciklama,
                ins.kasa_hareketi_id, ins.created_at
           FROM ins
           LEFT JOIN kasalar ks                    ON ks.id = ins.kasa_id
           LEFT JOIN gelir_gider_kategorileri kt   ON kt.id = ins.kategori_id"
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

    // Geriye donuk tarihler icin bakiyeleri yeniden hesapla
    crate::routes::common::recompute_kasa_bakiyeleri(&pool, input.kasa_id).await?;

    Ok(Json(kayit))
}

async fn delete_kayit(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    user.require_izin(&pool, "gelir_gider.yonet").await?;

    // Once kayitla baglantili kasa hareketini bul
    let row: Option<(i64, Option<i64>)> = sqlx::query_as(
        "SELECT kasa_id, kasa_hareketi_id FROM gelir_gider_kayitlari WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?;

    let Some((kasa_id, hareket_id_opt)) = row else {
        return Ok(Json(serde_json::json!({ "mesaj": "Kayit bulunamadi" })));
    };

    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM gelir_gider_kayitlari WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    if let Some(hid) = hareket_id_opt {
        sqlx::query("DELETE FROM kasa_hareketleri WHERE id = $1")
            .bind(hid)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    // Kalan hareketlerin bakiyesini yeniden hesapla
    crate::routes::common::recompute_kasa_bakiyeleri(&pool, kasa_id).await?;

    Ok(Json(serde_json::json!({ "mesaj": "Kayit silindi" })))
}
