-- Koop Yönetim Sistemi - PostgreSQL Migration
-- SQLite'dan PostgreSQL'e dönüştürülmüş

-- Kullanıcılar (Auth için)
CREATE TABLE IF NOT EXISTS kullanicilar (
    id          BIGSERIAL PRIMARY KEY,
    ad          TEXT        NOT NULL,
    email       TEXT        NOT NULL UNIQUE,
    sifre_hash  TEXT        NOT NULL,
    rol         TEXT        NOT NULL DEFAULT 'izleyici'
                            CHECK(rol IN ('admin', 'muhasebe', 'uye', 'izleyici')),
    aktif       BOOLEAN     NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Kasalar
CREATE TABLE IF NOT EXISTS kasalar (
    id          BIGSERIAL PRIMARY KEY,
    ad          TEXT        NOT NULL,
    para_birimi TEXT        NOT NULL DEFAULT 'TL',
    bakiye      DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    aciklama    TEXT,
    aktif       BOOLEAN     NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Kasa Hareketleri
CREATE TABLE IF NOT EXISTS kasa_hareketleri (
    id          BIGSERIAL PRIMARY KEY,
    kasa_id     BIGINT      NOT NULL REFERENCES kasalar(id) ON DELETE CASCADE,
    tarih       DATE        NOT NULL DEFAULT CURRENT_DATE,
    aciklama    TEXT        NOT NULL,
    giren       DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    cikan       DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    bakiye      DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Hissedarlar
CREATE TABLE IF NOT EXISTS hissedarlar (
    id                  BIGSERIAL PRIMARY KEY,
    ad                  TEXT    NOT NULL,
    soyad               TEXT    NOT NULL,
    kasa_id             BIGINT  NOT NULL REFERENCES kasalar(id),
    aile_sira_no        BIGINT,
    tcno                TEXT,
    tel                 TEXT,
    yakin_adi           TEXT,
    yakinlik_derecesi   TEXT,
    aktif               BOOLEAN NOT NULL DEFAULT true,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Dönemler
CREATE TABLE IF NOT EXISTS donemler (
    id                  BIGSERIAL PRIMARY KEY,
    ay                  INTEGER NOT NULL CHECK(ay BETWEEN 1 AND 12),
    yil                 INTEGER NOT NULL,
    hisse_basi_aidat    DOUBLE PRECISION NOT NULL DEFAULT 0,
    aktif               BOOLEAN NOT NULL DEFAULT true,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(ay, yil)
);

-- Toplantılar
CREATE TABLE IF NOT EXISTS toplantilar (
    id          BIGSERIAL PRIMARY KEY,
    donem_id    BIGINT  NOT NULL REFERENCES donemler(id) ON DELETE CASCADE,
    tarih       DATE    NOT NULL,
    konu        TEXT    NOT NULL,
    yer         TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Kararlar
CREATE TABLE IF NOT EXISTS kararlar (
    id          BIGSERIAL PRIMARY KEY,
    toplanti_id BIGINT  NOT NULL REFERENCES toplantilar(id) ON DELETE CASCADE,
    karar_no    INTEGER,
    aciklama    TEXT    NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Kasa Transferleri
CREATE TABLE IF NOT EXISTS kasa_transferleri (
    id              BIGSERIAL PRIMARY KEY,
    kaynak_kasa_id  BIGINT  NOT NULL REFERENCES kasalar(id),
    hedef_kasa_id   BIGINT  NOT NULL REFERENCES kasalar(id),
    tarih           DATE    NOT NULL DEFAULT CURRENT_DATE,
    kaynak_miktar   DOUBLE PRECISION NOT NULL,
    hedef_miktar    DOUBLE PRECISION NOT NULL,
    kur             DOUBLE PRECISION,
    aciklama        TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Hisseler
CREATE TABLE IF NOT EXISTS hisseler (
    id          BIGSERIAL PRIMARY KEY,
    kod         TEXT    NOT NULL UNIQUE,
    durum       TEXT    NOT NULL DEFAULT 'musait'
                        CHECK(durum IN ('musait', 'atanmis', 'satildi')),
    aciklama    TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Hisse Atamaları
CREATE TABLE IF NOT EXISTS hisse_atamalari (
    id          BIGSERIAL PRIMARY KEY,
    hisse_id    BIGINT  NOT NULL REFERENCES hisseler(id) ON DELETE CASCADE,
    hissedar_id BIGINT  NOT NULL REFERENCES hissedarlar(id),
    tarih       DATE    NOT NULL DEFAULT CURRENT_DATE,
    ucret       DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    aciklama    TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Dönem Aidat Borçları
CREATE TABLE IF NOT EXISTS donem_aidat_borclari (
    id              BIGSERIAL PRIMARY KEY,
    donem_id        BIGINT  NOT NULL REFERENCES donemler(id) ON DELETE CASCADE,
    hissedar_id     BIGINT  NOT NULL REFERENCES hissedarlar(id),
    hisse_sayisi    INTEGER NOT NULL DEFAULT 1,
    tutar           DOUBLE PRECISION NOT NULL,
    odendi          BOOLEAN NOT NULL DEFAULT false,
    odeme_tarihi    DATE,
    aciklama        TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Hissedar Cüzdanları
CREATE TABLE IF NOT EXISTS hissedar_cuzdanlari (
    id          BIGSERIAL PRIMARY KEY,
    hissedar_id BIGINT  NOT NULL REFERENCES hissedarlar(id) ON DELETE CASCADE,
    donem_id    BIGINT  REFERENCES donemler(id),
    tarih       DATE    NOT NULL DEFAULT CURRENT_DATE,
    bilgi       TEXT    NOT NULL,
    borc        DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    alacak      DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    bakiye      DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Hisse Satışları
CREATE TABLE IF NOT EXISTS hisse_satislari (
    id                  BIGSERIAL PRIMARY KEY,
    hisse_id            BIGINT  NOT NULL REFERENCES hisseler(id) ON DELETE CASCADE,
    hissedar_id         BIGINT  NOT NULL REFERENCES hissedarlar(id),
    kasa_id             BIGINT  NOT NULL REFERENCES kasalar(id),
    satis_tutari        DOUBLE PRECISION NOT NULL,
    tarih               DATE    NOT NULL DEFAULT CURRENT_DATE,
    tamamlandi          BOOLEAN NOT NULL DEFAULT false,
    tamamlanma_tarihi   DATE,
    iptal               BOOLEAN NOT NULL DEFAULT false,
    aciklama            TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Hisse Satış Ödemeleri
CREATE TABLE IF NOT EXISTS hisse_satis_odemeleri (
    id          BIGSERIAL PRIMARY KEY,
    satis_id    BIGINT  NOT NULL REFERENCES hisse_satislari(id) ON DELETE CASCADE,
    tutar       DOUBLE PRECISION NOT NULL,
    tarih       DATE    NOT NULL DEFAULT CURRENT_DATE,
    aciklama    TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Gelir/Gider Kategorileri
CREATE TABLE IF NOT EXISTS gelir_gider_kategorileri (
    id          BIGSERIAL PRIMARY KEY,
    ad          TEXT    NOT NULL,
    tip         TEXT    NOT NULL CHECK(tip IN ('gelir', 'gider')),
    aciklama    TEXT,
    aktif       BOOLEAN NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Gelir/Gider Kayıtları
CREATE TABLE IF NOT EXISTS gelir_gider_kayitlari (
    id                  BIGSERIAL PRIMARY KEY,
    kasa_id             BIGINT  NOT NULL REFERENCES kasalar(id),
    kategori_id         BIGINT  NOT NULL REFERENCES gelir_gider_kategorileri(id),
    tarih               DATE    NOT NULL DEFAULT CURRENT_DATE,
    tutar               DOUBLE PRECISION NOT NULL,
    aciklama            TEXT    NOT NULL,
    kasa_hareketi_id    BIGINT  REFERENCES kasa_hareketleri(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- İndeksler
CREATE INDEX IF NOT EXISTS idx_kasa_hareketleri_kasa_id ON kasa_hareketleri(kasa_id);
CREATE INDEX IF NOT EXISTS idx_hissedarlar_kasa_id ON hissedarlar(kasa_id);
CREATE INDEX IF NOT EXISTS idx_hisse_atamalari_hisse_id ON hisse_atamalari(hisse_id);
CREATE INDEX IF NOT EXISTS idx_hisse_atamalari_hissedar_id ON hisse_atamalari(hissedar_id);
CREATE INDEX IF NOT EXISTS idx_donem_aidat_borclari_donem_id ON donem_aidat_borclari(donem_id);
CREATE INDEX IF NOT EXISTS idx_donem_aidat_borclari_hissedar_id ON donem_aidat_borclari(hissedar_id);
CREATE INDEX IF NOT EXISTS idx_gelir_gider_kayitlari_tarih ON gelir_gider_kayitlari(tarih);
