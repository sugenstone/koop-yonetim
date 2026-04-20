use crate::db::Db;
use rusqlite::params;
use serde::{Deserialize, Serialize};

// ─── Modeller ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Kasa {
    pub id: i64,
    pub ad: String,
    pub para_birimi: String,
    pub bakiye: f64,
    pub aciklama: Option<String>,
    pub aktif: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateKasaInput {
    pub ad: String,
    pub para_birimi: String,
    pub aciklama: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateKasaInput {
    pub id: i64,
    pub ad: Option<String>,
    pub para_birimi: Option<String>,
    pub aciklama: Option<String>,
    pub aktif: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KasaHareketi {
    pub id: i64,
    pub kasa_id: i64,
    pub tarih: String,
    pub aciklama: String,
    pub giren: f64,
    pub cikan: f64,
    pub bakiye: f64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateHareket {
    pub kasa_id: i64,
    pub tarih: String,
    pub aciklama: String,
    pub giren: f64,
    pub cikan: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteHareket {
    pub id: i64,
    pub kasa_id: i64,
}

// ─── Kasa Komutları ───────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_kasalar(db: Db<'_>) -> Result<Vec<Kasa>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, ad, para_birimi, bakiye, aciklama, aktif, created_at, updated_at
             FROM kasalar ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let kasalar = stmt
        .query_map([], |row| {
            Ok(Kasa {
                id: row.get(0)?,
                ad: row.get(1)?,
                para_birimi: row.get(2)?,
                bakiye: row.get(3)?,
                aciklama: row.get(4)?,
                aktif: row.get::<_, i64>(5)? != 0,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(kasalar)
}

#[tauri::command]
pub fn get_kasa(db: Db<'_>, id: i64) -> Result<Kasa, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, ad, para_birimi, bakiye, aciklama, aktif, created_at, updated_at
         FROM kasalar WHERE id = ?1",
        params![id],
        |row| {
            Ok(Kasa {
                id: row.get(0)?,
                ad: row.get(1)?,
                para_birimi: row.get(2)?,
                bakiye: row.get(3)?,
                aciklama: row.get(4)?,
                aktif: row.get::<_, i64>(5)? != 0,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_kasa(db: Db<'_>, input: CreateKasaInput) -> Result<Kasa, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO kasalar (ad, para_birimi, aciklama) VALUES (?1, ?2, ?3)",
        params![input.ad, input.para_birimi, input.aciklama],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT id, ad, para_birimi, bakiye, aciklama, aktif, created_at, updated_at
         FROM kasalar WHERE id = ?1",
        params![id],
        |row| {
            Ok(Kasa {
                id: row.get(0)?,
                ad: row.get(1)?,
                para_birimi: row.get(2)?,
                bakiye: row.get(3)?,
                aciklama: row.get(4)?,
                aktif: row.get::<_, i64>(5)? != 0,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_kasa(db: Db<'_>, input: UpdateKasaInput) -> Result<Kasa, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let aktif_int = input.aktif.map(|v| if v { 1i64 } else { 0i64 });

    conn.execute(
        "UPDATE kasalar SET
            ad          = COALESCE(?1, ad),
            para_birimi = COALESCE(?2, para_birimi),
            aciklama    = COALESCE(?3, aciklama),
            aktif       = COALESCE(?4, aktif),
            updated_at  = datetime('now')
         WHERE id = ?5",
        params![input.ad, input.para_birimi, input.aciklama, aktif_int, input.id],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, ad, para_birimi, bakiye, aciklama, aktif, created_at, updated_at
         FROM kasalar WHERE id = ?1",
        params![input.id],
        |row| {
            Ok(Kasa {
                id: row.get(0)?,
                ad: row.get(1)?,
                para_birimi: row.get(2)?,
                bakiye: row.get(3)?,
                aciklama: row.get(4)?,
                aktif: row.get::<_, i64>(5)? != 0,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_kasa(db: Db<'_>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM kasalar WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Kasa Hareketi Komutları ──────────────────────────────────────────────────

#[tauri::command]
pub fn get_kasa_hareketleri(db: Db<'_>, kasa_id: i64) -> Result<Vec<KasaHareketi>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, kasa_id, tarih, aciklama, giren, cikan, bakiye, created_at
             FROM kasa_hareketleri WHERE kasa_id = ?1 ORDER BY tarih ASC, id ASC",
        )
        .map_err(|e| e.to_string())?;

    let hareketler = stmt
        .query_map(params![kasa_id], |row| {
            Ok(KasaHareketi {
                id: row.get(0)?,
                kasa_id: row.get(1)?,
                tarih: row.get(2)?,
                aciklama: row.get(3)?,
                giren: row.get(4)?,
                cikan: row.get(5)?,
                bakiye: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(hareketler)
}

#[tauri::command]
pub fn create_kasa_hareketi(db: Db<'_>, input: CreateHareket) -> Result<KasaHareketi, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Son bakiyeyi hesapla
    let son_bakiye: f64 = conn
        .query_row(
            "SELECT COALESCE(bakiye, 0.0) FROM kasa_hareketleri
             WHERE kasa_id = ?1 ORDER BY tarih DESC, id DESC LIMIT 1",
            params![input.kasa_id],
            |r| r.get(0),
        )
        .unwrap_or(0.0);

    let yeni_bakiye = son_bakiye + input.giren - input.cikan;

    conn.execute(
        "INSERT INTO kasa_hareketleri (kasa_id, tarih, aciklama, giren, cikan, bakiye)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![input.kasa_id, input.tarih, input.aciklama, input.giren, input.cikan, yeni_bakiye],
    )
    .map_err(|e| e.to_string())?;

    let hareket_id = conn.last_insert_rowid();

    // Kasa bakiyesini güncelle
    conn.execute(
        "UPDATE kasalar SET bakiye = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![yeni_bakiye, input.kasa_id],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, kasa_id, tarih, aciklama, giren, cikan, bakiye, created_at
         FROM kasa_hareketleri WHERE id = ?1",
        params![hareket_id],
        |row| {
            Ok(KasaHareketi {
                id: row.get(0)?,
                kasa_id: row.get(1)?,
                tarih: row.get(2)?,
                aciklama: row.get(3)?,
                giren: row.get(4)?,
                cikan: row.get(5)?,
                bakiye: row.get(6)?,
                created_at: row.get(7)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_kasa_hareketi(db: Db<'_>, input: DeleteHareket) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM kasa_hareketleri WHERE id = ?1",
        params![input.id],
    )
    .map_err(|e| e.to_string())?;

    // Kasanın bakiyesini son hareketten yeniden hesapla
    let son_bakiye: f64 = conn
        .query_row(
            "SELECT COALESCE(bakiye, 0.0) FROM kasa_hareketleri
             WHERE kasa_id = ?1 ORDER BY tarih DESC, id DESC LIMIT 1",
            params![input.kasa_id],
            |r| r.get(0),
        )
        .unwrap_or(0.0);

    conn.execute(
        "UPDATE kasalar SET bakiye = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![son_bakiye, input.kasa_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// ─── Kasa Transfer Modeli ─────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct KasaTransfer {
    pub id: i64,
    pub kaynak_kasa_id: i64,
    pub kaynak_kasa_ad: String,
    pub kaynak_kasa_para_birimi: String,
    pub hedef_kasa_id: i64,
    pub hedef_kasa_ad: String,
    pub hedef_kasa_para_birimi: String,
    pub tarih: String,
    pub kaynak_miktar: f64,
    pub hedef_miktar: f64,
    pub kur: Option<f64>,
    pub aciklama: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferInput {
    pub kaynak_kasa_id: i64,
    pub hedef_kasa_id: i64,
    pub tarih: String,
    /// Aynı para birimi → kullanıcının girdiği miktar (hem kaynak hem hedef)
    /// Farklı para birimi → hedef kasaya eklenecek miktar (örn. gram altın)
    pub hedef_miktar: f64,
    /// Farklı para birimi transferlerde dönüşüm kuru (1 hedef birimi = kur kaynak birimi)
    /// Aynı para birimi → None (ya da 1.0)
    pub kur: Option<f64>,
    pub aciklama: Option<String>,
}

// ─── Transfer Komutu ──────────────────────────────────────────────────────────

#[tauri::command]
pub fn kasa_transfer(db: Db<'_>, input: TransferInput) -> Result<KasaTransfer, String> {
    if input.kaynak_kasa_id == input.hedef_kasa_id {
        return Err("Kaynak ve hedef kasa aynı olamaz".to_string());
    }
    if input.hedef_miktar <= 0.0 {
        return Err("Transfer miktarı sıfırdan büyük olmalıdır".to_string());
    }

    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Kaynak ve hedef kasaları çek
    let (kaynak_bakiye, kaynak_para_birimi): (f64, String) = conn.query_row(
        "SELECT bakiye, para_birimi FROM kasalar WHERE id = ?1",
        params![input.kaynak_kasa_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ).map_err(|e| format!("Kaynak kasa bulunamadı: {e}"))?;

    let (_hedef_bakiye, hedef_para_birimi): (f64, String) = conn.query_row(
        "SELECT bakiye, para_birimi FROM kasalar WHERE id = ?1",
        params![input.hedef_kasa_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ).map_err(|e| format!("Hedef kasa bulunamadı: {e}"))?;

    // Kaynak miktarı hesapla
    let kaynak_miktar = if kaynak_para_birimi == hedef_para_birimi {
        // Aynı para birimi → miktar bire bir
        input.hedef_miktar
    } else {
        // Farklı para birimi → kur zorunlu
        let kur = input.kur.ok_or("Farklı para birimleri için kur girilmelidir")?;
        if kur <= 0.0 {
            return Err("Kur sıfırdan büyük olmalıdır".to_string());
        }
        input.hedef_miktar * kur
    };

    // Bakiye kontrolü
    if kaynak_bakiye < kaynak_miktar {
        return Err(format!(
            "Yetersiz bakiye. Mevcut: {:.4}, Gereken: {:.4}",
            kaynak_bakiye, kaynak_miktar
        ));
    }

    // Kaynak kasa hareketi: çıkan
    let kaynak_son_bakiye: f64 = conn.query_row(
        "SELECT COALESCE(bakiye, 0.0) FROM kasa_hareketleri
         WHERE kasa_id = ?1 ORDER BY tarih DESC, id DESC LIMIT 1",
        params![input.kaynak_kasa_id],
        |r| r.get(0),
    ).unwrap_or(0.0);
    let kaynak_yeni_bakiye = kaynak_son_bakiye - kaynak_miktar;

    let hedef_aciklama = input.aciklama.clone().unwrap_or_else(|| "Transfer".to_string());
    let kaynak_aciklama = format!(
        "Transfer → {} [{}]",
        hedef_para_birimi,
        hedef_aciklama
    );

    conn.execute(
        "INSERT INTO kasa_hareketleri (kasa_id, tarih, aciklama, giren, cikan, bakiye)
         VALUES (?1, ?2, ?3, 0.0, ?4, ?5)",
        params![input.kaynak_kasa_id, input.tarih, kaynak_aciklama, kaynak_miktar, kaynak_yeni_bakiye],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE kasalar SET bakiye = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![kaynak_yeni_bakiye, input.kaynak_kasa_id],
    ).map_err(|e| e.to_string())?;

    // Hedef kasa hareketi: giren
    let hedef_son_bakiye: f64 = conn.query_row(
        "SELECT COALESCE(bakiye, 0.0) FROM kasa_hareketleri
         WHERE kasa_id = ?1 ORDER BY tarih DESC, id DESC LIMIT 1",
        params![input.hedef_kasa_id],
        |r| r.get(0),
    ).unwrap_or(0.0);
    let hedef_yeni_bakiye = hedef_son_bakiye + input.hedef_miktar;

    let hedef_hareket_aciklama = format!(
        "Transfer ← {} [{}]",
        kaynak_para_birimi,
        hedef_aciklama
    );

    conn.execute(
        "INSERT INTO kasa_hareketleri (kasa_id, tarih, aciklama, giren, cikan, bakiye)
         VALUES (?1, ?2, ?3, ?4, 0.0, ?5)",
        params![input.hedef_kasa_id, input.tarih, hedef_hareket_aciklama, input.hedef_miktar, hedef_yeni_bakiye],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE kasalar SET bakiye = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![hedef_yeni_bakiye, input.hedef_kasa_id],
    ).map_err(|e| e.to_string())?;

    // Transfer kaydını oluştur
    conn.execute(
        "INSERT INTO kasa_transferleri
         (kaynak_kasa_id, hedef_kasa_id, tarih, kaynak_miktar, hedef_miktar, kur, aciklama)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            input.kaynak_kasa_id, input.hedef_kasa_id,
            input.tarih, kaynak_miktar, input.hedef_miktar,
            input.kur, input.aciklama
        ],
    ).map_err(|e| e.to_string())?;

    let transfer_id = conn.last_insert_rowid();

    conn.query_row(
        "SELECT t.id, t.kaynak_kasa_id, k1.ad, k1.para_birimi,
                t.hedef_kasa_id, k2.ad, k2.para_birimi,
                t.tarih, t.kaynak_miktar, t.hedef_miktar, t.kur, t.aciklama, t.created_at
         FROM kasa_transferleri t
         JOIN kasalar k1 ON k1.id = t.kaynak_kasa_id
         JOIN kasalar k2 ON k2.id = t.hedef_kasa_id
         WHERE t.id = ?1",
        params![transfer_id],
        |row| {
            Ok(KasaTransfer {
                id: row.get(0)?,
                kaynak_kasa_id: row.get(1)?,
                kaynak_kasa_ad: row.get(2)?,
                kaynak_kasa_para_birimi: row.get(3)?,
                hedef_kasa_id: row.get(4)?,
                hedef_kasa_ad: row.get(5)?,
                hedef_kasa_para_birimi: row.get(6)?,
                tarih: row.get(7)?,
                kaynak_miktar: row.get(8)?,
                hedef_miktar: row.get(9)?,
                kur: row.get(10)?,
                aciklama: row.get(11)?,
                created_at: row.get(12)?,
            })
        },
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_kasa_transferleri(db: Db<'_>, kasa_id: i64) -> Result<Vec<KasaTransfer>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT t.id, t.kaynak_kasa_id, k1.ad, k1.para_birimi,
                t.hedef_kasa_id, k2.ad, k2.para_birimi,
                t.tarih, t.kaynak_miktar, t.hedef_miktar, t.kur, t.aciklama, t.created_at
         FROM kasa_transferleri t
         JOIN kasalar k1 ON k1.id = t.kaynak_kasa_id
         JOIN kasalar k2 ON k2.id = t.hedef_kasa_id
         WHERE t.kaynak_kasa_id = ?1 OR t.hedef_kasa_id = ?1
         ORDER BY t.tarih DESC, t.id DESC",
    ).map_err(|e| e.to_string())?;

    let transferler = stmt.query_map(params![kasa_id], |row| {
        Ok(KasaTransfer {
            id: row.get(0)?,
            kaynak_kasa_id: row.get(1)?,
            kaynak_kasa_ad: row.get(2)?,
            kaynak_kasa_para_birimi: row.get(3)?,
            hedef_kasa_id: row.get(4)?,
            hedef_kasa_ad: row.get(5)?,
            hedef_kasa_para_birimi: row.get(6)?,
            tarih: row.get(7)?,
            kaynak_miktar: row.get(8)?,
            hedef_miktar: row.get(9)?,
            kur: row.get(10)?,
            aciklama: row.get(11)?,
            created_at: row.get(12)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(transferler)
}
