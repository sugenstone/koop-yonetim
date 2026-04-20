use crate::db::Db;
use rusqlite::params;
use serde::{Deserialize, Serialize};

// ─── Modeller ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AidatBorcu {
    pub id: i64,
    pub donem_id: i64,
    pub donem_adi: String, // "Ocak 2026" vb.
    pub hissedar_id: i64,
    pub hissedar_ad: String,
    pub hissedar_soyad: String,
    pub hisse_sayisi: i64,
    pub tutar: f64,
    pub odendi: bool,
    pub odeme_tarihi: Option<String>,
    pub aciklama: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BorcOlusturSonuc {
    pub olusturulan: i64,
    pub otomatik_tahsil: i64,
    pub tahsil_edilemeyen: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CuzdanHareketi {
    pub id: i64,
    pub hissedar_id: i64,
    pub donem_id: Option<i64>,
    pub donem_adi: Option<String>,
    pub tarih: String,
    pub bilgi: String,
    pub borc: f64,
    pub alacak: f64,
    pub bakiye: f64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CuzdanParaEkleInput {
    pub hissedar_id: i64,
    pub tutar: f64,
    pub aciklama: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CuzdanParaEkleSonuc {
    pub yeni_bakiye: f64,
    pub tahsil_edilen_borc_sayisi: i64,
    pub tahsil_edilen_toplam: f64,
}

// ─── Ay adları (Rust tarafı) ─────────────────────────────────────────────────

pub(crate) fn ay_adi(ay: i64) -> &'static str {
    match ay {
        1 => "Ocak",
        2 => "Şubat",
        3 => "Mart",
        4 => "Nisan",
        5 => "Mayıs",
        6 => "Haziran",
        7 => "Temmuz",
        8 => "Ağustos",
        9 => "Eylül",
        10 => "Ekim",
        11 => "Kasım",
        12 => "Aralık",
        _ => "?",
    }
}

// ─── Komutlar ────────────────────────────────────────────────────────────────

/// Hissedarın cüzdan son bakiyesini getir
pub(crate) fn cuzdan_son_bakiye(conn: &rusqlite::Connection, hissedar_id: i64) -> f64 {
    conn.query_row(
        "SELECT COALESCE(bakiye, 0.0) FROM hissedar_cuzdanlari
         WHERE hissedar_id = ?1 ORDER BY id DESC LIMIT 1",
        params![hissedar_id],
        |r| r.get(0),
    )
    .unwrap_or(0.0)
}

/// Kasa son bakiyesini getir
pub(crate) fn kasa_son_bakiye(conn: &rusqlite::Connection, kasa_id: i64) -> f64 {
    conn.query_row(
        "SELECT COALESCE(bakiye, 0.0) FROM kasa_hareketleri
         WHERE kasa_id = ?1 ORDER BY tarih DESC, id DESC LIMIT 1",
        params![kasa_id],
        |r| r.get(0),
    )
    .unwrap_or(0.0)
}

/// Tahsilat açıklaması oluştur (hissedar adı, yakın adı, yakınlık derecesi)
pub(crate) fn tahsilat_aciklamasi(
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
    if let Some(yakin) = yakin_adi {
        if !yakin.is_empty() {
            aciklama.push_str(&format!(" (Yakını: {}", yakin));
            if let Some(derece) = yakinlik_derecesi {
                if !derece.is_empty() {
                    aciklama.push_str(&format!(", {}", derece));
                }
            }
            aciklama.push(')');
        }
    }
    aciklama
}

/// Dönem için borç kaydı oluştur
/// - Atanmış her hisse için hissedar'a hisse_basi_aidat kadar borç yaz
/// - Hissedar cüzdanına borç kaydı ekle
/// - Cüzdan bakiyesi yeterliyse otomatik tahsil et ve kasaya aktar
#[tauri::command]
pub fn donem_borc_olustur(db: Db<'_>, donem_id: i64) -> Result<BorcOlusturSonuc, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Dönem bilgisini al
    let (hisse_basi_aidat, ay, yil): (f64, i64, i64) = conn
        .query_row(
            "SELECT hisse_basi_aidat, ay, yil FROM donemler WHERE id = ?1",
            params![donem_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .map_err(|e| format!("Dönem bulunamadı: {e}"))?;

    if hisse_basi_aidat <= 0.0 {
        return Err("Hisse başı aidat tutarı 0 veya negatif, borç oluşturulamaz".to_string());
    }

    // Atanmış hisseleri bul (son atama kaydına göre hissedar)
    let mut stmt = conn
        .prepare(
            "SELECT h.id, h.kod, ha.hissedar_id, hsd.ad, hsd.soyad, hsd.kasa_id,
                    hsd.yakin_adi, hsd.yakinlik_derecesi
             FROM hisseler h
             JOIN hisse_atamalari ha ON ha.hisse_id = h.id
             JOIN hissedarlar hsd ON hsd.id = ha.hissedar_id
             WHERE h.durum = 'atanmis'
               AND ha.id = (
                   SELECT id FROM hisse_atamalari
                   WHERE hisse_id = h.id
                   ORDER BY created_at DESC LIMIT 1
               )",
        )
        .map_err(|e| e.to_string())?;

    struct AtanmisHisse {
        hisse_kod: String,
        hissedar_id: i64,
        hissedar_ad: String,
        hissedar_soyad: String,
        kasa_id: i64,
        yakin_adi: Option<String>,
        yakinlik_derecesi: Option<String>,
    }

    let atanmis_hisseler: Vec<AtanmisHisse> = stmt
        .query_map([], |row| {
            Ok(AtanmisHisse {
                hisse_kod: row.get(1)?,
                hissedar_id: row.get(2)?,
                hissedar_ad: row.get(3)?,
                hissedar_soyad: row.get(4)?,
                kasa_id: row.get(5)?,
                yakin_adi: row.get(6)?,
                yakinlik_derecesi: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    if atanmis_hisseler.is_empty() {
        return Err("Atanmış hisse bulunamadı, borç oluşturulamaz".to_string());
    }

    // Hissedarlar bazında grupla: her hissedar için tek konsolide kayıt
    struct HissedarBorc {
        hissedar_id: i64,
        hissedar_ad: String,
        hissedar_soyad: String,
        kasa_id: i64,
        yakin_adi: Option<String>,
        yakinlik_derecesi: Option<String>,
        hisse_sayisi: i64,
        hisse_kodlari: Vec<String>,
    }

    let mut hissedar_map: std::collections::HashMap<i64, HissedarBorc> =
        std::collections::HashMap::new();

    for ah in &atanmis_hisseler {
        let entry = hissedar_map.entry(ah.hissedar_id).or_insert(HissedarBorc {
            hissedar_id: ah.hissedar_id,
            hissedar_ad: ah.hissedar_ad.clone(),
            hissedar_soyad: ah.hissedar_soyad.clone(),
            kasa_id: ah.kasa_id,
            yakin_adi: ah.yakin_adi.clone(),
            yakinlik_derecesi: ah.yakinlik_derecesi.clone(),
            hisse_sayisi: 0,
            hisse_kodlari: Vec::new(),
        });
        entry.hisse_sayisi += 1;
        entry.hisse_kodlari.push(ah.hisse_kod.clone());
    }

    let donem_adi = format!("{} {}", ay_adi(ay), yil);
    let tarih = chrono_today();
    let mut olusturulan: i64 = 0;
    let mut otomatik_tahsil: i64 = 0;
    let mut tahsil_edilemeyen: i64 = 0;

    for (_, hissedar) in &hissedar_map {
        // Bu dönem+hissedar için zaten borç var mı?
        let var_mi: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM donem_aidat_borclari WHERE donem_id = ?1 AND hissedar_id = ?2",
                params![donem_id, hissedar.hissedar_id],
                |r| r.get(0),
            )
            .unwrap_or(false);

        if var_mi {
            continue; // Zaten oluşturulmuş, atla
        }

        let toplam_tutar = hissedar.hisse_sayisi as f64 * hisse_basi_aidat;
        let borc_aciklama = format!(
            "{} aidatı - {} hisse ({}) ({} {})",
            donem_adi,
            hissedar.hisse_sayisi,
            hissedar.hisse_kodlari.join(", "),
            hissedar.hissedar_ad,
            hissedar.hissedar_soyad
        );

        // Cüzdan bakiyesini kontrol et
        let onceki_bakiye = cuzdan_son_bakiye(&conn, hissedar.hissedar_id);
        let yeterli = onceki_bakiye >= toplam_tutar;

        // Konsolide borç kaydı oluştur
        conn.execute(
            "INSERT INTO donem_aidat_borclari (donem_id, hissedar_id, hisse_sayisi, tutar, odendi, odeme_tarihi, aciklama)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                donem_id,
                hissedar.hissedar_id,
                hissedar.hisse_sayisi,
                toplam_tutar,
                yeterli as i64,
                if yeterli { Some(&tarih) } else { None },
                borc_aciklama,
            ],
        )
        .map_err(|e| format!("Borç kaydı oluşturma hatası: {e}"))?;

        olusturulan += 1;

        // Cüzdana borç kaydı (dönem borcu anında cüzdana yansır)
        let cuzdan_borc_bilgi = format!("{} aidatı - {} hisse", donem_adi, hissedar.hisse_sayisi);
        let c_borc_onceki = cuzdan_son_bakiye(&conn, hissedar.hissedar_id);
        let c_borc_yeni = c_borc_onceki - toplam_tutar;
        conn.execute(
            "INSERT INTO hissedar_cuzdanlari (hissedar_id, donem_id, tarih, bilgi, borc, alacak, bakiye)
             VALUES (?1, ?2, ?3, ?4, ?5, 0.0, ?6)",
            params![hissedar.hissedar_id, donem_id, &tarih, cuzdan_borc_bilgi, toplam_tutar, c_borc_yeni],
        )
        .map_err(|e| format!("Cüzdan borç kaydı hatası: {e}"))?;

        // Otomatik tahsilat: cüzdan bakiye yeterliyse kasaya aktar
        if yeterli {
            let kasa_aciklama = tahsilat_aciklamasi(
                &donem_adi,
                hissedar.hisse_sayisi,
                &hissedar.hissedar_ad,
                &hissedar.hissedar_soyad,
                &hissedar.yakin_adi,
                &hissedar.yakinlik_derecesi,
            );

            let kasa_onceki = kasa_son_bakiye(&conn, hissedar.kasa_id);
            let kasa_yeni = kasa_onceki + toplam_tutar;

            conn.execute(
                "INSERT INTO kasa_hareketleri (kasa_id, tarih, aciklama, giren, cikan, bakiye)
                 VALUES (?1, ?2, ?3, ?4, 0.0, ?5)",
                params![hissedar.kasa_id, &tarih, kasa_aciklama, toplam_tutar, kasa_yeni],
            )
            .map_err(|e| e.to_string())?;

            conn.execute(
                "UPDATE kasalar SET bakiye = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![kasa_yeni, hissedar.kasa_id],
            )
            .map_err(|e| e.to_string())?;

            // Cüzdana alacak kaydı (nakit kasaya geçti, borcu mahsup eder)
            let cuzdan_tahsil_bilgi = format!("Tahsilat: {} - {} hisse", donem_adi, hissedar.hisse_sayisi);
            let c_tahsil_onceki = cuzdan_son_bakiye(&conn, hissedar.hissedar_id);
            let c_tahsil_yeni = c_tahsil_onceki + toplam_tutar;
            conn.execute(
                "INSERT INTO hissedar_cuzdanlari (hissedar_id, donem_id, tarih, bilgi, borc, alacak, bakiye)
                 VALUES (?1, ?2, ?3, ?4, 0.0, ?5, ?6)",
                params![hissedar.hissedar_id, donem_id, &tarih, cuzdan_tahsil_bilgi, toplam_tutar, c_tahsil_yeni],
            )
            .map_err(|e| format!("Cüzdan tahsilat kaydı hatası: {e}"))?;

            otomatik_tahsil += 1;
        } else {
            tahsil_edilemeyen += 1;
        }
    }

    Ok(BorcOlusturSonuc {
        olusturulan,
        otomatik_tahsil,
        tahsil_edilemeyen,
    })
}

/// Dönemdeki tüm borç kayıtlarını getir
#[tauri::command]
pub fn get_donem_borclari(db: Db<'_>, donem_id: i64) -> Result<Vec<AidatBorcu>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT b.id, b.donem_id, d.ay, d.yil,
                    b.hissedar_id, hsd.ad, hsd.soyad,
                    b.hisse_sayisi, b.tutar, b.odendi, b.odeme_tarihi, b.aciklama, b.created_at
             FROM donem_aidat_borclari b
             JOIN donemler d ON d.id = b.donem_id
             JOIN hissedarlar hsd ON hsd.id = b.hissedar_id
             WHERE b.donem_id = ?1
             ORDER BY hsd.ad, hsd.soyad",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![donem_id], |row| {
            let ay: i64 = row.get(2)?;
            let yil: i64 = row.get(3)?;
            Ok(AidatBorcu {
                id: row.get(0)?,
                donem_id: row.get(1)?,
                donem_adi: format!("{} {}", ay_adi(ay), yil),
                hissedar_id: row.get(4)?,
                hissedar_ad: row.get(5)?,
                hissedar_soyad: row.get(6)?,
                hisse_sayisi: row.get(7)?,
                tutar: row.get(8)?,
                odendi: row.get::<_, i64>(9)? != 0,
                odeme_tarihi: row.get(10)?,
                aciklama: row.get(11)?,
                created_at: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

/// Bir hissenin güncel sahibinin borç kayıtlarını getir (hisse detay sayfası için)
#[tauri::command]
pub fn get_hisse_borclari(db: Db<'_>, hisse_id: i64) -> Result<Vec<AidatBorcu>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT b.id, b.donem_id, d.ay, d.yil,
                    b.hissedar_id, hsd.ad, hsd.soyad,
                    b.hisse_sayisi, b.tutar, b.odendi, b.odeme_tarihi, b.aciklama, b.created_at
             FROM donem_aidat_borclari b
             JOIN donemler d ON d.id = b.donem_id
             JOIN hissedarlar hsd ON hsd.id = b.hissedar_id
             WHERE b.hissedar_id = (
                 SELECT hissedar_id FROM hisse_atamalari
                 WHERE hisse_id = ?1
                 ORDER BY created_at DESC LIMIT 1
             )
             ORDER BY d.yil DESC, d.ay DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![hisse_id], |row| {
            let ay: i64 = row.get(2)?;
            let yil: i64 = row.get(3)?;
            Ok(AidatBorcu {
                id: row.get(0)?,
                donem_id: row.get(1)?,
                donem_adi: format!("{} {}", ay_adi(ay), yil),
                hissedar_id: row.get(4)?,
                hissedar_ad: row.get(5)?,
                hissedar_soyad: row.get(6)?,
                hisse_sayisi: row.get(7)?,
                tutar: row.get(8)?,
                odendi: row.get::<_, i64>(9)? != 0,
                odeme_tarihi: row.get(10)?,
                aciklama: row.get(11)?,
                created_at: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

/// Bir hissedarın tüm borç kayıtlarını getir (hissedar detay sayfası için)
#[tauri::command]
pub fn get_hissedar_borclari(db: Db<'_>, hissedar_id: i64) -> Result<Vec<AidatBorcu>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT b.id, b.donem_id, d.ay, d.yil,
                    b.hissedar_id, hsd.ad, hsd.soyad,
                    b.hisse_sayisi, b.tutar, b.odendi, b.odeme_tarihi, b.aciklama, b.created_at
             FROM donem_aidat_borclari b
             JOIN donemler d ON d.id = b.donem_id
             JOIN hissedarlar hsd ON hsd.id = b.hissedar_id
             WHERE b.hissedar_id = ?1
             ORDER BY d.yil DESC, d.ay DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![hissedar_id], |row| {
            let ay: i64 = row.get(2)?;
            let yil: i64 = row.get(3)?;
            Ok(AidatBorcu {
                id: row.get(0)?,
                donem_id: row.get(1)?,
                donem_adi: format!("{} {}", ay_adi(ay), yil),
                hissedar_id: row.get(4)?,
                hissedar_ad: row.get(5)?,
                hissedar_soyad: row.get(6)?,
                hisse_sayisi: row.get(7)?,
                tutar: row.get(8)?,
                odendi: row.get::<_, i64>(9)? != 0,
                odeme_tarihi: row.get(10)?,
                aciklama: row.get(11)?,
                created_at: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

// ─── Cüzdan Komutları ────────────────────────────────────────────────────────

/// Hissedarın cüzdan hareketlerini getir (son işlem üstte)
#[tauri::command]
pub fn get_hissedar_cuzdani(db: Db<'_>, hissedar_id: i64) -> Result<Vec<CuzdanHareketi>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.hissedar_id, c.donem_id, d.ay, d.yil,
                    c.tarih, c.bilgi, c.borc, c.alacak, c.bakiye, c.created_at
             FROM hissedar_cuzdanlari c
             LEFT JOIN donemler d ON d.id = c.donem_id
             WHERE c.hissedar_id = ?1
             ORDER BY c.id DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![hissedar_id], |row| {
            let donem_id: Option<i64> = row.get(2)?;
            let ay: Option<i64> = row.get(3)?;
            let yil: Option<i64> = row.get(4)?;
            let donem_adi = match (ay, yil) {
                (Some(a), Some(y)) => Some(format!("{} {}", ay_adi(a), y)),
                _ => None,
            };
            Ok(CuzdanHareketi {
                id: row.get(0)?,
                hissedar_id: row.get(1)?,
                donem_id,
                donem_adi,
                tarih: row.get(5)?,
                bilgi: row.get(6)?,
                borc: row.get(7)?,
                alacak: row.get(8)?,
                bakiye: row.get(9)?,
                created_at: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

/// Hissedar cüzdanına para ekle ve ödenmemiş borçları otomatik tahsil et
#[tauri::command]
pub fn hissedar_cuzdan_para_ekle(
    db: Db<'_>,
    input: CuzdanParaEkleInput,
) -> Result<CuzdanParaEkleSonuc, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    if input.tutar <= 0.0 {
        return Err("Tutar 0'dan büyük olmalıdır".to_string());
    }

    let tarih = chrono_today();
    let bilgi = input.aciklama.unwrap_or_else(|| "Para yükleme".to_string());

    // Cüzdana alacak kaydı ekle
    let onceki_bakiye = cuzdan_son_bakiye(&conn, input.hissedar_id);
    let yeni_bakiye = onceki_bakiye + input.tutar;

    conn.execute(
        "INSERT INTO hissedar_cuzdanlari (hissedar_id, donem_id, tarih, bilgi, borc, alacak, bakiye)
         VALUES (?1, NULL, ?2, ?3, 0.0, ?4, ?5)",
        params![input.hissedar_id, &tarih, bilgi, input.tutar, yeni_bakiye],
    )
    .map_err(|e| format!("Cüzdan alacak kaydı hatası: {e}"))?;

    // Ödenmemiş borçları bul ve otomatik tahsil et
    let mut borc_stmt = conn
        .prepare(
            "SELECT b.id, b.donem_id, b.hisse_sayisi, b.tutar, d.ay, d.yil
             FROM donem_aidat_borclari b
             JOIN donemler d ON d.id = b.donem_id
             WHERE b.hissedar_id = ?1 AND b.odendi = 0
             ORDER BY d.yil ASC, d.ay ASC",
        )
        .map_err(|e| e.to_string())?;

    struct OdenmemisBorc {
        id: i64,
        donem_id: i64,
        hisse_sayisi: i64,
        tutar: f64,
        ay: i64,
        yil: i64,
    }

    let odenmemis: Vec<OdenmemisBorc> = borc_stmt
        .query_map(params![input.hissedar_id], |row| {
            Ok(OdenmemisBorc {
                id: row.get(0)?,
                donem_id: row.get(1)?,
                hisse_sayisi: row.get(2)?,
                tutar: row.get(3)?,
                ay: row.get(4)?,
                yil: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // Hissedar bilgilerini al (kasa_id, ad, soyad, yakin_adi, yakinlik_derecesi)
    let (hissedar_ad, hissedar_soyad, kasa_id, yakin_adi, yakinlik_derecesi): (
        String,
        String,
        i64,
        Option<String>,
        Option<String>,
    ) = conn
        .query_row(
            "SELECT ad, soyad, kasa_id, yakin_adi, yakinlik_derecesi FROM hissedarlar WHERE id = ?1",
            params![input.hissedar_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .map_err(|e| format!("Hissedar bulunamadı: {e}"))?;

    let mut tahsil_sayisi: i64 = 0;
    let mut tahsil_toplam: f64 = 0.0;

    // Borçlar zaten cüzdana -tutar olarak yazılmış durumda (oluşturulduklarında).
    // Bu nedenle bakiye, tüm ödenmemiş borçların toplam yükünü taşıyor.
    // Harcanabilir nakit = bakiye + sum(ödenmemiş borç tutarları).
    // Bu miktar kadar borç ödenmesine izin verilir.
    let unpaid_iou_sum: f64 = odenmemis.iter().map(|b| b.tutar).sum();
    let bakiye_after_deposit = cuzdan_son_bakiye(&conn, input.hissedar_id);
    let mut nakit_var: f64 = bakiye_after_deposit + unpaid_iou_sum;

    // Küçük kayan-nokta toleransı
    const EPS: f64 = 0.005;

    for borc in &odenmemis {
        if nakit_var + EPS < borc.tutar {
            break; // Bu borcu ödemeye yetmiyor, kalan borçları atlat
        }

        // Borcu ödenmiş olarak işaretle
        conn.execute(
            "UPDATE donem_aidat_borclari SET odendi = 1, odeme_tarihi = ?1 WHERE id = ?2",
            params![&tarih, borc.id],
        )
        .map_err(|e| e.to_string())?;

        // Kasaya giren hareketi oluştur (detaylı açıklama)
        let donem_adi = format!("{} {}", ay_adi(borc.ay), borc.yil);
        let kasa_aciklama = tahsilat_aciklamasi(
            &donem_adi,
            borc.hisse_sayisi,
            &hissedar_ad,
            &hissedar_soyad,
            &yakin_adi,
            &yakinlik_derecesi,
        );

        let kasa_bak = kasa_son_bakiye(&conn, kasa_id);
        let kasa_yeni = kasa_bak + borc.tutar;

        conn.execute(
            "INSERT INTO kasa_hareketleri (kasa_id, tarih, aciklama, giren, cikan, bakiye)
             VALUES (?1, ?2, ?3, ?4, 0.0, ?5)",
            params![kasa_id, &tarih, kasa_aciklama, borc.tutar, kasa_yeni],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE kasalar SET bakiye = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![kasa_yeni, kasa_id],
        )
        .map_err(|e| e.to_string())?;

        // Cüzdana ek kayıt YAZILMIYOR: borç oluşturulduğunda IOU entry'si zaten
        // bakiyeden düşülmüş durumda. Bu sebeple ödeme, sadece donem_aidat_borclari
        // odendi flag'ini günceller ve kasaya giren yaratır. Cüzdan bakiyesi,
        // tam ödemede deposit ile IOU'ların birbirini nötrlemesi sonucu 0'da kalır.

        nakit_var -= borc.tutar;
        tahsil_sayisi += 1;
        tahsil_toplam += borc.tutar;
    }

    // ── Ödenmemiş hisse satın alma borçlarını tahsil et ───────────────────
    // Bu borçlar donem_aidat_borclari'nda olmayıp sadece cüzdan hareketlerinde
    // "Hisse satın alma:" bilgisiyle yazılı. Ödenmemiş kısmı:
    //   SUM(borc "Hisse satın alma:" prefix'li) − SUM(alacak "Hisse satın alma tahsilatı:" prefix'li)
    let hisse_borc_toplam: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(borc), 0) FROM hissedar_cuzdanlari
             WHERE hissedar_id = ?1
               AND borc > 0
               AND bilgi LIKE 'Hisse satın alma:%'",
            params![input.hissedar_id],
            |r| r.get(0),
        )
        .unwrap_or(0.0);
    let hisse_tahsil_toplam: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(alacak), 0) FROM hissedar_cuzdanlari
             WHERE hissedar_id = ?1
               AND alacak > 0
               AND bilgi LIKE 'Hisse satın alma tahsilatı:%'",
            params![input.hissedar_id],
            |r| r.get(0),
        )
        .unwrap_or(0.0);
    let odenmemis_hisse_borc = hisse_borc_toplam - hisse_tahsil_toplam;

    if odenmemis_hisse_borc > EPS && nakit_var + EPS >= odenmemis_hisse_borc {
        // Kasaya tek kayıt: toplam ödenmemiş hisse satın alma tutarı
        let kasa_aciklama = format!(
            "Hisse satın alma tahsilatı: {} {}",
            hissedar_ad, hissedar_soyad
        );
        let kasa_bak = kasa_son_bakiye(&conn, kasa_id);
        let kasa_yeni = kasa_bak + odenmemis_hisse_borc;

        conn.execute(
            "INSERT INTO kasa_hareketleri (kasa_id, tarih, aciklama, giren, cikan, bakiye)
             VALUES (?1, ?2, ?3, ?4, 0.0, ?5)",
            params![kasa_id, &tarih, kasa_aciklama, odenmemis_hisse_borc, kasa_yeni],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE kasalar SET bakiye = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![kasa_yeni, kasa_id],
        )
        .map_err(|e| e.to_string())?;

        // Cüzdana tek konsolide alacak kaydı (borçları nötrler)
        let c_bakiye = cuzdan_son_bakiye(&conn, input.hissedar_id);
        let c_yeni = c_bakiye + odenmemis_hisse_borc;
        let cuzdan_bilgi = "Hisse satın alma tahsilatı".to_string();
        conn.execute(
            "INSERT INTO hissedar_cuzdanlari (hissedar_id, donem_id, tarih, bilgi, borc, alacak, bakiye)
             VALUES (?1, NULL, ?2, ?3, 0.0, ?4, ?5)",
            params![input.hissedar_id, &tarih, cuzdan_bilgi, odenmemis_hisse_borc, c_yeni],
        )
        .map_err(|e| e.to_string())?;

        let _ = nakit_var; // harcanabilir nakit artık güncellendi (sonrası kullanılmıyor)
        tahsil_toplam += odenmemis_hisse_borc;
    }

    let son_bakiye = cuzdan_son_bakiye(&conn, input.hissedar_id);

    Ok(CuzdanParaEkleSonuc {
        yeni_bakiye: son_bakiye,
        tahsil_edilen_borc_sayisi: tahsil_sayisi,
        tahsil_edilen_toplam: tahsil_toplam,
    })
}

// ─── Yardımcı ────────────────────────────────────────────────────────────────

pub(crate) fn chrono_today() -> String {
    // SQLite uyumlu tarih formatı
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Basit tarih hesaplama (UTC) — yeterli doğrulukta
    let days = now / 86400;
    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let months_days = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 1;
    for &md in &months_days {
        if remaining < md {
            break;
        }
        remaining -= md;
        m += 1;
    }
    let d = remaining + 1;
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}
