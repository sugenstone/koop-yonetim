-- Rol ve izin yönetimi
-- Roller sabittir (admin, muhasebe, uye, izleyici).
-- İzinler "kategori.islem" formatında (ör. "kasa.create", "kullanici.manage")
-- role_permissions: hangi rolün hangi izinlere sahip olduğu

CREATE TABLE IF NOT EXISTS izinler (
    id          BIGSERIAL PRIMARY KEY,
    anahtar     TEXT NOT NULL UNIQUE,   -- ör. "kasa.create"
    kategori    TEXT NOT NULL,          -- ör. "kasa"
    aciklama    TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_izinler_kategori ON izinler(kategori);

CREATE TABLE IF NOT EXISTS rol_izinleri (
    rol         TEXT NOT NULL,          -- admin | muhasebe | uye | izleyici
    izin_id     BIGINT NOT NULL REFERENCES izinler(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (rol, izin_id)
);

CREATE INDEX IF NOT EXISTS idx_rol_izinleri_rol ON rol_izinleri(rol);

-- ─── Seed: kanonik izin listesi ─────────────────────────────────────────────

INSERT INTO izinler (anahtar, kategori, aciklama) VALUES
    -- Kullanıcı yönetimi
    ('kullanici.goruntule',  'kullanici', 'Kullanıcı listesini görüntüle'),
    ('kullanici.olustur',    'kullanici', 'Yeni kullanıcı oluştur'),
    ('kullanici.duzenle',    'kullanici', 'Kullanıcı bilgilerini düzenle'),
    ('kullanici.sil',        'kullanici', 'Kullanıcıyı pasifize et'),
    ('kullanici.sifre',      'kullanici', 'Başka kullanıcının şifresini değiştir'),
    -- Kasa
    ('kasa.goruntule',       'kasa',      'Kasa listesini görüntüle'),
    ('kasa.olustur',         'kasa',      'Yeni kasa oluştur'),
    ('kasa.duzenle',         'kasa',      'Kasa düzenle'),
    ('kasa.sil',             'kasa',      'Kasa sil'),
    ('kasa.hareket',         'kasa',      'Kasa hareketi ekle/sil'),
    ('kasa.transfer',        'kasa',      'Kasalar arası transfer'),
    -- Hissedar
    ('hissedar.goruntule',   'hissedar',  'Hissedarları görüntüle'),
    ('hissedar.olustur',     'hissedar',  'Hissedar ekle'),
    ('hissedar.duzenle',     'hissedar',  'Hissedar düzenle'),
    ('hissedar.sil',         'hissedar',  'Hissedar sil'),
    ('hissedar.cuzdan',      'hissedar',  'Hissedar cüzdan işlemleri'),
    -- Dönem & toplantı
    ('donem.goruntule',      'donem',     'Dönemleri görüntüle'),
    ('donem.yonet',          'donem',     'Dönem oluştur/düzenle/sil'),
    ('toplanti.yonet',       'donem',     'Toplantı ve karar yönetimi'),
    -- Hisse
    ('hisse.goruntule',      'hisse',     'Hisseleri görüntüle'),
    ('hisse.yonet',          'hisse',     'Hisse oluştur/sil/ata'),
    ('hisse.satis',          'hisse',     'Hisse satış işlemleri'),
    ('hisse.transfer',       'hisse',     'Hisse transferi'),
    -- Aidat/borç
    ('borc.goruntule',       'borc',      'Borçları görüntüle'),
    ('borc.yonet',           'borc',      'Borç oluştur/tahsil'),
    -- Gelir/gider
    ('gelir_gider.goruntule','gelir_gider','Gelir-gider kayıtlarını görüntüle'),
    ('gelir_gider.yonet',    'gelir_gider','Gelir-gider kayıt ve kategori yönetimi'),
    -- Rol/izin yönetimi
    ('rol.yonet',            'sistem',    'Rollere izin ata/kaldır')
ON CONFLICT (anahtar) DO NOTHING;

-- ─── Seed: varsayılan rol-izin ataması ──────────────────────────────────────

-- admin: tüm izinler
INSERT INTO rol_izinleri (rol, izin_id)
SELECT 'admin', id FROM izinler
ON CONFLICT DO NOTHING;

-- muhasebe: kasa/hissedar/gelir-gider tam, hisse görüntüleme, borç yönetim
INSERT INTO rol_izinleri (rol, izin_id)
SELECT 'muhasebe', id FROM izinler WHERE anahtar IN (
    'kasa.goruntule','kasa.olustur','kasa.duzenle','kasa.hareket','kasa.transfer',
    'hissedar.goruntule','hissedar.olustur','hissedar.duzenle','hissedar.cuzdan',
    'donem.goruntule','toplanti.yonet',
    'hisse.goruntule',
    'borc.goruntule','borc.yonet',
    'gelir_gider.goruntule','gelir_gider.yonet'
)
ON CONFLICT DO NOTHING;

-- uye: görüntüleme + kendi hissesi
INSERT INTO rol_izinleri (rol, izin_id)
SELECT 'uye', id FROM izinler WHERE anahtar IN (
    'kasa.goruntule',
    'hissedar.goruntule',
    'donem.goruntule',
    'hisse.goruntule',
    'borc.goruntule',
    'gelir_gider.goruntule'
)
ON CONFLICT DO NOTHING;

-- izleyici: salt okunur
INSERT INTO rol_izinleri (rol, izin_id)
SELECT 'izleyici', id FROM izinler WHERE anahtar LIKE '%.goruntule'
ON CONFLICT DO NOTHING;
