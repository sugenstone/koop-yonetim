use axum::{
    extract::{Path, State},
    routing::{get, post, delete},
    Json, Router,
};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use crate::auth::AuthUser;
use crate::errors::AppResult;
use super::common::{
    ata_hisse_tam, ata_hisseler_toplu_tam, cuzdan_alacak_ekle, cuzdan_borc_ekle, cuzdan_son_bakiye, hissedar_bilgi,
    kasa_cikan_ekle, kasa_giren_ekle, kasa_son_bakiye,
};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Hisse {
    pub id: i64,
    pub kod: String,
    pub durum: String,
    pub aciklama: Option<String>,
    pub hissedar_id: Option<i64>,
    pub hissedar_adi: Option<String>,
    pub atama_tarih: Option<chrono::NaiveDate>,
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
                a.tarih AS atama_tarih,
                h.created_at, h.updated_at
         FROM hisseler h
         LEFT JOIN LATERAL (
             SELECT hissedar_id, tarih FROM hisse_atamalari
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
                a.tarih AS atama_tarih,
                h.created_at, h.updated_at
         FROM hisseler h
         LEFT JOIN LATERAL (
             SELECT hissedar_id, tarih FROM hisse_atamalari
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
                   NULL::DATE AS atama_tarih,
                   created_at, updated_at"
    )
    .bind(kod)
    .bind(input.aciklama)
    .fetch_one(&pool)
    .await?;

    // Hissedar seçildiyse otomatik atama yap (cüzdan + kasa + retroaktif)
    if let Some(hissedar_id) = input.atama_hissedar_id {
        let tarih = input.atama_tarih.unwrap_or_else(|| chrono::Utc::now().date_naive());
        let ucret = input.atama_ucret.unwrap_or(0.0);
        ata_hisse_tam(&pool, hisse.id, hissedar_id, tarih, ucret, input.atama_aciklama.as_deref())
            .await?;
        // Atama bilgisiyle birlikte tekrar çek
        let hisse = sqlx::query_as::<_, Hisse>(
            "SELECT h.id, h.kod, h.durum, h.aciklama,
                    a.hissedar_id,
                    (SELECT soyad || ' ' || ad FROM hissedarlar WHERE id = a.hissedar_id) AS hissedar_adi,
                    a.tarih AS atama_tarih,
                    h.created_at, h.updated_at
             FROM hisseler h
             LEFT JOIN LATERAL (
                 SELECT hissedar_id, tarih FROM hisse_atamalari
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
                       NULL::DATE AS atama_tarih,
                       created_at, updated_at"
        )
        .bind(kod)
        .bind(input.aciklama.clone())
        .fetch_one(&pool)
        .await?;
        olusturulan_idler.push(h.id);
        hisseler.push(h);
    }

    // Hissedar seçildiyse tüm oluşturulan hisseleri TEK toplu çağrı ile ata
    // (cüzdanda tek satır, kasada tek satır, her dönem için tek konsolide borç)
    if let Some(hissedar_id) = input.atama_hissedar_id {
        let tarih = input.atama_tarih.unwrap_or_else(|| chrono::Utc::now().date_naive());
        let ucret = input.atama_ucret.unwrap_or(0.0);
        ata_hisseler_toplu_tam(
            &pool, &olusturulan_idler, hissedar_id, tarih, ucret,
            input.atama_aciklama.as_deref(),
        ).await?;
        // Güncel bilgilerle tekrar çek
        let guncel = sqlx::query_as::<_, Hisse>(
            "SELECT h.id, h.kod, h.durum, h.aciklama,
                    a.hissedar_id,
                    (SELECT soyad || ' ' || ad FROM hissedarlar WHERE id = a.hissedar_id) AS hissedar_adi,
                    a.tarih AS atama_tarih,
                    h.created_at, h.updated_at
             FROM hisseler h
             LEFT JOIN LATERAL (
                 SELECT hissedar_id, tarih FROM hisse_atamalari
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

    let atama_id = ata_hisse_tam(
        &pool, input.hisse_id, input.hissedar_id, input.tarih, input.ucret,
        input.aciklama.as_deref(),
    ).await?;

    let atama = sqlx::query_as::<_, HisseAtama>(
        "SELECT a.id, a.hisse_id, a.hissedar_id,
                (h.soyad || ' ' || h.ad) AS hissedar_adi,
                a.tarih, a.ucret, a.aciklama, a.created_at
         FROM hisse_atamalari a
         JOIN hissedarlar h ON h.id = a.hissedar_id
         WHERE a.id = $1"
    )
    .bind(atama_id)
    .fetch_one(&pool)
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
    use crate::errors::AppError;
    user.require_izin(&pool, "hisse.yonet").await?;

    if input.ucret < 0.0 {
        return Err(AppError::BadRequest("Transfer ücreti negatif olamaz".into()));
    }

    // Hisse bilgisi
    #[derive(sqlx::FromRow)]
    struct HisseRow { kod: String, durum: String }
    let hisse: HisseRow = sqlx::query_as("SELECT kod, durum FROM hisseler WHERE id = $1")
        .bind(input.hisse_id)
        .fetch_one(&pool)
        .await?;
    if hisse.durum == "satildi" {
        return Err(AppError::BadRequest("Bu hisse sisteme satılmış ve artık kullanılamaz.".into()));
    }

    // Aktif satış kontrolü
    let aktif_satis: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM hisse_satislari
         WHERE hisse_id = $1 AND tamamlandi = false AND iptal = false"
    )
    .bind(input.hisse_id)
    .fetch_one(&pool)
    .await?;
    if aktif_satis > 0 {
        return Err(AppError::BadRequest(
            "Bu hisse için devam eden bir satış süreci var. Transfer yapılamaz.".into()
        ));
    }

    // Mevcut sahibi bul
    #[derive(sqlx::FromRow)]
    struct Eski { id: i64, ad: String, soyad: String, kasa_id: i64 }
    let eski: Option<Eski> = sqlx::query_as(
        "SELECT hsd.id, hsd.ad, hsd.soyad, hsd.kasa_id
         FROM hisse_atamalari ha
         JOIN hissedarlar hsd ON hsd.id = ha.hissedar_id
         WHERE ha.hisse_id = $1
         ORDER BY ha.created_at DESC, ha.id DESC LIMIT 1"
    )
    .bind(input.hisse_id)
    .fetch_optional(&pool)
    .await?;
    let eski = eski.ok_or_else(|| AppError::BadRequest(
        "Bu hisse henüz kimseye atanmamış, transfer için önce atama yapın.".into()
    ))?;

    if eski.id == input.yeni_hissedar_id {
        return Err(AppError::BadRequest("Hisse zaten bu hissedarda.".into()));
    }

    let yeni = hissedar_bilgi(&pool, input.yeni_hissedar_id).await?;

    // Atama açıklaması
    let atama_aciklama = match input.aciklama.as_deref() {
        Some(a) if !a.trim().is_empty() =>
            format!("Transfer: {} {} → {} {} | {}", eski.ad, eski.soyad, yeni.ad, yeni.soyad, a),
        _ =>
            format!("Transfer: {} {} → {} {}", eski.ad, eski.soyad, yeni.ad, yeni.soyad),
    };

    // Yeni atama kaydı
    let atama_id: i64 = sqlx::query_scalar(
        "INSERT INTO hisse_atamalari (hisse_id, hissedar_id, tarih, ucret, aciklama)
         VALUES ($1, $2, $3, $4, $5) RETURNING id"
    )
    .bind(input.hisse_id)
    .bind(input.yeni_hissedar_id)
    .bind(input.tarih)
    .bind(input.ucret)
    .bind(&atama_aciklama)
    .fetch_one(&pool)
    .await?;

    sqlx::query("UPDATE hisseler SET durum = 'atanmis', updated_at = NOW() WHERE id = $1")
        .bind(input.hisse_id)
        .execute(&pool)
        .await?;

    // Ücret > 0 ise cüzdan + kasa hareketleri
    if input.ucret > 0.0 {
        let bilgi_alici = format!("Hisse transferi (alım): {} - {} {}", hisse.kod, eski.ad, eski.soyad);
        let bilgi_satici = format!("Hisse transferi (satım): {} - {} {}", hisse.kod, yeni.ad, yeni.soyad);

        // Alıcı cüzdanı: borç; önceki bakiyeyi alacak mantığı ile ayrıca tutuyoruz
        let alici_onceki = cuzdan_son_bakiye(&pool, input.yeni_hissedar_id).await?;
        cuzdan_borc_ekle(&pool, input.yeni_hissedar_id, None, input.tarih, &bilgi_alici, input.ucret).await?;

        // Satıcı cüzdanı: alacak
        cuzdan_alacak_ekle(&pool, eski.id, None, input.tarih, &bilgi_satici, input.ucret).await?;

        // Alıcı cüzdanı yeterliyse: alıcı kasasından çıkış, satıcı kasasına giriş
        if alici_onceki >= input.ucret {
            let alici_kasa_ac = format!("Hisse transfer ödemesi: {} → {} {}", hisse.kod, eski.ad, eski.soyad);
            let satici_kasa_ac = format!("Hisse transfer tahsilatı: {} ← {} {}", hisse.kod, yeni.ad, yeni.soyad);
            kasa_cikan_ekle(&pool, yeni.kasa_id, input.tarih, &alici_kasa_ac, input.ucret).await?;
            kasa_giren_ekle(&pool, eski.kasa_id, input.tarih, &satici_kasa_ac, input.ucret).await?;
        }
    }

    let atama = sqlx::query_as::<_, HisseAtama>(
        "SELECT a.id, a.hisse_id, a.hissedar_id,
                (h.soyad || ' ' || h.ad) AS hissedar_adi,
                a.tarih, a.ucret, a.aciklama, a.created_at
         FROM hisse_atamalari a
         JOIN hissedarlar h ON h.id = a.hissedar_id
         WHERE a.id = $1"
    )
    .bind(atama_id)
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
    use crate::errors::AppError;
    user.require_izin(&pool, "hisse.yonet").await?;

    if input.tutar <= 0.0 {
        return Err(AppError::BadRequest("Ödeme tutarı pozitif olmalıdır".into()));
    }

    // Satış + hisse + hissedar bilgilerini tek sorguda al
    #[derive(sqlx::FromRow)]
    struct SatisCtx {
        hisse_id: i64,
        hisse_kod: String,
        hissedar_id: i64,
        hissedar_ad: String,
        hissedar_soyad: String,
        kasa_id: i64,
        kasa_ad: String,
        satis_tutari: f64,
        tamamlandi: bool,
        iptal: bool,
        odenen: f64,
    }
    let ctx: SatisCtx = sqlx::query_as(
        "SELECT s.hisse_id, hs.kod AS hisse_kod,
                s.hissedar_id, hsd.ad AS hissedar_ad, hsd.soyad AS hissedar_soyad,
                s.kasa_id, k.ad AS kasa_ad,
                s.satis_tutari, s.tamamlandi, s.iptal,
                COALESCE((SELECT SUM(tutar) FROM hisse_satis_odemeleri WHERE satis_id = s.id), 0.0) AS odenen
         FROM hisse_satislari s
         JOIN hisseler hs ON hs.id = s.hisse_id
         JOIN hissedarlar hsd ON hsd.id = s.hissedar_id
         JOIN kasalar k ON k.id = s.kasa_id
         WHERE s.id = $1"
    )
    .bind(satis_id)
    .fetch_one(&pool)
    .await?;

    if ctx.iptal {
        return Err(AppError::BadRequest("Bu satış iptal edilmiş, ödeme eklenemez.".into()));
    }
    if ctx.tamamlandi {
        return Err(AppError::BadRequest("Bu satış tamamlanmış, ödeme eklenemez.".into()));
    }
    let kalan = ctx.satis_tutari - ctx.odenen;
    if input.tutar > kalan + 1e-6 {
        return Err(AppError::BadRequest(format!(
            "Ödeme kalan tutardan büyük olamaz. Kalan: {:.2}", kalan
        )));
    }

    // Kasa bakiyesi kontrol
    let kasa_bak = kasa_son_bakiye(&pool, ctx.kasa_id).await?;
    if kasa_bak < input.tutar {
        return Err(AppError::BadRequest(format!(
            "Kasa bakiyesi yetersiz (mevcut: {:.2}, gerekli: {:.2})", kasa_bak, input.tutar
        )));
    }

    // Ödeme kaydı
    let odeme = sqlx::query_as::<_, HisseSatisOdeme>(
        "INSERT INTO hisse_satis_odemeleri (satis_id, tutar, tarih, aciklama)
         VALUES ($1, $2, $3, $4)
         RETURNING id, satis_id, tutar, tarih, aciklama, created_at"
    )
    .bind(satis_id)
    .bind(input.tutar)
    .bind(input.tarih)
    .bind(&input.aciklama)
    .fetch_one(&pool)
    .await?;

    // Kasa: çıkış (sistem hissedara ödüyor)
    let kasa_ac = format!(
        "Hisse satın alma ödemesi: {} - {} {}",
        ctx.hisse_kod, ctx.hissedar_ad, ctx.hissedar_soyad
    );
    kasa_cikan_ekle(&pool, ctx.kasa_id, input.tarih, &kasa_ac, input.tutar).await?;

    // Cüzdan: alacak (hissedarın eline para geçti)
    let cuzdan_bilgi = format!(
        "Hisse satın alma ödemesi: {} ({})", ctx.hisse_kod, ctx.kasa_ad
    );
    cuzdan_alacak_ekle(&pool, ctx.hissedar_id, None, input.tarih, &cuzdan_bilgi, input.tutar).await?;

    // Tamamlandı mı?
    let yeni_odenen = ctx.odenen + input.tutar;
    if yeni_odenen + 1e-6 >= ctx.satis_tutari {
        sqlx::query(
            "UPDATE hisse_satislari SET tamamlandi = true, tamamlanma_tarihi = $1 WHERE id = $2"
        )
        .bind(input.tarih)
        .bind(satis_id)
        .execute(&pool)
        .await?;
        sqlx::query("UPDATE hisseler SET durum = 'satildi', updated_at = NOW() WHERE id = $1")
            .bind(ctx.hisse_id)
            .execute(&pool)
            .await?;
    }

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
    use crate::errors::AppError;
    user.require_izin(&pool, "hisse.yonet").await?;

    #[derive(sqlx::FromRow)]
    struct S { tamamlandi: bool, odenen: f64 }
    let s: S = sqlx::query_as(
        "SELECT s.tamamlandi,
                COALESCE((SELECT SUM(tutar) FROM hisse_satis_odemeleri WHERE satis_id = s.id), 0.0) AS odenen
         FROM hisse_satislari s WHERE s.id = $1"
    )
    .bind(satis_id)
    .fetch_one(&pool)
    .await?;

    if s.tamamlandi {
        return Err(AppError::BadRequest("Tamamlanmış satış iptal edilemez.".into()));
    }
    if s.odenen > 0.0 {
        return Err(AppError::BadRequest(
            "Bu satış için ödemeler yapılmış. Önce ödeme iadelerini manuel işleyin.".into()
        ));
    }

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


