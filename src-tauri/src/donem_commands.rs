use crate::db::Db;
use rusqlite::params;
use serde::{Deserialize, Serialize};

// ─── Modeller ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Donem {
    pub id: i64,
    pub ay: i64,
    pub yil: i64,
    pub hisse_basi_aidat: f64,
    pub aktif: bool,
    pub toplanti_sayisi: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Toplanti {
    pub id: i64,
    pub donem_id: i64,
    pub tarih: String,
    pub konu: String,
    pub yer: Option<String>,
    pub karar_sayisi: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Karar {
    pub id: i64,
    pub toplanti_id: i64,
    pub karar_no: Option<i64>,
    pub aciklama: String,
    pub created_at: String,
}

// ─── Input Tipleri ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDonemInput {
    pub ay: i64,
    pub yil: i64,
    pub hisse_basi_aidat: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateDonemInput {
    pub id: i64,
    pub ay: Option<i64>,
    pub yil: Option<i64>,
    pub hisse_basi_aidat: Option<f64>,
    pub aktif: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateToplantIInput {
    pub donem_id: i64,
    pub tarih: String,
    pub konu: String,
    pub yer: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateToplantIInput {
    pub id: i64,
    pub tarih: Option<String>,
    pub konu: Option<String>,
    pub yer: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateKararInput {
    pub toplanti_id: i64,
    pub karar_no: Option<i64>,
    pub aciklama: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateKararInput {
    pub id: i64,
    pub karar_no: Option<i64>,
    pub aciklama: Option<String>,
}

// ─── SQL Sabitleri ────────────────────────────────────────────────────────────

const DONEM_SELECT: &str = "
    SELECT d.id, d.ay, d.yil, d.hisse_basi_aidat, d.aktif, d.created_at, d.updated_at,
           COUNT(t.id) AS toplanti_sayisi
    FROM donemler d
    LEFT JOIN toplantilar t ON t.donem_id = d.id
";

const TOPLANTI_SELECT: &str = "
    SELECT t.id, t.donem_id, t.tarih, t.konu, t.yer, t.created_at, t.updated_at,
           COUNT(k.id) AS karar_sayisi
    FROM toplantilar t
    LEFT JOIN kararlar k ON k.toplanti_id = t.id
";

// ─── Map Fonksiyonları ────────────────────────────────────────────────────────

fn map_donem(row: &rusqlite::Row) -> rusqlite::Result<Donem> {
    Ok(Donem {
        id:                 row.get(0)?,
        ay:                 row.get(1)?,
        yil:                row.get(2)?,
        hisse_basi_aidat:   row.get(3)?,
        aktif:              row.get::<_, i64>(4)? != 0,
        created_at:         row.get(5)?,
        updated_at:         row.get(6)?,
        toplanti_sayisi:    row.get(7)?,
    })
}

fn map_toplanti(row: &rusqlite::Row) -> rusqlite::Result<Toplanti> {
    Ok(Toplanti {
        id:           row.get(0)?,
        donem_id:     row.get(1)?,
        tarih:        row.get(2)?,
        konu:         row.get(3)?,
        yer:          row.get(4)?,
        created_at:   row.get(5)?,
        updated_at:   row.get(6)?,
        karar_sayisi: row.get(7)?,
    })
}

fn map_karar(row: &rusqlite::Row) -> rusqlite::Result<Karar> {
    Ok(Karar {
        id:          row.get(0)?,
        toplanti_id: row.get(1)?,
        karar_no:    row.get(2)?,
        aciklama:    row.get(3)?,
        created_at:  row.get(4)?,
    })
}

// ─── Dönem Komutları ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_donemler(db: Db) -> Result<Vec<Donem>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let sql = format!("{} GROUP BY d.id ORDER BY d.yil DESC, d.ay DESC", DONEM_SELECT);
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let donemler = stmt.query_map([], map_donem)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(donemler)
}

#[tauri::command]
pub fn get_donem(db: Db, id: i64) -> Result<Donem, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let sql = format!("{} WHERE d.id = ? GROUP BY d.id", DONEM_SELECT);
    conn.query_row(&sql, params![id], map_donem)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_donem(db: Db, input: CreateDonemInput) -> Result<Donem, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO donemler (ay, yil, hisse_basi_aidat) VALUES (?, ?, ?)",
        params![input.ay, input.yil, input.hisse_basi_aidat],
    ).map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    let sql = format!("{} WHERE d.id = ? GROUP BY d.id", DONEM_SELECT);
    conn.query_row(&sql, params![id], map_donem)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_donem(db: Db, input: UpdateDonemInput) -> Result<Donem, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    if let Some(ay) = input.ay {
        conn.execute(
            "UPDATE donemler SET ay = ?, updated_at = datetime('now') WHERE id = ?",
            params![ay, input.id],
        ).map_err(|e| e.to_string())?;
    }
    if let Some(yil) = input.yil {
        conn.execute(
            "UPDATE donemler SET yil = ?, updated_at = datetime('now') WHERE id = ?",
            params![yil, input.id],
        ).map_err(|e| e.to_string())?;
    }
    if let Some(aidat) = input.hisse_basi_aidat {
        conn.execute(
            "UPDATE donemler SET hisse_basi_aidat = ?, updated_at = datetime('now') WHERE id = ?",
            params![aidat, input.id],
        ).map_err(|e| e.to_string())?;
    }
    if let Some(aktif) = input.aktif {
        conn.execute(
            "UPDATE donemler SET aktif = ?, updated_at = datetime('now') WHERE id = ?",
            params![aktif as i64, input.id],
        ).map_err(|e| e.to_string())?;
    }
    let sql = format!("{} WHERE d.id = ? GROUP BY d.id", DONEM_SELECT);
    conn.query_row(&sql, params![input.id], map_donem)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_donem(db: Db, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM donemler WHERE id = ?", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Toplantı Komutları ──────────────────────────────────────────────────────

#[tauri::command]
pub fn get_toplantilar(db: Db, donem_id: i64) -> Result<Vec<Toplanti>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let sql = format!(
        "{} WHERE t.donem_id = ? GROUP BY t.id ORDER BY t.tarih DESC",
        TOPLANTI_SELECT
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let toplantilar = stmt.query_map(params![donem_id], map_toplanti)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(toplantilar)
}

#[tauri::command]
pub fn create_toplanti(db: Db, input: CreateToplantIInput) -> Result<Toplanti, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO toplantilar (donem_id, tarih, konu, yer) VALUES (?, ?, ?, ?)",
        params![input.donem_id, input.tarih, input.konu, input.yer],
    ).map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    let sql = format!("{} WHERE t.id = ? GROUP BY t.id", TOPLANTI_SELECT);
    conn.query_row(&sql, params![id], map_toplanti)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_toplanti(db: Db, input: UpdateToplantIInput) -> Result<Toplanti, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    if let Some(tarih) = input.tarih {
        conn.execute(
            "UPDATE toplantilar SET tarih = ?, updated_at = datetime('now') WHERE id = ?",
            params![tarih, input.id],
        ).map_err(|e| e.to_string())?;
    }
    if let Some(konu) = input.konu {
        conn.execute(
            "UPDATE toplantilar SET konu = ?, updated_at = datetime('now') WHERE id = ?",
            params![konu, input.id],
        ).map_err(|e| e.to_string())?;
    }
    if let Some(yer) = input.yer {
        conn.execute(
            "UPDATE toplantilar SET yer = ?, updated_at = datetime('now') WHERE id = ?",
            params![yer, input.id],
        ).map_err(|e| e.to_string())?;
    }
    let sql = format!("{} WHERE t.id = ? GROUP BY t.id", TOPLANTI_SELECT);
    conn.query_row(&sql, params![input.id], map_toplanti)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_toplanti(db: Db, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM toplantilar WHERE id = ?", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Karar Komutları ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_kararlar(db: Db, toplanti_id: i64) -> Result<Vec<Karar>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, toplanti_id, karar_no, aciklama, created_at
         FROM kararlar WHERE toplanti_id = ?
         ORDER BY karar_no ASC, id ASC",
    ).map_err(|e| e.to_string())?;
    let kararlar = stmt.query_map(params![toplanti_id], map_karar)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(kararlar)
}

#[tauri::command]
pub fn create_karar(db: Db, input: CreateKararInput) -> Result<Karar, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO kararlar (toplanti_id, karar_no, aciklama) VALUES (?, ?, ?)",
        params![input.toplanti_id, input.karar_no, input.aciklama],
    ).map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT id, toplanti_id, karar_no, aciklama, created_at FROM kararlar WHERE id = ?",
        params![id],
        map_karar,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_karar(db: Db, input: UpdateKararInput) -> Result<Karar, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    if let Some(karar_no) = input.karar_no {
        conn.execute(
            "UPDATE kararlar SET karar_no = ? WHERE id = ?",
            params![karar_no, input.id],
        ).map_err(|e| e.to_string())?;
    }
    if let Some(aciklama) = input.aciklama {
        conn.execute(
            "UPDATE kararlar SET aciklama = ? WHERE id = ?",
            params![aciklama, input.id],
        ).map_err(|e| e.to_string())?;
    }
    conn.query_row(
        "SELECT id, toplanti_id, karar_no, aciklama, created_at FROM kararlar WHERE id = ?",
        params![input.id],
        map_karar,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_karar(db: Db, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM kararlar WHERE id = ?", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
