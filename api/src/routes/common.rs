//! Ortak helper'lar — cüzdan, kasa ve tahsilat mantığı (Tauri ile eşdeğer).

use sqlx::PgPool;

/// Türkçe ay adı
pub fn ay_adi(ay: i32) -> &'static str {
    match ay {
        1 => "Ocak", 2 => "Şubat", 3 => "Mart", 4 => "Nisan",
        5 => "Mayıs", 6 => "Haziran", 7 => "Temmuz", 8 => "Ağustos",
        9 => "Eylül", 10 => "Ekim", 11 => "Kasım", 12 => "Aralık",
        _ => "?",
    }
}

pub fn donem_adi(ay: i32, yil: i32) -> String {
    format!("{} {}", ay_adi(ay), yil)
}

/// Tahsilat açıklaması (kasaya giren kaydında kullanılır)
pub fn tahsilat_aciklamasi(
    donem_adi: &str,
    hisse_sayisi: i64,
    hissedar_ad: &str,
    hissedar_soyad: &str,
    yakin_adi: &Option<String>,
    yakinlik_derecesi: &Option<String>,
) -> String {
    let mut aciklama = format!(
        "Aidat tahsilat: {} - {} hisse - {} {}",
        donem_adi, hisse_sayisi, hissedar_ad, hissedar_soyad
    );
    if let Some(y) = yakin_adi {
        if !y.is_empty() {
            aciklama.push_str(&format!(" (Yakını: {}", y));
            if let Some(d) = yakinlik_derecesi {
                if !d.is_empty() {
                    aciklama.push_str(&format!(", {}", d));
                }
            }
            aciklama.push(')');
        }
    }
    aciklama
}

// ─── Cüzdan ────────────────────────────────────────────────────────────────

/// Hissedarın cüzdanındaki son bakiye (kayıt yoksa 0)
pub async fn cuzdan_son_bakiye(pool: &PgPool, hissedar_id: i64) -> sqlx::Result<f64> {
    let b: Option<f64> = sqlx::query_scalar(
        "SELECT bakiye FROM hissedar_cuzdanlari
         WHERE hissedar_id = $1 ORDER BY id DESC LIMIT 1",
    )
    .bind(hissedar_id)
    .fetch_optional(pool)
    .await?;
    Ok(b.unwrap_or(0.0))
}

/// Cüzdana borç kaydı ekler, yeni bakiyeyi döner
pub async fn cuzdan_borc_ekle(
    pool: &PgPool,
    hissedar_id: i64,
    donem_id: Option<i64>,
    tarih: chrono::NaiveDate,
    bilgi: &str,
    tutar: f64,
) -> sqlx::Result<f64> {
    let onceki = cuzdan_son_bakiye(pool, hissedar_id).await?;
    let yeni = onceki - tutar;
    sqlx::query(
        "INSERT INTO hissedar_cuzdanlari (hissedar_id, donem_id, tarih, bilgi, borc, alacak, bakiye)
         VALUES ($1, $2, $3, $4, $5, 0.0, $6)",
    )
    .bind(hissedar_id)
    .bind(donem_id)
    .bind(tarih)
    .bind(bilgi)
    .bind(tutar)
    .bind(yeni)
    .execute(pool)
    .await?;
    Ok(yeni)
}

/// Cüzdana alacak kaydı ekler
pub async fn cuzdan_alacak_ekle(
    pool: &PgPool,
    hissedar_id: i64,
    donem_id: Option<i64>,
    tarih: chrono::NaiveDate,
    bilgi: &str,
    tutar: f64,
) -> sqlx::Result<f64> {
    let onceki = cuzdan_son_bakiye(pool, hissedar_id).await?;
    let yeni = onceki + tutar;
    sqlx::query(
        "INSERT INTO hissedar_cuzdanlari (hissedar_id, donem_id, tarih, bilgi, borc, alacak, bakiye)
         VALUES ($1, $2, $3, $4, 0.0, $5, $6)",
    )
    .bind(hissedar_id)
    .bind(donem_id)
    .bind(tarih)
    .bind(bilgi)
    .bind(tutar)
    .bind(yeni)
    .execute(pool)
    .await?;
    Ok(yeni)
}

