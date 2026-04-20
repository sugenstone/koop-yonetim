use crate::aidat_commands::{ay_adi, chrono_today, cuzdan_son_bakiye, kasa_son_bakiye, tahsilat_aciklamasi};
use crate::db::Db;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

// ─── Modeller ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hisse {
    pub id: i64,
    pub kod: String,
    pub durum: String, // "musait" | "atanmis" | "satildi"
    pub aciklama: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub hissedar_id: Option<i64>,
    pub hissedar_ad: Option<String>,
    pub hissedar_soyad: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HisseAtama {
    pub id: i64,
    pub hisse_id: i64,
    pub hisse_kod: String,
    pub hissedar_id: i64,
    pub hissedar_ad: String,
    pub hissedar_soyad: String,
    pub tarih: String,
    pub ucret: f64,
    pub aciklama: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateHisseInput {
    pub aciklama: Option<String>,
    pub atama_hissedar_id: Option<i64>,
    pub atama_tarih: Option<String>,
    pub atama_ucret: Option<f64>,
    pub atama_aciklama: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateHisseTopluInput {
    pub adet: i64,
    pub aciklama: Option<String>,
    pub atama_hissedar_id: Option<i64>,
    pub atama_tarih: Option<String>,
    pub atama_ucret: Option<f64>,
    pub atama_aciklama: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AtamaInput {
    pub hisse_id: i64,
    pub hissedar_id: i64,
    pub tarih: String,
    pub ucret: f64,
    pub aciklama: Option<String>,
}

// ─── Yardımcı: Yeni hisse kodu üret ─────────────────────────────────────────

fn yeni_hisse_kodu(
    conn: &rusqlite::Connection,
    offset: i64,
) -> Result<String, String> {
    let son_no: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(CAST(SUBSTR(kod, 2) AS INTEGER)), 0) FROM hisseler",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    Ok(format!("H{:04}", son_no + 1 + offset))
}

// ─── Komutlar ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_hisseler(db: Db<'_>) -> Result<Vec<Hisse>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT h.id, h.kod, h.durum, h.aciklama, h.created_at, h.updated_at,
                    hs.id, hs.ad, hs.soyad
             FROM hisseler h
             LEFT JOIN (
                 SELECT ha.hisse_id, hsd.id, hsd.ad, hsd.soyad
                 FROM hisse_atamalari ha
                 JOIN hissedarlar hsd ON hsd.id = ha.hissedar_id
                 WHERE ha.id = (
                     SELECT id FROM hisse_atamalari
                     WHERE hisse_id = ha.hisse_id
                     ORDER BY created_at DESC LIMIT 1
                 )
             ) hs ON hs.hisse_id = h.id
             ORDER BY CAST(SUBSTR(h.kod, 2) AS INTEGER) ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Hisse {
                id: row.get(0)?,
                kod: row.get(1)?,
                durum: row.get(2)?,
                aciklama: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                hissedar_id: row.get(6)?,
                hissedar_ad: row.get(7)?,
                hissedar_soyad: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

#[tauri::command]
pub fn get_hisse(db: Db<'_>, id: i64) -> Result<Hisse, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, kod, durum, aciklama, created_at, updated_at FROM hisseler WHERE id = ?1",
        params![id],
        |row| {
            Ok(Hisse {
                id: row.get(0)?,
                kod: row.get(1)?,
                durum: row.get(2)?,
                aciklama: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                hissedar_id: None,
                hissedar_ad: None,
                hissedar_soyad: None,
            })
        },
    )
    .map_err(|e| format!("Hisse bulunamadı: {e}"))
}

#[tauri::command]
pub fn create_hisse(db: Db<'_>, input: CreateHisseInput) -> Result<Hisse, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let kod = yeni_hisse_kodu(&conn, 0)?;

    conn.execute(
        "INSERT INTO hisseler (kod, aciklama) VALUES (?1, ?2)",
        params![kod, input.aciklama],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    // Hissedar seçildiyse otomatik ata
    if let Some(hissedar_id) = input.atama_hissedar_id {
        let atama_input = AtamaInput {
            hisse_id: id,
            hissedar_id,
            tarih: input.atama_tarih.unwrap_or_else(|| chrono_today()),
            ucret: input.atama_ucret.unwrap_or(0.0),
            aciklama: input.atama_aciklama,
        };
        ata_hisse_conn(&conn, &atama_input)?;
    }

    conn.query_row(
        "SELECT h.id, h.kod, h.durum, h.aciklama, h.created_at, h.updated_at,
                hs.id, hs.ad, hs.soyad
         FROM hisseler h
         LEFT JOIN (
             SELECT ha.hisse_id, hsd.id, hsd.ad, hsd.soyad
             FROM hisse_atamalari ha
             JOIN hissedarlar hsd ON hsd.id = ha.hissedar_id
             WHERE ha.id = (
                 SELECT id FROM hisse_atamalari
                 WHERE hisse_id = ha.hisse_id
                 ORDER BY created_at DESC LIMIT 1
             )
         ) hs ON hs.hisse_id = h.id
         WHERE h.id = ?1",
        params![id],
        |row| {
            Ok(Hisse {
                id: row.get(0)?,
                kod: row.get(1)?,
                durum: row.get(2)?,
                aciklama: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                hissedar_id: row.get(6)?,
                hissedar_ad: row.get(7)?,
                hissedar_soyad: row.get(8)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_hisse_toplu(
    db: Db<'_>,
    input: CreateHisseTopluInput,
) -> Result<Vec<Hisse>, String> {
    if input.adet <= 0 {
        return Err("Adet sıfırdan büyük olmalıdır".to_string());
    }
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // 1) Önce tüm hisseleri oluştur, ID'lerini topla
    let mut olusturulan_idler: Vec<i64> = Vec::with_capacity(input.adet as usize);
    for i in 0..input.adet {
        let kod = yeni_hisse_kodu(&conn, i)?;
        conn.execute(
            "INSERT INTO hisseler (kod, aciklama) VALUES (?1, ?2)",
            params![kod, input.aciklama],
        )
        .map_err(|e| format!("Hisse oluşturma hatası ({kod}): {e}"))?;
        olusturulan_idler.push(conn.last_insert_rowid());
    }

    // 2) Hissedar seçildiyse TEK toplu atama çağrısı (konsolide cüzdan kaydı)
    if let Some(hissedar_id) = input.atama_hissedar_id {
        let tarih = input.atama_tarih.unwrap_or_else(|| chrono_today());
        let ucret = input.atama_ucret.unwrap_or(0.0);
        ata_hisseler_toplu_conn(
            &conn,
            hissedar_id,
            &olusturulan_idler,
            &tarih,
            ucret,
            input.atama_aciklama.as_deref(),
        )?;
    }

    // 3) Oluşturulan hisseleri (atama bilgisiyle) dön
    let mut olusturulanlar: Vec<Hisse> = Vec::with_capacity(olusturulan_idler.len());
    for id in &olusturulan_idler {
        let hisse = conn
            .query_row(
                "SELECT h.id, h.kod, h.durum, h.aciklama, h.created_at, h.updated_at,
                        hs.id, hs.ad, hs.soyad
                 FROM hisseler h
                 LEFT JOIN (
                     SELECT ha.hisse_id, hsd.id, hsd.ad, hsd.soyad
                     FROM hisse_atamalari ha
                     JOIN hissedarlar hsd ON hsd.id = ha.hissedar_id
                     WHERE ha.id = (
                         SELECT id FROM hisse_atamalari
                         WHERE hisse_id = ha.hisse_id
                         ORDER BY created_at DESC LIMIT 1
                     )
                 ) hs ON hs.hisse_id = h.id
                 WHERE h.id = ?1",
                params![id],
                |row| {
                    Ok(Hisse {
                        id: row.get(0)?,
                        kod: row.get(1)?,
                        durum: row.get(2)?,
                        aciklama: row.get(3)?,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                        hissedar_id: row.get(6)?,
                        hissedar_ad: row.get(7)?,
                        hissedar_soyad: row.get(8)?,
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        olusturulanlar.push(hisse);
    }

    Ok(olusturulanlar)
}

#[tauri::command]
pub fn delete_hisse(db: Db<'_>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Atanmış hisse silinemez
    let atama_sayisi: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM hisse_atamalari WHERE hisse_id = ?1",
            params![id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if atama_sayisi > 0 {
        return Err("Bu hisse bir hissedara atanmış, silinemez".to_string());
    }

    conn.execute("DELETE FROM hisseler WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ─── Atama Komutları ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_hisse_atamalari(db: Db<'_>, hisse_id: i64) -> Result<Vec<HisseAtama>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.hisse_id, h.kod, a.hissedar_id,
                    hs.ad, hs.soyad, a.tarih, a.ucret, a.aciklama, a.created_at
             FROM hisse_atamalari a
             JOIN hisseler h ON h.id = a.hisse_id
             JOIN hissedarlar hs ON hs.id = a.hissedar_id
             WHERE a.hisse_id = ?1
             ORDER BY a.tarih DESC, a.id DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![hisse_id], |row| {
            Ok(HisseAtama {
                id: row.get(0)?,
                hisse_id: row.get(1)?,
                hisse_kod: row.get(2)?,
                hissedar_id: row.get(3)?,
                hissedar_ad: row.get(4)?,
                hissedar_soyad: row.get(5)?,
                tarih: row.get(6)?,
                ucret: row.get(7)?,
                aciklama: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

#[tauri::command]
pub fn get_hissedar_atamalari(db: Db<'_>, hissedar_id: i64) -> Result<Vec<HisseAtama>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.hisse_id, h.kod, a.hissedar_id,
                    hs.ad, hs.soyad, a.tarih, a.ucret, a.aciklama, a.created_at
             FROM hisse_atamalari a
             JOIN hisseler h ON h.id = a.hisse_id
             JOIN hissedarlar hs ON hs.id = a.hissedar_id
             WHERE a.hissedar_id = ?1
             ORDER BY a.tarih DESC, a.id DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![hissedar_id], |row| {
            Ok(HisseAtama {
                id: row.get(0)?,
                hisse_id: row.get(1)?,
                hisse_kod: row.get(2)?,
                hissedar_id: row.get(3)?,
                hissedar_ad: row.get(4)?,
                hissedar_soyad: row.get(5)?,
                tarih: row.get(6)?,
                ucret: row.get(7)?,
                aciklama: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

fn ata_hisse_conn(conn: &rusqlite::Connection, input: &AtamaInput) -> Result<HisseAtama, String> {
    if input.ucret < 0.0 {
        return Err("Ücret negatif olamaz".to_string());
    }

    // Hissedarın bilgilerini al
    let (hissedar_kasa_id, hissedar_ad, hissedar_soyad, yakin_adi, yakinlik_derecesi): (
        i64,
        String,
        String,
        Option<String>,
        Option<String>,
    ) = conn
        .query_row(
            "SELECT kasa_id, ad, soyad, yakin_adi, yakinlik_derecesi FROM hissedarlar WHERE id = ?1",
            params![input.hissedar_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .map_err(|_| "Hissedar bulunamadı".to_string())?;

    // Hisse kodunu al
    let hisse_kod: String = conn
        .query_row(
            "SELECT kod FROM hisseler WHERE id = ?1",
            params![input.hisse_id],
            |r| r.get(0),
        )
        .map_err(|_| "Hisse bulunamadı".to_string())?;

    // Hisse durumunu kontrol et — sadece musait hisseler atanabilir
    let mevcut_durum: String = conn
        .query_row(
            "SELECT durum FROM hisseler WHERE id = ?1",
            params![input.hisse_id],
            |r| r.get(0),
        )
        .map_err(|_| "Hisse bulunamadı".to_string())?;
    match mevcut_durum.as_str() {
        "atanmis" => {
            return Err(
                "Bu hisse zaten bir hissedara atanmış. Başka bir hissedara aktarmak için \"Transfer Et\" kullanın."
                    .to_string(),
            )
        }
        "satildi" => {
            return Err("Bu hisse sisteme satılmış ve artık kullanılamaz.".to_string())
        }
        _ => {}
    }

    // Atamayı kaydet
    conn.execute(
        "INSERT INTO hisse_atamalari (hisse_id, hissedar_id, tarih, ucret, aciklama)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            input.hisse_id,
            input.hissedar_id,
            input.tarih,
            input.ucret,
            input.aciklama
        ],
    )
    .map_err(|e| e.to_string())?;

    let atama_id = conn.last_insert_rowid();

    // Hisseyi "atanmis" olarak işaretle
    conn.execute(
        "UPDATE hisseler SET durum = 'atanmis', updated_at = datetime('now') WHERE id = ?1",
        params![input.hisse_id],
    )
    .map_err(|e| e.to_string())?;

    // Ücret varsa: sadece cüzdana borç kaydı yaz (kasa ile doğrudan ilgisi yok)
    // Cüzdan bakiyesi yeterliyse otomatik tahsilat → kasaya giren olarak yansır
    if input.ucret > 0.0 {
        let onceki_cuzdan = cuzdan_son_bakiye(&conn, input.hissedar_id);
        let yeni_cuzdan = onceki_cuzdan - input.ucret;
        let cuzdan_bilgi = format!("Hisse satın alma: {}", hisse_kod);

        // Cüzdana borç kaydı
        conn.execute(
            "INSERT INTO hissedar_cuzdanlari (hissedar_id, donem_id, tarih, bilgi, borc, alacak, bakiye)
             VALUES (?1, NULL, ?2, ?3, ?4, 0.0, ?5)",
            params![
                input.hissedar_id,
                input.tarih,
                cuzdan_bilgi,
                input.ucret,
                yeni_cuzdan
            ],
        )
        .map_err(|e| e.to_string())?;

        // Cüzdan bakiyesi yeterliyse otomatik tahsilat kasaya giren olarak işlenir
        if onceki_cuzdan >= input.ucret {
            let kasa_aciklama = format!(
                "Hisse satın alma tahsilatı: {} - {} {}",
                hisse_kod, hissedar_ad, hissedar_soyad
            );
            let kasa_onceki = kasa_son_bakiye(&conn, hissedar_kasa_id);
            let kasa_yeni = kasa_onceki + input.ucret;

            conn.execute(
                "INSERT INTO kasa_hareketleri (kasa_id, tarih, aciklama, giren, cikan, bakiye)
                 VALUES (?1, ?2, ?3, ?4, 0.0, ?5)",
                params![hissedar_kasa_id, input.tarih, kasa_aciklama, input.ucret, kasa_yeni],
            )
            .map_err(|e| e.to_string())?;

            conn.execute(
                "UPDATE kasalar SET bakiye = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![kasa_yeni, hissedar_kasa_id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // ── Geçmiş dönemler için retroaktif borç oluştur ──────────────────────────
    // En güncel dönemin aidat tutarını al (adalet gereği tüm geçmiş dönemler bu tutarla hesaplanır)
    #[derive(Debug)]
    struct DonemBilgi {
        id: i64,
        ay: i64,
        yil: i64,
    }

    let en_guncel_aidat: Option<f64> = conn
        .query_row(
            "SELECT hisse_basi_aidat FROM donemler ORDER BY yil DESC, ay DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()
        .unwrap_or(None);

    if let Some(aidat_tutari) = en_guncel_aidat {
        if aidat_tutari > 0.0 {
            let mut donem_stmt = conn
                .prepare("SELECT id, ay, yil FROM donemler ORDER BY yil ASC, ay ASC")
                .map_err(|e| e.to_string())?;

            let donemler: Vec<DonemBilgi> = donem_stmt
                .query_map([], |row| {
                    Ok(DonemBilgi {
                        id: row.get(0)?,
                        ay: row.get(1)?,
                        yil: row.get(2)?,
                    })
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            let tarih_retro = chrono_today();

            for donem in &donemler {
                let donem_adi = format!("{} {}", ay_adi(donem.ay), donem.yil);

                // Cüzdan bakiyesi
                let cuzdan_onceki = cuzdan_son_bakiye(&conn, input.hissedar_id);

                // Bu dönem + hissedar için ödenmemiş borç var mı?
                let mevcut_odenmemis: Option<(i64, f64, i64)> = conn
                    .query_row(
                        "SELECT id, tutar, hisse_sayisi FROM donem_aidat_borclari
                         WHERE donem_id = ?1 AND hissedar_id = ?2 AND odendi = 0
                         ORDER BY id DESC LIMIT 1",
                        params![donem.id, input.hissedar_id],
                        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                    )
                    .optional()
                    .unwrap_or(None);

                if let Some((borc_id, mevcut_tutar, mevcut_hisse_sayisi)) = mevcut_odenmemis {
                    // Ödenmemiş kayıt var → güncelle
                    let yeni_tutar = mevcut_tutar + aidat_tutari;
                    let yeni_hisse_sayisi = mevcut_hisse_sayisi + 1;
                    let yeterli = cuzdan_onceki >= yeni_tutar;

                    let borc_aciklama = format!(
                        "{} aidatı - {} hisse ({} {}) [Geçmiş dönem - güncel tarife]",
                        donem_adi, yeni_hisse_sayisi, hissedar_ad, hissedar_soyad
                    );

                    conn.execute(
                        "UPDATE donem_aidat_borclari
                         SET tutar = ?1, hisse_sayisi = ?2, aciklama = ?3,
                             odendi = ?4, odeme_tarihi = ?5
                         WHERE id = ?6",
                        params![
                            yeni_tutar,
                            yeni_hisse_sayisi,
                            borc_aciklama,
                            yeterli as i64,
                            if yeterli { Some(&tarih_retro) } else { None },
                            borc_id,
                        ],
                    )
                    .map_err(|e| format!("Borç güncelleme hatası: {e}"))?;

                    // Cüzdana delta borç kaydı (yeni hissenin aidatı anında cüzdana yansır)
                    let cuzdan_borc_bilgi = format!("{} aidatı - {} hisse [geçmiş dönem]", donem_adi, hisse_kod);
                    let cb_onceki = cuzdan_son_bakiye(&conn, input.hissedar_id);
                    let cb_yeni = cb_onceki - aidat_tutari;
                    conn.execute(
                        "INSERT INTO hissedar_cuzdanlari
                             (hissedar_id, donem_id, tarih, bilgi, borc, alacak, bakiye)
                         VALUES (?1, ?2, ?3, ?4, ?5, 0.0, ?6)",
                        params![input.hissedar_id, donem.id, &tarih_retro, cuzdan_borc_bilgi, aidat_tutari, cb_yeni],
                    )
                    .map_err(|e| format!("Cüzdan borç kaydı hatası: {e}"))?;

                    // Otomatik tahsilat: tam tutar ödenebilirse kasaya aktar
                    if yeterli {
                        let kasa_aciklama = tahsilat_aciklamasi(
                            &donem_adi,
                            yeni_hisse_sayisi,
                            &hissedar_ad,
                            &hissedar_soyad,
                            &yakin_adi,
                            &yakinlik_derecesi,
                        );
                        let kasa_onceki = kasa_son_bakiye(&conn, hissedar_kasa_id);
                        let kasa_yeni = kasa_onceki + yeni_tutar;

                        conn.execute(
                            "INSERT INTO kasa_hareketleri
                                 (kasa_id, tarih, aciklama, giren, cikan, bakiye)
                             VALUES (?1, ?2, ?3, ?4, 0.0, ?5)",
                            params![hissedar_kasa_id, &tarih_retro, kasa_aciklama, yeni_tutar, kasa_yeni],
                        )
                        .map_err(|e| e.to_string())?;

                        conn.execute(
                            "UPDATE kasalar SET bakiye = ?1, updated_at = datetime('now') WHERE id = ?2",
                            params![kasa_yeni, hissedar_kasa_id],
                        )
                        .map_err(|e| e.to_string())?;

                        // Cüzdana alacak kaydı (nakit kasaya geçti, borcu mahsup eder)
                        let cuzdan_tahsil_bilgi = format!("Tahsilat: {} - {} hisse", donem_adi, yeni_hisse_sayisi);
                        let ct_onceki = cuzdan_son_bakiye(&conn, input.hissedar_id);
                        let ct_yeni = ct_onceki + yeni_tutar;
                        conn.execute(
                            "INSERT INTO hissedar_cuzdanlari
                                 (hissedar_id, donem_id, tarih, bilgi, borc, alacak, bakiye)
                             VALUES (?1, ?2, ?3, ?4, 0.0, ?5, ?6)",
                            params![input.hissedar_id, donem.id, &tarih_retro, cuzdan_tahsil_bilgi, yeni_tutar, ct_yeni],
                        )
                        .map_err(|e| format!("Cüzdan tahsilat kaydı hatası: {e}"))?;
                    }
                } else {
                    // Ödenmemiş kayıt yok → yeni kayıt oluştur
                    let yeterli = cuzdan_onceki >= aidat_tutari;

                    let borc_aciklama = format!(
                        "{} aidatı - 1 hisse ({} {}) [Geçmiş dönem - güncel tarife]",
                        donem_adi, hissedar_ad, hissedar_soyad
                    );

                    conn.execute(
                        "INSERT INTO donem_aidat_borclari
                             (donem_id, hissedar_id, hisse_sayisi, tutar, odendi, odeme_tarihi, aciklama)
                         VALUES (?1, ?2, 1, ?3, ?4, ?5, ?6)",
                        params![
                            donem.id,
                            input.hissedar_id,
                            aidat_tutari,
                            yeterli as i64,
                            if yeterli { Some(&tarih_retro) } else { None },
                            borc_aciklama,
                        ],
                    )
                    .map_err(|e| format!("Geçmiş dönem borç kaydı hatası: {e}"))?;

                    // Cüzdana borç kaydı (anında yansır)
                    let cuzdan_borc_bilgi2 = format!("{} aidatı - {} [geçmiş dönem]", donem_adi, hisse_kod);
                    let cb2_onceki = cuzdan_son_bakiye(&conn, input.hissedar_id);
                    let cb2_yeni = cb2_onceki - aidat_tutari;
                    conn.execute(
                        "INSERT INTO hissedar_cuzdanlari
                             (hissedar_id, donem_id, tarih, bilgi, borc, alacak, bakiye)
                         VALUES (?1, ?2, ?3, ?4, ?5, 0.0, ?6)",
                        params![input.hissedar_id, donem.id, &tarih_retro, cuzdan_borc_bilgi2, aidat_tutari, cb2_yeni],
                    )
                    .map_err(|e| format!("Cüzdan borç kaydı hatası: {e}"))?;

                    // Otomatik tahsilat
                    if yeterli {
                        let kasa_aciklama = tahsilat_aciklamasi(
                            &donem_adi,
                            1i64,
                            &hissedar_ad,
                            &hissedar_soyad,
                            &yakin_adi,
                            &yakinlik_derecesi,
                        );
                        let kasa_onceki = kasa_son_bakiye(&conn, hissedar_kasa_id);
                        let kasa_yeni = kasa_onceki + aidat_tutari;

                        conn.execute(
                            "INSERT INTO kasa_hareketleri
                                 (kasa_id, tarih, aciklama, giren, cikan, bakiye)
                             VALUES (?1, ?2, ?3, ?4, 0.0, ?5)",
                            params![hissedar_kasa_id, &tarih_retro, kasa_aciklama, aidat_tutari, kasa_yeni],
                        )
                        .map_err(|e| e.to_string())?;

                        conn.execute(
                            "UPDATE kasalar SET bakiye = ?1, updated_at = datetime('now') WHERE id = ?2",
                            params![kasa_yeni, hissedar_kasa_id],
                        )
                        .map_err(|e| e.to_string())?;

                        // Cüzdana alacak kaydı (borcu mahsup eder)
                        let cuzdan_tahsil_bilgi2 = format!("Tahsilat: {} - 1 hisse", donem_adi);
                        let ct2_onceki = cuzdan_son_bakiye(&conn, input.hissedar_id);
                        let ct2_yeni = ct2_onceki + aidat_tutari;
                        conn.execute(
                            "INSERT INTO hissedar_cuzdanlari
                                 (hissedar_id, donem_id, tarih, bilgi, borc, alacak, bakiye)
                             VALUES (?1, ?2, ?3, ?4, 0.0, ?5, ?6)",
                            params![input.hissedar_id, donem.id, &tarih_retro, cuzdan_tahsil_bilgi2, aidat_tutari, ct2_yeni],
                        )
                        .map_err(|e| format!("Cüzdan tahsilat kaydı hatası: {e}"))?;
                    }
                }
            }
        }
    }

    // Dönüş için atama + join bilgisini çek
    conn.query_row(
        "SELECT a.id, a.hisse_id, h.kod, a.hissedar_id,
                hs.ad, hs.soyad, a.tarih, a.ucret, a.aciklama, a.created_at
         FROM hisse_atamalari a
         JOIN hisseler h ON h.id = a.hisse_id
         JOIN hissedarlar hs ON hs.id = a.hissedar_id
         WHERE a.id = ?1",
        params![atama_id],
        |row| {
            Ok(HisseAtama {
                id: row.get(0)?,
                hisse_id: row.get(1)?,
                hisse_kod: row.get(2)?,
                hissedar_id: row.get(3)?,
                hissedar_ad: row.get(4)?,
                hissedar_soyad: row.get(5)?,
                tarih: row.get(6)?,
                ucret: row.get(7)?,
                aciklama: row.get(8)?,
                created_at: row.get(9)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

/// Birden fazla hisseyi aynı hissedara toplu olarak atar.
/// Cüzdan/kasa kayıtları tek seferlik (konsolide) olarak oluşturulur.
fn ata_hisseler_toplu_conn(
    conn: &rusqlite::Connection,
    hissedar_id: i64,
    hisse_idler: &[i64],
    tarih: &str,
    ucret_per_hisse: f64,
    aciklama: Option<&str>,
) -> Result<(), String> {
    if hisse_idler.is_empty() {
        return Ok(());
    }
    if ucret_per_hisse < 0.0 {
        return Err("Ücret negatif olamaz".to_string());
    }

    let adet = hisse_idler.len() as i64;
    let toplam_ucret = ucret_per_hisse * adet as f64;

    // Hissedar bilgileri
    let (hissedar_kasa_id, hissedar_ad, hissedar_soyad, yakin_adi, yakinlik_derecesi): (
        i64,
        String,
        String,
        Option<String>,
        Option<String>,
    ) = conn
        .query_row(
            "SELECT kasa_id, ad, soyad, yakin_adi, yakinlik_derecesi FROM hissedarlar WHERE id = ?1",
            params![hissedar_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .map_err(|_| "Hissedar bulunamadı".to_string())?;

    // Tüm hisse kodlarını topla + atamaları oluştur + hisseleri atanmış yap
    let mut hisse_kodlari: Vec<String> = Vec::with_capacity(hisse_idler.len());
    for &hisse_id in hisse_idler {
        let hisse_kod: String = conn
            .query_row(
                "SELECT kod FROM hisseler WHERE id = ?1",
                params![hisse_id],
                |r| r.get(0),
            )
            .map_err(|_| format!("Hisse bulunamadı: {hisse_id}"))?;
        hisse_kodlari.push(hisse_kod);

        conn.execute(
            "INSERT INTO hisse_atamalari (hisse_id, hissedar_id, tarih, ucret, aciklama)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![hisse_id, hissedar_id, tarih, ucret_per_hisse, aciklama],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE hisseler SET durum = 'atanmis', updated_at = datetime('now') WHERE id = ?1",
            params![hisse_id],
        )
        .map_err(|e| e.to_string())?;
    }

    let kodlar_str = hisse_kodlari.join(", ");

    // ── Ücret varsa: cüzdana TEK konsolide borç kaydı ───────────────────────
    if toplam_ucret > 0.0 {
        let onceki_cuzdan = cuzdan_son_bakiye(conn, hissedar_id);
        let yeni_cuzdan = onceki_cuzdan - toplam_ucret;
        let cuzdan_bilgi = format!("Hisse satın alma: {} ({} hisse)", kodlar_str, adet);

        conn.execute(
            "INSERT INTO hissedar_cuzdanlari (hissedar_id, donem_id, tarih, bilgi, borc, alacak, bakiye)
             VALUES (?1, NULL, ?2, ?3, ?4, 0.0, ?5)",
            params![hissedar_id, tarih, cuzdan_bilgi, toplam_ucret, yeni_cuzdan],
        )
        .map_err(|e| e.to_string())?;

        // Cüzdan bakiyesi yeterliyse otomatik tahsilat (tek kasa kaydı)
        if onceki_cuzdan >= toplam_ucret {
            let kasa_aciklama = format!(
                "Hisse satın alma tahsilatı: {} ({} hisse) - {} {}",
                kodlar_str, adet, hissedar_ad, hissedar_soyad
            );
            let kasa_onceki = kasa_son_bakiye(conn, hissedar_kasa_id);
            let kasa_yeni = kasa_onceki + toplam_ucret;

            conn.execute(
                "INSERT INTO kasa_hareketleri (kasa_id, tarih, aciklama, giren, cikan, bakiye)
                 VALUES (?1, ?2, ?3, ?4, 0.0, ?5)",
                params![hissedar_kasa_id, tarih, kasa_aciklama, toplam_ucret, kasa_yeni],
            )
            .map_err(|e| e.to_string())?;

            conn.execute(
                "UPDATE kasalar SET bakiye = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![kasa_yeni, hissedar_kasa_id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // ── Retroaktif geçmiş dönem borçları (her dönem için TEK konsolide kayıt) ─
    let en_guncel_aidat: Option<f64> = conn
        .query_row(
            "SELECT hisse_basi_aidat FROM donemler ORDER BY yil DESC, ay DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()
        .unwrap_or(None);

    if let Some(aidat_tutari) = en_guncel_aidat {
        if aidat_tutari > 0.0 {
            struct DonemBilgi {
                id: i64,
                ay: i64,
                yil: i64,
            }

            let mut donem_stmt = conn
                .prepare("SELECT id, ay, yil FROM donemler ORDER BY yil ASC, ay ASC")
                .map_err(|e| e.to_string())?;

            let donemler: Vec<DonemBilgi> = donem_stmt
                .query_map([], |row| {
                    Ok(DonemBilgi {
                        id: row.get(0)?,
                        ay: row.get(1)?,
                        yil: row.get(2)?,
                    })
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            let tarih_retro = chrono_today();
            let donem_toplam_tutar = aidat_tutari * adet as f64;

            for donem in &donemler {
                let donem_adi = format!("{} {}", ay_adi(donem.ay), donem.yil);

                // Bu dönem + hissedar için ödenmemiş borç var mı?
                let mevcut_odenmemis: Option<(i64, f64, i64)> = conn
                    .query_row(
                        "SELECT id, tutar, hisse_sayisi FROM donem_aidat_borclari
                         WHERE donem_id = ?1 AND hissedar_id = ?2 AND odendi = 0
                         ORDER BY id DESC LIMIT 1",
                        params![donem.id, hissedar_id],
                        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                    )
                    .optional()
                    .unwrap_or(None);

                let (yeni_tutar, yeni_hisse_sayisi) = if let Some((borc_id, mevcut_tutar, mevcut_hisse_sayisi)) = mevcut_odenmemis {
                    // Güncelle: mevcut + eklenen (adet kadar)
                    let yt = mevcut_tutar + donem_toplam_tutar;
                    let yhs = mevcut_hisse_sayisi + adet;

                    let borc_aciklama = format!(
                        "{} aidatı - {} hisse ({} {}) [Geçmiş dönem - güncel tarife]",
                        donem_adi, yhs, hissedar_ad, hissedar_soyad
                    );
                    conn.execute(
                        "UPDATE donem_aidat_borclari
                         SET tutar = ?1, hisse_sayisi = ?2, aciklama = ?3
                         WHERE id = ?4",
                        params![yt, yhs, borc_aciklama, borc_id],
                    )
                    .map_err(|e| format!("Borç güncelleme hatası: {e}"))?;
                    (yt, yhs)
                } else {
                    // Yeni kayıt: adet hisse ile (ödenmemiş olarak oluştur, manuel tahsilat bekler)
                    let borc_aciklama = format!(
                        "{} aidatı - {} hisse ({} {}) [Geçmiş dönem - güncel tarife]",
                        donem_adi, adet, hissedar_ad, hissedar_soyad
                    );
                    conn.execute(
                        "INSERT INTO donem_aidat_borclari
                             (donem_id, hissedar_id, hisse_sayisi, tutar, odendi, odeme_tarihi, aciklama)
                         VALUES (?1, ?2, ?3, ?4, 0, NULL, ?5)",
                        params![donem.id, hissedar_id, adet, donem_toplam_tutar, borc_aciklama],
                    )
                    .map_err(|e| format!("Geçmiş dönem borç kaydı hatası: {e}"))?;
                    (donem_toplam_tutar, adet)
                };

                // Cüzdana TEK konsolide borç kaydı (bu atama için eklenen tutar = donem_toplam_tutar)
                let cuzdan_borc_bilgi = format!(
                    "{} aidatı - {} hisse [geçmiş dönem]",
                    donem_adi, adet
                );
                let cb_onceki = cuzdan_son_bakiye(conn, hissedar_id);
                let cb_yeni = cb_onceki - donem_toplam_tutar;
                conn.execute(
                    "INSERT INTO hissedar_cuzdanlari
                         (hissedar_id, donem_id, tarih, bilgi, borc, alacak, bakiye)
                     VALUES (?1, ?2, ?3, ?4, ?5, 0.0, ?6)",
                    params![hissedar_id, donem.id, &tarih_retro, cuzdan_borc_bilgi, donem_toplam_tutar, cb_yeni],
                )
                .map_err(|e| format!("Cüzdan borç kaydı hatası: {e}"))?;

                let _ = (&yakin_adi, &yakinlik_derecesi, &hissedar_kasa_id, &yeni_tutar, &yeni_hisse_sayisi);
            }
        }
    }

    Ok(())
}

#[tauri::command]pub fn hisse_ata(db: Db<'_>, input: AtamaInput) -> Result<HisseAtama, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    ata_hisse_conn(&conn, &input)
}

#[tauri::command]pub fn hisse_atama_sil(db: Db<'_>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Atama bilgisini al (hisse_id için)
    let hisse_id: Option<i64> = conn
        .query_row(
            "SELECT hisse_id FROM hisse_atamalari WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let hisse_id = hisse_id.ok_or("Atama bulunamadı")?;

    conn.execute(
        "DELETE FROM hisse_atamalari WHERE id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    // Eğer bu hissenin başka ataması kalmadıysa "musait" yap
    let kalan: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM hisse_atamalari WHERE hisse_id = ?1",
            params![hisse_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if kalan == 0 {
        conn.execute(
            "UPDATE hisseler SET durum = 'musait', updated_at = datetime('now') WHERE id = ?1",
            params![hisse_id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

// ─── Hisse Transferi ────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct HisseTransferInput {
    pub hisse_id: i64,
    pub yeni_hissedar_id: i64,
    pub tarih: String,
    pub ucret: f64,
    pub aciklama: Option<String>,
}

/// Hisseyi bir hissedardan başka bir hissedara transfer eder.
/// Eski atama kayıtları silinmez; yalnızca yeni bir atama kaydı eklenir
/// (en güncel kayıt mevcut sahibi belirler). Ücret 0 olabilir.
/// Ücret > 0 ise: alıcıya borç, satıcıya alacak cüzdan kaydı oluşturulur ve
/// alıcı cüzdanı yeterliyse kasalar arası transfer ile dengelenir.
#[tauri::command]
pub fn hisse_transfer(db: Db<'_>, input: HisseTransferInput) -> Result<HisseAtama, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    if input.ucret < 0.0 {
        return Err("Transfer ücreti negatif olamaz".to_string());
    }

    // Hisseyi doğrula + kodunu al
    let (hisse_kod, hisse_durum): (String, String) = conn
        .query_row(
            "SELECT kod, durum FROM hisseler WHERE id = ?1",
            params![input.hisse_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|_| "Hisse bulunamadı".to_string())?;

    if hisse_durum == "satildi" {
        return Err("Bu hisse sisteme satılmış ve artık kullanılamaz.".to_string());
    }

    // Aktif (tamamlanmamış, iptal edilmemiş) satış varsa transfer engellenir
    let aktif_satis: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM hisse_satislari
             WHERE hisse_id = ?1 AND tamamlandi = 0 AND iptal = 0",
            params![input.hisse_id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if aktif_satis > 0 {
        return Err(
            "Bu hisse için devam eden bir satış süreci var. Transfer yapılamaz; önce satışı tamamlayın veya iptal edin."
                .to_string(),
        );
    }

    // Mevcut sahibi bul (en son atama)
    let mevcut: Option<(i64, String, String, i64)> = conn
        .query_row(
            "SELECT hsd.id, hsd.ad, hsd.soyad, hsd.kasa_id
             FROM hisse_atamalari ha
             JOIN hissedarlar hsd ON hsd.id = ha.hissedar_id
             WHERE ha.hisse_id = ?1
             ORDER BY ha.created_at DESC, ha.id DESC
             LIMIT 1",
            params![input.hisse_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let (eski_hissedar_id, eski_ad, eski_soyad, eski_kasa_id) =
        mevcut.ok_or("Bu hisse henüz kimseye atanmamış, transfer için önce atama yapın")?;

    if eski_hissedar_id == input.yeni_hissedar_id {
        return Err("Hisse zaten bu hissedarda, transfer yapılamaz".to_string());
    }

    // Yeni hissedar bilgileri
    let (yeni_kasa_id, yeni_ad, yeni_soyad): (i64, String, String) = conn
        .query_row(
            "SELECT kasa_id, ad, soyad FROM hissedarlar WHERE id = ?1",
            params![input.yeni_hissedar_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .map_err(|_| "Hedef hissedar bulunamadı".to_string())?;

    // Atama açıklaması
    let atama_aciklama = match input.aciklama.as_deref() {
        Some(a) if !a.trim().is_empty() => format!(
            "Transfer: {} {} → {} {} | {}",
            eski_ad, eski_soyad, yeni_ad, yeni_soyad, a
        ),
        _ => format!(
            "Transfer: {} {} → {} {}",
            eski_ad, eski_soyad, yeni_ad, yeni_soyad
        ),
    };

    // Yeni atama kaydı (geçmiş korunur)
    conn.execute(
        "INSERT INTO hisse_atamalari (hisse_id, hissedar_id, tarih, ucret, aciklama)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            input.hisse_id,
            input.yeni_hissedar_id,
            input.tarih,
            input.ucret,
            atama_aciklama
        ],
    )
    .map_err(|e| e.to_string())?;

    let atama_id = conn.last_insert_rowid();

    // Hisse zaten 'atanmis' olmalı, yine de güncelle (kod tutarlılığı için)
    conn.execute(
        "UPDATE hisseler SET durum = 'atanmis', updated_at = datetime('now') WHERE id = ?1",
        params![input.hisse_id],
    )
    .map_err(|e| e.to_string())?;

    // ── Ücret > 0 ise cüzdan ve kasa hareketleri ──────────────────────────
    if input.ucret > 0.0 {
        let cuzdan_bilgi_alici = format!(
            "Hisse transferi (alım): {} - {} {}",
            hisse_kod, eski_ad, eski_soyad
        );
        let cuzdan_bilgi_satici = format!(
            "Hisse transferi (satım): {} - {} {}",
            hisse_kod, yeni_ad, yeni_soyad
        );

        // Alıcı cüzdanı: borç
        let alici_cuzdan_onceki = cuzdan_son_bakiye(&conn, input.yeni_hissedar_id);
        let alici_cuzdan_yeni = alici_cuzdan_onceki - input.ucret;
        conn.execute(
            "INSERT INTO hissedar_cuzdanlari
                 (hissedar_id, donem_id, tarih, bilgi, borc, alacak, bakiye)
             VALUES (?1, NULL, ?2, ?3, ?4, 0.0, ?5)",
            params![
                input.yeni_hissedar_id,
                input.tarih,
                cuzdan_bilgi_alici,
                input.ucret,
                alici_cuzdan_yeni
            ],
        )
        .map_err(|e| e.to_string())?;

        // Satıcı cüzdanı: alacak
        let satici_cuzdan_onceki = cuzdan_son_bakiye(&conn, eski_hissedar_id);
        let satici_cuzdan_yeni = satici_cuzdan_onceki + input.ucret;
        conn.execute(
            "INSERT INTO hissedar_cuzdanlari
                 (hissedar_id, donem_id, tarih, bilgi, borc, alacak, bakiye)
             VALUES (?1, NULL, ?2, ?3, 0.0, ?4, ?5)",
            params![
                eski_hissedar_id,
                input.tarih,
                cuzdan_bilgi_satici,
                input.ucret,
                satici_cuzdan_yeni
            ],
        )
        .map_err(|e| e.to_string())?;

        // Alıcı cüzdanı yeterliyse: alıcı kasasından çık, satıcı kasasına gir
        if alici_cuzdan_onceki >= input.ucret {
            let alici_kasa_aciklama = format!(
                "Hisse transfer ödemesi: {} → {} {} ({})",
                hisse_kod, eski_ad, eski_soyad, yeni_ad
            );
            let satici_kasa_aciklama = format!(
                "Hisse transfer tahsilatı: {} ← {} {} ({})",
                hisse_kod, yeni_ad, yeni_soyad, eski_ad
            );

            // Alıcı kasasından çıkış
            let alici_kasa_onceki = kasa_son_bakiye(&conn, yeni_kasa_id);
            let alici_kasa_yeni = alici_kasa_onceki - input.ucret;
            conn.execute(
                "INSERT INTO kasa_hareketleri (kasa_id, tarih, aciklama, giren, cikan, bakiye)
                 VALUES (?1, ?2, ?3, 0.0, ?4, ?5)",
                params![
                    yeni_kasa_id,
                    input.tarih,
                    alici_kasa_aciklama,
                    input.ucret,
                    alici_kasa_yeni
                ],
            )
            .map_err(|e| e.to_string())?;
            conn.execute(
                "UPDATE kasalar SET bakiye = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![alici_kasa_yeni, yeni_kasa_id],
            )
            .map_err(|e| e.to_string())?;

            // Satıcı kasasına giriş
            let satici_kasa_onceki = kasa_son_bakiye(&conn, eski_kasa_id);
            let satici_kasa_yeni = satici_kasa_onceki + input.ucret;
            conn.execute(
                "INSERT INTO kasa_hareketleri (kasa_id, tarih, aciklama, giren, cikan, bakiye)
                 VALUES (?1, ?2, ?3, ?4, 0.0, ?5)",
                params![
                    eski_kasa_id,
                    input.tarih,
                    satici_kasa_aciklama,
                    input.ucret,
                    satici_kasa_yeni
                ],
            )
            .map_err(|e| e.to_string())?;
            conn.execute(
                "UPDATE kasalar SET bakiye = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![satici_kasa_yeni, eski_kasa_id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // Oluşturulan atama kaydını dön
    let atama: HisseAtama = conn
        .query_row(
            "SELECT a.id, a.hisse_id, h.kod, a.hissedar_id, hs.ad, hs.soyad,
                    a.tarih, a.ucret, a.aciklama, a.created_at
             FROM hisse_atamalari a
             JOIN hisseler h ON h.id = a.hisse_id
             JOIN hissedarlar hs ON hs.id = a.hissedar_id
             WHERE a.id = ?1",
            params![atama_id],
            |row| {
                Ok(HisseAtama {
                    id: row.get(0)?,
                    hisse_id: row.get(1)?,
                    hisse_kod: row.get(2)?,
                    hissedar_id: row.get(3)?,
                    hissedar_ad: row.get(4)?,
                    hissedar_soyad: row.get(5)?,
                    tarih: row.get(6)?,
                    ucret: row.get(7)?,
                    aciklama: row.get(8)?,
                    created_at: row.get(9)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(atama)
}

// ─── Hisse Satışı (Hissedar → Sistem) ───────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HisseSatis {
    pub id: i64,
    pub hisse_id: i64,
    pub hisse_kod: String,
    pub hissedar_id: i64,
    pub hissedar_ad: String,
    pub hissedar_soyad: String,
    pub kasa_id: i64,
    pub kasa_ad: String,
    pub satis_tutari: f64,
    pub odenen_tutar: f64,
    pub kalan_tutar: f64,
    pub tarih: String,
    pub tamamlandi: bool,
    pub tamamlanma_tarihi: Option<String>,
    pub iptal: bool,
    pub aciklama: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HisseSatisOdeme {
    pub id: i64,
    pub satis_id: i64,
    pub tutar: f64,
    pub tarih: String,
    pub aciklama: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HisseSatisBaslatInput {
    pub hisse_id: i64,
    pub kasa_id: i64,
    pub satis_tutari: f64,
    pub tarih: String,
    pub aciklama: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HisseSatisOdemeInput {
    pub satis_id: i64,
    pub tutar: f64,
    pub tarih: String,
    pub aciklama: Option<String>,
}

fn satis_odenen_toplam(conn: &rusqlite::Connection, satis_id: i64) -> f64 {
    conn.query_row(
        "SELECT COALESCE(SUM(tutar), 0.0) FROM hisse_satis_odemeleri WHERE satis_id = ?1",
        params![satis_id],
        |r| r.get::<_, f64>(0),
    )
    .unwrap_or(0.0)
}

fn satisi_getir(conn: &rusqlite::Connection, satis_id: i64) -> Result<HisseSatis, String> {
    let (
        id,
        hisse_id,
        hisse_kod,
        hissedar_id,
        hissedar_ad,
        hissedar_soyad,
        kasa_id,
        kasa_ad,
        satis_tutari,
        tarih,
        tamamlandi,
        tamamlanma_tarihi,
        iptal,
        aciklama,
        created_at,
    ): (
        i64,
        i64,
        String,
        i64,
        String,
        String,
        i64,
        String,
        f64,
        String,
        i64,
        Option<String>,
        i64,
        Option<String>,
        String,
    ) = conn
        .query_row(
            "SELECT s.id, s.hisse_id, h.kod, s.hissedar_id, hs.ad, hs.soyad,
                    s.kasa_id, k.ad, s.satis_tutari, s.tarih, s.tamamlandi,
                    s.tamamlanma_tarihi, s.iptal, s.aciklama, s.created_at
             FROM hisse_satislari s
             JOIN hisseler h ON h.id = s.hisse_id
             JOIN hissedarlar hs ON hs.id = s.hissedar_id
             JOIN kasalar k ON k.id = s.kasa_id
             WHERE s.id = ?1",
            params![satis_id],
            |r| {
                Ok((
                    r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?,
                    r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?, r.get(9)?,
                    r.get(10)?, r.get(11)?, r.get(12)?, r.get(13)?, r.get(14)?,
                ))
            },
        )
        .map_err(|_| "Satış bulunamadı".to_string())?;

    let odenen = satis_odenen_toplam(conn, id);
    Ok(HisseSatis {
        id,
        hisse_id,
        hisse_kod,
        hissedar_id,
        hissedar_ad,
        hissedar_soyad,
        kasa_id,
        kasa_ad,
        satis_tutari,
        odenen_tutar: odenen,
        kalan_tutar: (satis_tutari - odenen).max(0.0),
        tarih,
        tamamlandi: tamamlandi != 0,
        tamamlanma_tarihi,
        iptal: iptal != 0,
        aciklama,
        created_at,
    })
}

#[tauri::command]
pub fn hisse_satis_baslat(
    db: Db<'_>,
    input: HisseSatisBaslatInput,
) -> Result<HisseSatis, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    if input.satis_tutari <= 0.0 {
        return Err("Satış tutarı pozitif olmalıdır".to_string());
    }

    // Hisse durumu
    let hisse_durum: String = conn
        .query_row(
            "SELECT durum FROM hisseler WHERE id = ?1",
            params![input.hisse_id],
            |r| r.get(0),
        )
        .map_err(|_| "Hisse bulunamadı".to_string())?;
    if hisse_durum != "atanmis" {
        return Err("Sadece atanmış bir hisse satışa çıkarılabilir.".to_string());
    }

    // Mevcut sahibi bul
    let (hissedar_id,): (i64,) = conn
        .query_row(
            "SELECT hissedar_id FROM hisse_atamalari
             WHERE hisse_id = ?1
             ORDER BY created_at DESC, id DESC LIMIT 1",
            params![input.hisse_id],
            |r| Ok((r.get(0)?,)),
        )
        .map_err(|_| "Hissenin mevcut sahibi bulunamadı".to_string())?;

    // Aktif satış var mı?
    let aktif_satis: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM hisse_satislari
             WHERE hisse_id = ?1 AND tamamlandi = 0 AND iptal = 0",
            params![input.hisse_id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if aktif_satis > 0 {
        return Err("Bu hisse için zaten devam eden bir satış süreci var.".to_string());
    }

    // Kasa doğrulama
    let _: i64 = conn
        .query_row(
            "SELECT id FROM kasalar WHERE id = ?1",
            params![input.kasa_id],
            |r| r.get(0),
        )
        .map_err(|_| "Kasa bulunamadı".to_string())?;

    conn.execute(
        "INSERT INTO hisse_satislari
            (hisse_id, hissedar_id, kasa_id, satis_tutari, tarih, aciklama)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            input.hisse_id,
            hissedar_id,
            input.kasa_id,
            input.satis_tutari,
            input.tarih,
            input.aciklama
        ],
    )
    .map_err(|e| e.to_string())?;

    let satis_id = conn.last_insert_rowid();
    satisi_getir(&conn, satis_id)
}

#[tauri::command]
pub fn hisse_satis_odeme_ekle(
    db: Db<'_>,
    input: HisseSatisOdemeInput,
) -> Result<HisseSatis, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    if input.tutar <= 0.0 {
        return Err("Ödeme tutarı pozitif olmalıdır".to_string());
    }

    // Satışı al
    let satis = satisi_getir(&conn, input.satis_id)?;
    if satis.iptal {
        return Err("Bu satış iptal edilmiş, ödeme eklenemez.".to_string());
    }
    if satis.tamamlandi {
        return Err("Bu satış tamamlanmış, ödeme eklenemez.".to_string());
    }
    if input.tutar > satis.kalan_tutar + 1e-6 {
        return Err(format!(
            "Ödeme kalan tutardan büyük olamaz. Kalan: {:.2}",
            satis.kalan_tutar
        ));
    }

    // Kasa bakiyesi kontrol
    let kasa_onceki = kasa_son_bakiye(&conn, satis.kasa_id);
    if kasa_onceki < input.tutar {
        return Err(format!(
            "Kasa bakiyesi yetersiz (mevcut: {:.2}, gerekli: {:.2}).",
            kasa_onceki, input.tutar
        ));
    }

    // Ödeme kaydı
    conn.execute(
        "INSERT INTO hisse_satis_odemeleri (satis_id, tutar, tarih, aciklama)
         VALUES (?1, ?2, ?3, ?4)",
        params![input.satis_id, input.tutar, input.tarih, input.aciklama],
    )
    .map_err(|e| e.to_string())?;

    // Kasa hareketi — çıkış
    let kasa_aciklama = format!(
        "Hisse satın alma ödemesi: {} - {} {}",
        satis.hisse_kod, satis.hissedar_ad, satis.hissedar_soyad
    );
    let kasa_yeni = kasa_onceki - input.tutar;
    conn.execute(
        "INSERT INTO kasa_hareketleri (kasa_id, tarih, aciklama, giren, cikan, bakiye)
         VALUES (?1, ?2, ?3, 0.0, ?4, ?5)",
        params![
            satis.kasa_id,
            input.tarih,
            kasa_aciklama,
            input.tutar,
            kasa_yeni
        ],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE kasalar SET bakiye = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![kasa_yeni, satis.kasa_id],
    )
    .map_err(|e| e.to_string())?;

    // Hissedar cüzdanı — alacak (sistem hissedara ödeme yaptı)
    let cuzdan_bilgi = format!(
        "Hisse satın alma ödemesi: {} ({})",
        satis.hisse_kod, satis.kasa_ad
    );
    let cuzdan_onceki = cuzdan_son_bakiye(&conn, satis.hissedar_id);
    let cuzdan_yeni = cuzdan_onceki + input.tutar;
    conn.execute(
        "INSERT INTO hissedar_cuzdanlari
             (hissedar_id, donem_id, tarih, bilgi, borc, alacak, bakiye)
         VALUES (?1, NULL, ?2, ?3, 0.0, ?4, ?5)",
        params![
            satis.hissedar_id,
            input.tarih,
            cuzdan_bilgi,
            input.tutar,
            cuzdan_yeni
        ],
    )
    .map_err(|e| e.to_string())?;

    // Ödeme sonrası toplam kontrolü — tamamlandı mı?
    let yeni_odenen = satis.odenen_tutar + input.tutar;
    if yeni_odenen + 1e-6 >= satis.satis_tutari {
        conn.execute(
            "UPDATE hisse_satislari
             SET tamamlandi = 1, tamamlanma_tarihi = ?1
             WHERE id = ?2",
            params![input.tarih, input.satis_id],
        )
        .map_err(|e| e.to_string())?;

        // Hisse artık kullanılamaz
        conn.execute(
            "UPDATE hisseler SET durum = 'satildi', updated_at = datetime('now')
             WHERE id = ?1",
            params![satis.hisse_id],
        )
        .map_err(|e| e.to_string())?;
    }

    satisi_getir(&conn, input.satis_id)
}

#[tauri::command]
pub fn get_hisse_satis_aktif(
    db: Db<'_>,
    hisse_id: i64,
) -> Result<Option<HisseSatis>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let row: Option<i64> = conn
        .query_row(
            "SELECT id FROM hisse_satislari
             WHERE hisse_id = ?1 AND iptal = 0
             ORDER BY tamamlandi ASC, created_at DESC
             LIMIT 1",
            params![hisse_id],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    match row {
        Some(id) => satisi_getir(&conn, id).map(Some),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn get_hisse_satis_odemeleri(
    db: Db<'_>,
    satis_id: i64,
) -> Result<Vec<HisseSatisOdeme>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, satis_id, tutar, tarih, aciklama, created_at
             FROM hisse_satis_odemeleri
             WHERE satis_id = ?1
             ORDER BY tarih ASC, id ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![satis_id], |r| {
            Ok(HisseSatisOdeme {
                id: r.get(0)?,
                satis_id: r.get(1)?,
                tutar: r.get(2)?,
                tarih: r.get(3)?,
                aciklama: r.get(4)?,
                created_at: r.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[tauri::command]
pub fn hisse_satis_iptal(db: Db<'_>, satis_id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let satis = satisi_getir(&conn, satis_id)?;
    if satis.tamamlandi {
        return Err("Tamamlanmış satış iptal edilemez.".to_string());
    }
    if satis.odenen_tutar > 0.0 {
        return Err(
            "Bu satış için ödemeler yapılmış. Geri alma işlemi otomatik yapılmaz; önce ödeme iadelerini manuel işleyin."
                .to_string(),
        );
    }
    conn.execute(
        "UPDATE hisse_satislari SET iptal = 1 WHERE id = ?1",
        params![satis_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
