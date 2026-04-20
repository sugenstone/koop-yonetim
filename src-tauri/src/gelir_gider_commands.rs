use crate::db::Db;
use rusqlite::params;
use serde::{Deserialize, Serialize};

// ─── Modeller ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GelirGiderKategori {
    pub id: i64,
    pub ad: String,
    pub tip: String, // "gelir" | "gider"
    pub aciklama: Option<String>,
    pub aktif: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateKategoriInput {
    pub ad: String,
    pub tip: String,
    pub aciklama: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateKategoriInput {
    pub id: i64,
    pub ad: Option<String>,
    pub aciklama: Option<String>,
    pub aktif: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GelirGiderKayit {
    pub id: i64,
    pub kasa_id: i64,
    pub kasa_ad: String,
    pub kategori_id: i64,
    pub kategori_ad: String,
    pub kategori_tip: String,
    pub tarih: String,
    pub tutar: f64,
    pub aciklama: String,
    pub kasa_hareketi_id: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateKayitInput {
    pub kasa_id: i64,
    pub kategori_id: i64,
    pub tarih: String,
    pub tutar: f64,
    pub aciklama: String,
}

// ─── Kategori Komutları ───────────────────────────────────────────────────────

#[tauri::command]
pub fn get_gelir_gider_kategorileri(db: Db<'_>) -> Result<Vec<GelirGiderKategori>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, ad, tip, aciklama, aktif, created_at, updated_at
             FROM gelir_gider_kategorileri ORDER BY tip ASC, ad ASC",
        )
        .map_err(|e| e.to_string())?;

    let kategoriler = stmt
        .query_map([], |row| {
            Ok(GelirGiderKategori {
                id: row.get(0)?,
                ad: row.get(1)?,
                tip: row.get(2)?,
                aciklama: row.get(3)?,
                aktif: row.get::<_, i64>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(kategoriler)
}

#[tauri::command]
pub fn create_gelir_gider_kategori(
    db: Db<'_>,
    input: CreateKategoriInput,
) -> Result<GelirGiderKategori, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO gelir_gider_kategorileri (ad, tip, aciklama) VALUES (?1, ?2, ?3)",
        params![input.ad, input.tip, input.aciklama],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT id, ad, tip, aciklama, aktif, created_at, updated_at
         FROM gelir_gider_kategorileri WHERE id = ?1",
        params![id],
        |row| {
            Ok(GelirGiderKategori {
                id: row.get(0)?,
                ad: row.get(1)?,
                tip: row.get(2)?,
                aciklama: row.get(3)?,
                aktif: row.get::<_, i64>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_gelir_gider_kategori(
    db: Db<'_>,
    input: UpdateKategoriInput,
) -> Result<GelirGiderKategori, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let aktif_int = input.aktif.map(|v| if v { 1i64 } else { 0i64 });

    conn.execute(
        "UPDATE gelir_gider_kategorileri SET
            ad         = COALESCE(?1, ad),
            aciklama   = COALESCE(?2, aciklama),
            aktif      = COALESCE(?3, aktif),
            updated_at = datetime('now')
         WHERE id = ?4",
        params![input.ad, input.aciklama, aktif_int, input.id],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, ad, tip, aciklama, aktif, created_at, updated_at
         FROM gelir_gider_kategorileri WHERE id = ?1",
        params![input.id],
        |row| {
            Ok(GelirGiderKategori {
                id: row.get(0)?,
                ad: row.get(1)?,
                tip: row.get(2)?,
                aciklama: row.get(3)?,
                aktif: row.get::<_, i64>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_gelir_gider_kategori(db: Db<'_>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let kayit_sayisi: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM gelir_gider_kayitlari WHERE kategori_id = ?1",
            params![id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if kayit_sayisi > 0 {
        return Err(format!(
            "Bu kategoriye ait {} kayıt bulunmaktadır. Önce kayıtları silin.",
            kayit_sayisi
        ));
    }

    conn.execute(
        "DELETE FROM gelir_gider_kategorileri WHERE id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// ─── Kayıt Komutları ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_gelir_gider_kayitlari(
    db: Db<'_>,
    kasa_id: Option<i64>,
    kategori_id: Option<i64>,
) -> Result<Vec<GelirGiderKayit>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT g.id, g.kasa_id, k.ad AS kasa_ad, g.kategori_id,
                    kat.ad AS kategori_ad, kat.tip AS kategori_tip,
                    g.tarih, g.tutar, g.aciklama, g.kasa_hareketi_id, g.created_at
             FROM gelir_gider_kayitlari g
             JOIN kasalar k ON k.id = g.kasa_id
             JOIN gelir_gider_kategorileri kat ON kat.id = g.kategori_id
             WHERE (?1 IS NULL OR g.kasa_id = ?1)
               AND (?2 IS NULL OR g.kategori_id = ?2)
             ORDER BY g.tarih DESC, g.id DESC",
        )
        .map_err(|e| e.to_string())?;

    let kayitlar = stmt
        .query_map(params![kasa_id, kategori_id], |row| {
            Ok(GelirGiderKayit {
                id: row.get(0)?,
                kasa_id: row.get(1)?,
                kasa_ad: row.get(2)?,
                kategori_id: row.get(3)?,
                kategori_ad: row.get(4)?,
                kategori_tip: row.get(5)?,
                tarih: row.get(6)?,
                tutar: row.get(7)?,
                aciklama: row.get(8)?,
                kasa_hareketi_id: row.get(9)?,
                created_at: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(kayitlar)
}

#[tauri::command]
pub fn create_gelir_gider_kaydi(
    db: Db<'_>,
    input: CreateKayitInput,
) -> Result<GelirGiderKayit, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Kategori tipini al
    let (kategori_ad, kategori_tip): (String, String) = conn
        .query_row(
            "SELECT ad, tip FROM gelir_gider_kategorileri WHERE id = ?1",
            params![input.kategori_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    // Gelir → giren, Gider → çıkan
    let (giren, cikan) = if kategori_tip == "gelir" {
        (input.tutar, 0.0f64)
    } else {
        (0.0f64, input.tutar)
    };

    // Son bakiyeyi hesapla
    let son_bakiye: f64 = conn
        .query_row(
            "SELECT COALESCE(bakiye, 0.0) FROM kasa_hareketleri
             WHERE kasa_id = ?1 ORDER BY tarih DESC, id DESC LIMIT 1",
            params![input.kasa_id],
            |r| r.get(0),
        )
        .unwrap_or(0.0);

    let yeni_bakiye = son_bakiye + giren - cikan;

    // Kasa hareketi oluştur
    conn.execute(
        "INSERT INTO kasa_hareketleri (kasa_id, tarih, aciklama, giren, cikan, bakiye)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![input.kasa_id, input.tarih, input.aciklama, giren, cikan, yeni_bakiye],
    )
    .map_err(|e| e.to_string())?;

    let hareket_id = conn.last_insert_rowid();

    // Kasa bakiyesini güncelle
    conn.execute(
        "UPDATE kasalar SET bakiye = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![yeni_bakiye, input.kasa_id],
    )
    .map_err(|e| e.to_string())?;

    // Gelir/gider kaydı oluştur
    conn.execute(
        "INSERT INTO gelir_gider_kayitlari (kasa_id, kategori_id, tarih, tutar, aciklama, kasa_hareketi_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![input.kasa_id, input.kategori_id, input.tarih, input.tutar, input.aciklama, hareket_id],
    )
    .map_err(|e| e.to_string())?;

    let kayit_id = conn.last_insert_rowid();

    let kasa_ad: String = conn
        .query_row(
            "SELECT ad FROM kasalar WHERE id = ?1",
            params![input.kasa_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let created_at: String = conn
        .query_row(
            "SELECT created_at FROM gelir_gider_kayitlari WHERE id = ?1",
            params![kayit_id],
            |row| row.get(0),
        )
        .unwrap_or_default();

    Ok(GelirGiderKayit {
        id: kayit_id,
        kasa_id: input.kasa_id,
        kasa_ad,
        kategori_id: input.kategori_id,
        kategori_ad,
        kategori_tip,
        tarih: input.tarih,
        tutar: input.tutar,
        aciklama: input.aciklama,
        kasa_hareketi_id: Some(hareket_id),
        created_at,
    })
}

#[tauri::command]
pub fn delete_gelir_gider_kaydi(db: Db<'_>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Silinecek kaydın hareket ve kasa bilgisini al
    let (hareket_id, kasa_id): (Option<i64>, i64) = conn
        .query_row(
            "SELECT kasa_hareketi_id, kasa_id FROM gelir_gider_kayitlari WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    // Kaydı sil
    conn.execute(
        "DELETE FROM gelir_gider_kayitlari WHERE id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    // İlişkili kasa hareketini sil
    if let Some(hid) = hareket_id {
        conn.execute(
            "DELETE FROM kasa_hareketleri WHERE id = ?1",
            params![hid],
        )
        .map_err(|e| e.to_string())?;
    }

    // Kasa bakiyesini sıfırdan hesapla
    let yeni_bakiye: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(giren) - SUM(cikan), 0.0) FROM kasa_hareketleri WHERE kasa_id = ?1",
            params![kasa_id],
            |r| r.get(0),
        )
        .unwrap_or(0.0);

    conn.execute(
        "UPDATE kasalar SET bakiye = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![yeni_bakiye, kasa_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