// ─── Kasa ──────────────────────────────────────────────────────────────────

/// Kasanın son bakiyesi (kasa_hareketleri üzerinden)
pub async fn kasa_son_bakiye(pool: &PgPool, kasa_id: i64) -> sqlx::Result<f64> {
    let b: Option<f64> = sqlx::query_scalar(
        "SELECT bakiye FROM kasa_hareketleri
         WHERE kasa_id = $1 ORDER BY tarih DESC, id DESC LIMIT 1",
    )
    .bind(kasa_id)
    .fetch_optional(pool)
    .await?;
    Ok(b.unwrap_or(0.0))
}

/// Kasaya giren kaydı + kasalar.bakiye günceller
pub async fn kasa_giren_ekle(
    pool: &PgPool,
    kasa_id: i64,
    tarih: chrono::NaiveDate,
    aciklama: &str,
    tutar: f64,
) -> sqlx::Result<f64> {
    let onceki = kasa_son_bakiye(pool, kasa_id).await?;
    let yeni = onceki + tutar;
    sqlx::query(
        "INSERT INTO kasa_hareketleri (kasa_id, tarih, aciklama, giren, cikan, bakiye)
         VALUES ($1, $2, $3, $4, 0.0, $5)",
    )
    .bind(kasa_id)
    .bind(tarih)
    .bind(aciklama)
    .bind(tutar)
    .bind(yeni)
    .execute(pool)
    .await?;
    sqlx::query("UPDATE kasalar SET bakiye = $1, updated_at = NOW() WHERE id = $2")
        .bind(yeni)
        .bind(kasa_id)
        .execute(pool)
        .await?;
    Ok(yeni)
}

/// Kasadan çıkan kaydı + kasalar.bakiye günceller
pub async fn kasa_cikan_ekle(
    pool: &PgPool,
    kasa_id: i64,
    tarih: chrono::NaiveDate,
    aciklama: &str,
    tutar: f64,
) -> sqlx::Result<f64> {
    let onceki = kasa_son_bakiye(pool, kasa_id).await?;
    let yeni = onceki - tutar;
    sqlx::query(
        "INSERT INTO kasa_hareketleri (kasa_id, tarih, aciklama, giren, cikan, bakiye)
         VALUES ($1, $2, $3, 0.0, $4, $5)",
    )
    .bind(kasa_id)
    .bind(tarih)
    .bind(aciklama)
    .bind(tutar)
    .bind(yeni)
    .execute(pool)
    .await?;
    sqlx::query("UPDATE kasalar SET bakiye = $1, updated_at = NOW() WHERE id = $2")
        .bind(yeni)
        .bind(kasa_id)
        .execute(pool)
        .await?;
    Ok(yeni)
}

// ─── Hissedar yardımcıları ─────────────────────────────────────────────────

#[derive(Debug, sqlx::FromRow)]
pub struct HissedarBilgi {
    pub id: i64,
    pub ad: String,
    pub soyad: String,
    pub kasa_id: i64,
    pub yakin_adi: Option<String>,
    pub yakinlik_derecesi: Option<String>,
}

pub async fn hissedar_bilgi(pool: &PgPool, hissedar_id: i64) -> sqlx::Result<HissedarBilgi> {
    sqlx::query_as::<_, HissedarBilgi>(
        "SELECT id, ad, soyad, kasa_id, yakin_adi, yakinlik_derecesi
         FROM hissedarlar WHERE id = $1",
    )
    .bind(hissedar_id)
    .fetch_one(pool)
    .await
}

// ─── Retroaktif Dönem Borçları (KONSOLİDE) ─────────────────────────────────

