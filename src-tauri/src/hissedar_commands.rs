use crate::db::Db;
use rusqlite::params;
use serde::{Deserialize, Serialize};

// ─── Modeller ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hissedar {
    pub id: i64,
    pub ad: String,
    pub soyad: String,
    pub kasa_id: i64,
    pub kasa_ad: Option<String>,        // JOIN ile gelir
    pub aile_sira_no: Option<i64>,
    pub tcno: Option<String>,
    pub tel: Option<String>,
    pub yakin_adi: Option<String>,
    pub yakinlik_derecesi: Option<String>,
    pub aktif: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateHissedarInput {
    pub id: i64,
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

// ─── Yardımcı makro yerine ortak mapper ──────────────────────────────────────

fn map_hissedar(row: &rusqlite::Row<'_>) -> rusqlite::Result<Hissedar> {
    Ok(Hissedar {
        id: row.get(0)?,
        ad: row.get(1)?,
        soyad: row.get(2)?,
        kasa_id: row.get(3)?,
        kasa_ad: row.get(4)?,
        aile_sira_no: row.get(5)?,
        tcno: row.get(6)?,
        tel: row.get(7)?,
        yakin_adi: row.get(8)?,
        yakinlik_derecesi: row.get(9)?,
        aktif: row.get::<_, i64>(10)? != 0,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

const SELECT_SQL: &str = "
    SELECT h.id, h.ad, h.soyad, h.kasa_id, k.ad AS kasa_ad,
           h.aile_sira_no, h.tcno, h.tel, h.yakin_adi, h.yakinlik_derecesi,
           h.aktif, h.created_at, h.updated_at
    FROM hissedarlar h
    LEFT JOIN kasalar k ON k.id = h.kasa_id
";

// ─── Komutlar ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_hissedarlar(db: Db<'_>) -> Result<Vec<Hissedar>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let sql = format!("{} ORDER BY h.soyad, h.ad", SELECT_SQL);
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let liste = stmt
        .query_map([], map_hissedar)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(liste)
}

#[tauri::command]
pub fn get_hissedar(db: Db<'_>, id: i64) -> Result<Hissedar, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let sql = format!("{} WHERE h.id = ?1", SELECT_SQL);
    conn.query_row(&sql, params![id], map_hissedar)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_hissedar(db: Db<'_>, input: CreateHissedarInput) -> Result<Hissedar, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO hissedarlar
            (ad, soyad, kasa_id, aile_sira_no, tcno, tel, yakin_adi, yakinlik_derecesi)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            input.ad,
            input.soyad,
            input.kasa_id,
            input.aile_sira_no,
            input.tcno,
            input.tel,
            input.yakin_adi,
            input.yakinlik_derecesi,
        ],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    let sql = format!("{} WHERE h.id = ?1", SELECT_SQL);
    conn.query_row(&sql, params![id], map_hissedar)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_hissedar(db: Db<'_>, input: UpdateHissedarInput) -> Result<Hissedar, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let aktif_int = input.aktif.map(|v| if v { 1i64 } else { 0i64 });

    conn.execute(
        "UPDATE hissedarlar SET
            ad                = COALESCE(?1, ad),
            soyad             = COALESCE(?2, soyad),
            kasa_id           = COALESCE(?3, kasa_id),
            aile_sira_no      = COALESCE(?4, aile_sira_no),
            tcno              = COALESCE(?5, tcno),
            tel               = COALESCE(?6, tel),
            yakin_adi         = COALESCE(?7, yakin_adi),
            yakinlik_derecesi = COALESCE(?8, yakinlik_derecesi),
            aktif             = COALESCE(?9, aktif),
            updated_at        = datetime('now')
         WHERE id = ?10",
        params![
            input.ad,
            input.soyad,
            input.kasa_id,
            input.aile_sira_no,
            input.tcno,
            input.tel,
            input.yakin_adi,
            input.yakinlik_derecesi,
            aktif_int,
            input.id,
        ],
    )
    .map_err(|e| e.to_string())?;

    let sql = format!("{} WHERE h.id = ?1", SELECT_SQL);
    conn.query_row(&sql, params![input.id], map_hissedar)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_hissedar(db: Db<'_>, id: i64) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM hissedarlar WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
