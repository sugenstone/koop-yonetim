-- Kullanıcı kayıt/onay akışı
-- onay_durumu: 'beklemede' (yeni kayıt), 'onaylanmis' (admin onayladı), 'reddedilmis'

ALTER TABLE kullanicilar
    ADD COLUMN IF NOT EXISTS onay_durumu TEXT NOT NULL DEFAULT 'onaylanmis'
        CHECK (onay_durumu IN ('beklemede', 'onaylanmis', 'reddedilmis'));

ALTER TABLE kullanicilar
    ADD COLUMN IF NOT EXISTS kayit_tarihi TIMESTAMPTZ NOT NULL DEFAULT NOW();

ALTER TABLE kullanicilar
    ADD COLUMN IF NOT EXISTS onaylayan_id BIGINT REFERENCES kullanicilar(id) ON DELETE SET NULL;

ALTER TABLE kullanicilar
    ADD COLUMN IF NOT EXISTS onay_tarihi TIMESTAMPTZ;

-- Mevcut admin zaten onaylı
UPDATE kullanicilar SET onay_durumu = 'onaylanmis' WHERE id = 1 AND onay_durumu IS NULL;

CREATE INDEX IF NOT EXISTS idx_kullanicilar_onay_durumu ON kullanicilar(onay_durumu);


-- Onay yetkisi eklentisi
INSERT INTO izinler (anahtar, kategori, aciklama) VALUES
    ('kullanici.onayla', 'kullanici', 'Bekleyen kullanici kayitlarini onayla/reddet')
ON CONFLICT (anahtar) DO NOTHING;
