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

// ─── Retroaktif Dönem Borçları ─────────────────────────────────────────────

/// Yeni atanan hissedara geçmiş tüm dönemler için borç oluşturur (güncel tarife ile).
/// Tauri'deki ata_hisse_conn içindeki retroaktif mantığın PostgreSQL karşılığı.
///
/// `hisse_kod` aktif yeni atanan hisse kodu (bilgi metinlerinde kullanılır).
pub async fn retroaktif_donem_borclari(
    pool: &PgPool,
    hissedar_id: i64,
    hisse_kod: &str,
) -> sqlx::Result<u64> {
    // En güncel dönemin aidat tutarı (adalet gereği tüm geçmiş dönemler bu tarife)
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
    let mut eklenen: u64 = 0;

    for donem in &donemler {
        let donem_ad = donem_adi(donem.ay, donem.yil);

        // Bu dönem + hissedar için mevcut ödenmemiş borç
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

        let cuzdan_onceki = cuzdan_son_bakiye(pool, hissedar_id).await?;

        if let Some(m) = mevcut {
            // Var olan borcu güncelle (+1 hisse, +aidat_tutari)
            let yeni_tutar = m.tutar + aidat_tutari;
            let yeni_hs = m.hisse_sayisi + 1;
            let yeterli = cuzdan_onceki >= yeni_tutar;
            let borc_aciklama = format!(
                "{} aidatı - {} hisse ({} {}) [Geçmiş dönem - güncel tarife]",
                donem_ad, yeni_hs, h.ad, h.soyad
            );

            sqlx::query(
                "UPDATE donem_aidat_borclari
                 SET tutar=$1, hisse_sayisi=$2, aciklama=$3, odendi=$4, odeme_tarihi=$5
                 WHERE id=$6",
            )
            .bind(yeni_tutar)
            .bind(yeni_hs)
            .bind(&borc_aciklama)
            .bind(yeterli)
            .bind(if yeterli { Some(tarih) } else { None })
            .bind(m.id)
            .execute(pool)
            .await?;

            // Cüzdana delta borç (sadece yeni hisse kadar)
            let bilgi_borc = format!("{} aidatı - {} [geçmiş dönem]", donem_ad, hisse_kod);
            cuzdan_borc_ekle(pool, hissedar_id, Some(donem.id), tarih, &bilgi_borc, aidat_tutari).await?;

            if yeterli {
                // Otomatik tahsilat: tam tutarı kasaya aktar + cüzdanda mahsup et
                let kasa_ac = tahsilat_aciklamasi(
                    &donem_ad, yeni_hs as i64, &h.ad, &h.soyad, &h.yakin_adi, &h.yakinlik_derecesi
                );
                kasa_giren_ekle(pool, h.kasa_id, tarih, &kasa_ac, yeni_tutar).await?;
                let tahsil_bilgi = format!("Tahsilat: {} - {} hisse", donem_ad, yeni_hs);
                cuzdan_alacak_ekle(pool, hissedar_id, Some(donem.id), tarih, &tahsil_bilgi, yeni_tutar).await?;
            }
        } else {
            // Yeni borç kaydı
            let yeterli = cuzdan_onceki >= aidat_tutari;
            let borc_aciklama = format!(
                "{} aidatı - 1 hisse ({} {}) [Geçmiş dönem - güncel tarife]",
                donem_ad, h.ad, h.soyad
            );
            sqlx::query(
                "INSERT INTO donem_aidat_borclari
                     (donem_id, hissedar_id, hisse_sayisi, tutar, odendi, odeme_tarihi, aciklama)
                 VALUES ($1, $2, 1, $3, $4, $5, $6)",
            )
            .bind(donem.id)
            .bind(hissedar_id)
            .bind(aidat_tutari)
            .bind(yeterli)
            .bind(if yeterli { Some(tarih) } else { None })
            .bind(&borc_aciklama)
            .execute(pool)
            .await?;

            let bilgi_borc = format!("{} aidatı - {} [geçmiş dönem]", donem_ad, hisse_kod);
            cuzdan_borc_ekle(pool, hissedar_id, Some(donem.id), tarih, &bilgi_borc, aidat_tutari).await?;

            if yeterli {
                let kasa_ac = tahsilat_aciklamasi(
                    &donem_ad, 1, &h.ad, &h.soyad, &h.yakin_adi, &h.yakinlik_derecesi
                );
                kasa_giren_ekle(pool, h.kasa_id, tarih, &kasa_ac, aidat_tutari).await?;
                let tahsil_bilgi = format!("Tahsilat: {} - 1 hisse", donem_ad);
                cuzdan_alacak_ekle(pool, hissedar_id, Some(donem.id), tarih, &tahsil_bilgi, aidat_tutari).await?;
            }
            eklenen += 1;
        }
    }

    Ok(eklenen)
}

// ─── Tam Hisse Atama (kasa + cüzdan + retroaktif) ──────────────────────────

/// Bir hisseyi hissedara atar ve Tauri ata_hisse_conn ile birebir mantıkta
/// cüzdan/kasa/retroaktif kayıtlarını oluşturur.
///
/// Ön koşul: hisse 'musait' durumda olmalı.
pub async fn ata_hisse_tam(
    pool: &PgPool,
    hisse_id: i64,
    hissedar_id: i64,
    tarih: chrono::NaiveDate,
    ucret: f64,
    aciklama: Option<&str>,
) -> anyhow::Result<i64> {
    if ucret < 0.0 {
        anyhow::bail!("Ücret negatif olamaz");
    }

    let hisse_kod: String = sqlx::query_scalar("SELECT kod FROM hisseler WHERE id = $1")
        .bind(hisse_id)
        .fetch_one(pool)
        .await?;

    let h = hissedar_bilgi(pool, hissedar_id).await?;

    // Atama kaydı
    let atama_id: i64 = sqlx::query_scalar(
        "INSERT INTO hisse_atamalari (hisse_id, hissedar_id, tarih, ucret, aciklama)
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(hisse_id)
    .bind(hissedar_id)
    .bind(tarih)
    .bind(ucret)
    .bind(aciklama)
    .fetch_one(pool)
    .await?;

    sqlx::query("UPDATE hisseler SET durum = 'atanmis', updated_at = NOW() WHERE id = $1")
        .bind(hisse_id)
        .execute(pool)
        .await?;

    // Ücret > 0 → cüzdan borç + (bakiye yeterliyse kasa giren)
    if ucret > 0.0 {
        let onceki = cuzdan_son_bakiye(pool, hissedar_id).await?;
        let bilgi = format!("Hisse satın alma: {}", hisse_kod);
        cuzdan_borc_ekle(pool, hissedar_id, None, tarih, &bilgi, ucret).await?;

        if onceki >= ucret {
            let kasa_ac = format!(
                "Hisse satın alma tahsilatı: {} - {} {}",
                hisse_kod, h.ad, h.soyad
            );
            kasa_giren_ekle(pool, h.kasa_id, tarih, &kasa_ac, ucret).await?;
        }
    }

    // Retroaktif dönem borçları
    retroaktif_donem_borclari(pool, hissedar_id, &hisse_kod).await?;

    Ok(atama_id)
}