/// Yeni atanan hissedara geçmiş tüm dönemler için borç oluşturur (güncel tarife ile).
/// `adet` kadar hisse için tek kayıt — cüzdanda her dönem için TEK satır,
/// donem_aidat_borclari'nda her dönem için TEK konsolide satır.
/// Tauri'deki ata_hisseler_toplu_conn ile birebir aynı mantık.
/// Retroaktif borçlar `odenmemiş` olarak kalır; manuel tahsilat bekler.
pub async fn retroaktif_donem_borclari_toplu(
    pool: &PgPool,
    hissedar_id: i64,
    adet: i64,
    _kodlar_str: &str,
) -> sqlx::Result<u64> {
    if adet <= 0 {
        return Ok(0);
    }

    // En güncel dönem tarifesi (adalet gereği tüm geçmiş dönemler bu tarife)
    let aidat: Option<f64> = sqlx::query_scalar(
        "SELECT hisse_basi_aidat FROM donemler
         ORDER BY yil DESC, ay DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    let aidat_tutari = match aidat {
        Some(t) if t > 0.0 => t,
        _ => return Ok(0),
    };

    let h = hissedar_bilgi(pool, hissedar_id).await?;

    #[derive(sqlx::FromRow)]
    struct DonemBilgi { id: i64, ay: i32, yil: i32 }
    let donemler: Vec<DonemBilgi> = sqlx::query_as(
        "SELECT id, ay, yil FROM donemler ORDER BY yil ASC, ay ASC",
    )
    .fetch_all(pool)
    .await?;

    let tarih = chrono::Utc::now().date_naive();
    let donem_toplam_tutar = aidat_tutari * adet as f64;
    let mut eklenen: u64 = 0;

    for donem in &donemler {
        let donem_ad = donem_adi(donem.ay, donem.yil);

        // Bu dönem + hissedar için ödenmemiş borç var mı?
        #[derive(sqlx::FromRow)]
        struct Mevcut { id: i64, tutar: f64, hisse_sayisi: i32 }
        let mevcut: Option<Mevcut> = sqlx::query_as(
            "SELECT id, tutar, hisse_sayisi FROM donem_aidat_borclari
             WHERE donem_id = $1 AND hissedar_id = $2 AND odendi = false
             ORDER BY id DESC LIMIT 1",
        )
        .bind(donem.id)
        .bind(hissedar_id)
        .fetch_optional(pool)
        .await?;

        if let Some(m) = mevcut {
            // Güncelle: mevcut + adet kadar ekleme
            let yeni_tutar = m.tutar + donem_toplam_tutar;
            let yeni_hs = m.hisse_sayisi + adet as i32;
            let borc_aciklama = format!(
                "{} aidatı - {} hisse ({} {}) [Geçmiş dönem - güncel tarife]",
                donem_ad, yeni_hs, h.ad, h.soyad
            );

            sqlx::query(
                "UPDATE donem_aidat_borclari
                 SET tutar=$1, hisse_sayisi=$2, aciklama=$3
                 WHERE id=$4",
            )
            .bind(yeni_tutar)
            .bind(yeni_hs)
            .bind(&borc_aciklama)
            .bind(m.id)
            .execute(pool)
            .await?;
        } else {
            // Yeni kayıt (ödenmemiş; Tauri ile aynı mantık)
            let borc_aciklama = format!(
                "{} aidatı - {} hisse ({} {}) [Geçmiş dönem - güncel tarife]",
                donem_ad, adet, h.ad, h.soyad
            );
            sqlx::query(
                "INSERT INTO donem_aidat_borclari
                     (donem_id, hissedar_id, hisse_sayisi, tutar, odendi, odeme_tarihi, aciklama)
                 VALUES ($1, $2, $3, $4, false, NULL, $5)",
            )
            .bind(donem.id)
            .bind(hissedar_id)
            .bind(adet as i32)
            .bind(donem_toplam_tutar)
            .bind(&borc_aciklama)
            .execute(pool)
            .await?;
            eklenen += 1;
        }

        // Cüzdana TEK konsolide borç kaydı (adet hisse)
        let cuzdan_borc_bilgi = format!("{} aidatı - {} hisse [geçmiş dönem]", donem_ad, adet);
        cuzdan_borc_ekle(pool, hissedar_id, Some(donem.id), tarih, &cuzdan_borc_bilgi, donem_toplam_tutar).await?;
    }

    Ok(eklenen)
}

// ─── Tam Hisse Atama - TOPLU (kasa + cüzdan + retroaktif, KONSOLİDE) ───────

/// Birden çok hisseyi aynı hissedara TEK seferde atar; cüzdan/kasa/dönem
/// kayıtları konsolide (tek satır) oluşturulur. Tauri'deki
/// ata_hisseler_toplu_conn ile birebir aynı mantık.
///
/// Döndürdüğü: oluşturulan atama id'leri (hisse_idler ile aynı sırada).
pub async fn ata_hisseler_toplu_tam(
    pool: &PgPool,
    hisse_idler: &[i64],
    hissedar_id: i64,
    tarih: chrono::NaiveDate,
    ucret_per_hisse: f64,
    aciklama: Option<&str>,
) -> anyhow::Result<Vec<i64>> {
    if hisse_idler.is_empty() {
        return Ok(Vec::new());
    }
    if ucret_per_hisse < 0.0 {
        anyhow::bail!("Ücret negatif olamaz");
    }

    let adet = hisse_idler.len() as i64;
    let toplam_ucret = ucret_per_hisse * adet as f64;

    let h = hissedar_bilgi(pool, hissedar_id).await?;

    // Hisse kodlarını topla + her hisse için atama ve durum güncelle
    let mut atama_idler: Vec<i64> = Vec::with_capacity(hisse_idler.len());
    let mut hisse_kodlari: Vec<String> = Vec::with_capacity(hisse_idler.len());

    for &hid in hisse_idler {
        let kod: String = sqlx::query_scalar("SELECT kod FROM hisseler WHERE id = $1")
            .bind(hid)
            .fetch_one(pool)
            .await?;
        hisse_kodlari.push(kod);

        let aid: i64 = sqlx::query_scalar(
            "INSERT INTO hisse_atamalari (hisse_id, hissedar_id, tarih, ucret, aciklama)
             VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
        .bind(hid)
        .bind(hissedar_id)
        .bind(tarih)
        .bind(ucret_per_hisse)
        .bind(aciklama)
        .fetch_one(pool)
        .await?;
        atama_idler.push(aid);

        sqlx::query("UPDATE hisseler SET durum = 'atanmis', updated_at = NOW() WHERE id = $1")
            .bind(hid)
            .execute(pool)
            .await?;
    }

    let kodlar_str = hisse_kodlari.join(", ");

    // ── Ücret varsa: TEK konsolide cüzdan borç + (yeterliyse) TEK kasa giren ─
    if toplam_ucret > 0.0 {
        let onceki_cuzdan = cuzdan_son_bakiye(pool, hissedar_id).await?;
        let cuzdan_bilgi = format!("Hisse satın alma: {} ({} hisse)", kodlar_str, adet);
        cuzdan_borc_ekle(pool, hissedar_id, None, tarih, &cuzdan_bilgi, toplam_ucret).await?;

        if onceki_cuzdan >= toplam_ucret {
            let kasa_ac = format!(
                "Hisse satın alma tahsilatı: {} ({} hisse) - {} {}",
                kodlar_str, adet, h.ad, h.soyad
            );
            kasa_giren_ekle(pool, h.kasa_id, tarih, &kasa_ac, toplam_ucret).await?;
        }
    }

    // ── Retroaktif dönem borçları (her dönem için TEK konsolide kayıt) ─────
    retroaktif_donem_borclari_toplu(pool, hissedar_id, adet, &kodlar_str).await?;

    Ok(atama_idler)
}

// ─── Tek Hisse Atama wrapper ───────────────────────────────────────────────

/// Tek bir hisseyi atamak için `ata_hisseler_toplu_tam` wrapper'ı.
/// Döner: oluşturulan atama id.
pub async fn ata_hisse_tam(
    pool: &PgPool,
    hisse_id: i64,
    hissedar_id: i64,
    tarih: chrono::NaiveDate,
    ucret: f64,
    aciklama: Option<&str>,
) -> anyhow::Result<i64> {
    let idler = ata_hisseler_toplu_tam(pool, &[hisse_id], hissedar_id, tarih, ucret, aciklama).await?;
    idler.into_iter().next().ok_or_else(|| anyhow::anyhow!("Atama oluşturulamadı"))
}

