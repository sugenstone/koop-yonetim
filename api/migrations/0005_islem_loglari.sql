-- İşlem (audit) logları
CREATE TABLE IF NOT EXISTS islem_loglari (
    id BIGSERIAL PRIMARY KEY,
    tarih TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    kullanici_id BIGINT REFERENCES kullanicilar(id) ON DELETE SET NULL,
    kullanici_email TEXT,
    rol TEXT,
    yontem TEXT NOT NULL,
    yol TEXT NOT NULL,
    durum_kodu INT NOT NULL,
    ip TEXT,
    user_agent TEXT,
    sure_ms INT,
    hata TEXT
);

CREATE INDEX IF NOT EXISTS idx_islem_loglari_tarih ON islem_loglari(tarih DESC);
CREATE INDEX IF NOT EXISTS idx_islem_loglari_kullanici ON islem_loglari(kullanici_id);
CREATE INDEX IF NOT EXISTS idx_islem_loglari_yol ON islem_loglari(yol);

INSERT INTO izinler (anahtar, kategori, aciklama) VALUES
    ('log.goruntule', 'sistem', 'Islem loglarini goruntule')
ON CONFLICT (anahtar) DO NOTHING;

-- Admin rolüne logu görme izni ver (backstop zaten admin'e tüm izin verir ama açıkça ekleyelim)
INSERT INTO rol_izinleri (rol, izin_id)
SELECT 'admin', id FROM izinler WHERE anahtar = 'log.goruntule'
ON CONFLICT DO NOTHING;
