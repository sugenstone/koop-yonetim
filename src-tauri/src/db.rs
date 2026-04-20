use rusqlite::{Connection, Result};
use std::sync::Mutex;
use tauri::State;

pub struct DbState(pub Mutex<Connection>);

impl DbState {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(DbState(Mutex::new(conn)))
    }
}

pub type Db<'a> = State<'a, DbState>;

/// Veritabanı tablolarını oluştur (ilk çalıştırmada)
pub fn init_db(db_path: &str) -> Result<()> {
    let conn = Connection::open(db_path)?;

    conn.execute_batch("
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;

        CREATE TABLE IF NOT EXISTS users (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT NOT NULL,
            email       TEXT NOT NULL UNIQUE,
            role        TEXT NOT NULL DEFAULT 'user',
            status      TEXT NOT NULL DEFAULT 'active',
            created_at  TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS products (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT NOT NULL,
            category    TEXT NOT NULL,
            price       REAL NOT NULL DEFAULT 0.0,
            stock       INTEGER NOT NULL DEFAULT 0,
            status      TEXT NOT NULL DEFAULT 'active',
            created_at  TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS orders (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            total       REAL NOT NULL DEFAULT 0.0,
            status      TEXT NOT NULL DEFAULT 'pending',
            created_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS order_items (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            order_id    INTEGER NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
            product_id  INTEGER NOT NULL REFERENCES products(id),
            quantity    INTEGER NOT NULL DEFAULT 1,
            unit_price  REAL NOT NULL DEFAULT 0.0
        );

        CREATE TABLE IF NOT EXISTS kasalar (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            ad          TEXT NOT NULL,
            para_birimi TEXT NOT NULL DEFAULT 'TL',
            bakiye      REAL NOT NULL DEFAULT 0.0,
            aciklama    TEXT,
            aktif       INTEGER NOT NULL DEFAULT 1,
            created_at  TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS kasa_hareketleri (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            kasa_id     INTEGER NOT NULL REFERENCES kasalar(id) ON DELETE CASCADE,
            tarih       TEXT NOT NULL DEFAULT (date('now')),
            aciklama    TEXT NOT NULL,
            giren       REAL NOT NULL DEFAULT 0.0,
            cikan       REAL NOT NULL DEFAULT 0.0,
            bakiye      REAL NOT NULL DEFAULT 0.0,
            created_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS hissedarlar (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            ad                  TEXT NOT NULL,
            soyad               TEXT NOT NULL,
            kasa_id             INTEGER NOT NULL REFERENCES kasalar(id),
            aile_sira_no        INTEGER,
            tcno                TEXT,
            tel                 TEXT,
            yakin_adi           TEXT,
            yakinlik_derecesi   TEXT,
            aktif               INTEGER NOT NULL DEFAULT 1,
            created_at          TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at          TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS donemler (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            ay                  INTEGER NOT NULL CHECK(ay BETWEEN 1 AND 12),
            yil                 INTEGER NOT NULL,
            hisse_basi_aidat    REAL    NOT NULL DEFAULT 0,
            aktif               INTEGER NOT NULL DEFAULT 1,
            created_at          TEXT    NOT NULL DEFAULT (datetime('now')),
            updated_at          TEXT    NOT NULL DEFAULT (datetime('now')),
            UNIQUE(ay, yil)
        );

        CREATE TABLE IF NOT EXISTS toplantilar (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            donem_id            INTEGER NOT NULL REFERENCES donemler(id) ON DELETE CASCADE,
            tarih               TEXT    NOT NULL,
            konu                TEXT    NOT NULL,
            yer                 TEXT,
            created_at          TEXT    NOT NULL DEFAULT (datetime('now')),
            updated_at          TEXT    NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS kararlar (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            toplanti_id         INTEGER NOT NULL REFERENCES toplantilar(id) ON DELETE CASCADE,
            karar_no            INTEGER,
            aciklama            TEXT    NOT NULL,
            created_at          TEXT    NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS kasa_transferleri (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            kaynak_kasa_id      INTEGER NOT NULL REFERENCES kasalar(id),
            hedef_kasa_id       INTEGER NOT NULL REFERENCES kasalar(id),
            tarih               TEXT    NOT NULL DEFAULT (date('now')),
            kaynak_miktar       REAL    NOT NULL,
            hedef_miktar        REAL    NOT NULL,
            kur                 REAL,
            aciklama            TEXT,
            created_at          TEXT    NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS hisseler (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            kod         TEXT    NOT NULL UNIQUE,
            durum       TEXT    NOT NULL DEFAULT 'musait',
            aciklama    TEXT,
            created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
            updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS hisse_atamalari (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            hisse_id        INTEGER NOT NULL REFERENCES hisseler(id) ON DELETE CASCADE,
            hissedar_id     INTEGER NOT NULL REFERENCES hissedarlar(id),
            tarih           TEXT    NOT NULL DEFAULT (date('now')),
            ucret           REAL    NOT NULL DEFAULT 0.0,
            aciklama        TEXT,
            created_at      TEXT    NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS donem_aidat_borclari (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            donem_id        INTEGER NOT NULL REFERENCES donemler(id) ON DELETE CASCADE,
            hissedar_id     INTEGER NOT NULL REFERENCES hissedarlar(id),
            hisse_sayisi    INTEGER NOT NULL DEFAULT 1,
            tutar           REAL    NOT NULL,
            odendi          INTEGER NOT NULL DEFAULT 0,
            odeme_tarihi    TEXT,
            aciklama        TEXT,
            created_at      TEXT    NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS hissedar_cuzdanlari (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            hissedar_id     INTEGER NOT NULL REFERENCES hissedarlar(id) ON DELETE CASCADE,
            donem_id        INTEGER REFERENCES donemler(id),
            tarih           TEXT    NOT NULL DEFAULT (date('now')),
            bilgi           TEXT    NOT NULL,
            borc            REAL    NOT NULL DEFAULT 0.0,
            alacak          REAL    NOT NULL DEFAULT 0.0,
            bakiye          REAL    NOT NULL DEFAULT 0.0,
            created_at      TEXT    NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS hisse_satislari (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            hisse_id            INTEGER NOT NULL REFERENCES hisseler(id) ON DELETE CASCADE,
            hissedar_id         INTEGER NOT NULL REFERENCES hissedarlar(id),
            kasa_id             INTEGER NOT NULL REFERENCES kasalar(id),
            satis_tutari        REAL    NOT NULL,
            tarih               TEXT    NOT NULL DEFAULT (date('now')),
            tamamlandi          INTEGER NOT NULL DEFAULT 0,
            tamamlanma_tarihi   TEXT,
            iptal               INTEGER NOT NULL DEFAULT 0,
            aciklama            TEXT,
            created_at          TEXT    NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS hisse_satis_odemeleri (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            satis_id        INTEGER NOT NULL REFERENCES hisse_satislari(id) ON DELETE CASCADE,
            tutar           REAL    NOT NULL,
            tarih           TEXT    NOT NULL DEFAULT (date('now')),
            aciklama        TEXT,
            created_at      TEXT    NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS gelir_gider_kategorileri (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            ad          TEXT    NOT NULL,
            tip         TEXT    NOT NULL CHECK(tip IN ('gelir', 'gider')),
            aciklama    TEXT,
            aktif       INTEGER NOT NULL DEFAULT 1,
            created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
            updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS gelir_gider_kayitlari (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            kasa_id             INTEGER NOT NULL REFERENCES kasalar(id),
            kategori_id         INTEGER NOT NULL REFERENCES gelir_gider_kategorileri(id),
            tarih               TEXT    NOT NULL DEFAULT (date('now')),
            tutar               REAL    NOT NULL,
            aciklama            TEXT    NOT NULL,
            kasa_hareketi_id    INTEGER REFERENCES kasa_hareketleri(id),
            created_at          TEXT    NOT NULL DEFAULT (datetime('now'))
        );
    ")?;

    // Migration: eski şemada hisse_id sütunu varsa tabloyu yeniden oluştur
    let eski_sema: bool = {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('donem_aidat_borclari') WHERE name='hisse_id'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        count > 0
    };
    if eski_sema {
        conn.execute_batch(
            "DROP TABLE IF EXISTS donem_aidat_borclari;
             CREATE TABLE donem_aidat_borclari (
                 id              INTEGER PRIMARY KEY AUTOINCREMENT,
                 donem_id        INTEGER NOT NULL REFERENCES donemler(id) ON DELETE CASCADE,
                 hissedar_id     INTEGER NOT NULL REFERENCES hissedarlar(id),
                 hisse_sayisi    INTEGER NOT NULL DEFAULT 1,
                 tutar           REAL    NOT NULL,
                 odendi          INTEGER NOT NULL DEFAULT 0,
                 odeme_tarihi    TEXT,
                 aciklama        TEXT,
                 created_at      TEXT    NOT NULL DEFAULT (datetime('now'))
             );",
        )?;
    }

    Ok(())
}
