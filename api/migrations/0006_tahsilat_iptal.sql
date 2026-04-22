-- Tahsilat (cüzdan para ekleme) işlemlerini birbirine bağlamak için operation_id
-- Bir tahsilat iptal edildiğinde aynı operation_id'ye sahip tüm kayıtlar
-- (cüzdan, kasa, dönem borçları) geri alınabilir.

ALTER TABLE hissedar_cuzdanlari
    ADD COLUMN IF NOT EXISTS operation_id UUID;

ALTER TABLE kasa_hareketleri
    ADD COLUMN IF NOT EXISTS operation_id UUID;

ALTER TABLE donem_aidat_borclari
    ADD COLUMN IF NOT EXISTS odeme_operation_id UUID;

CREATE INDEX IF NOT EXISTS idx_cuzdan_operation_id
    ON hissedar_cuzdanlari(operation_id) WHERE operation_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_kasa_hareketleri_operation_id
    ON kasa_hareketleri(operation_id) WHERE operation_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_donem_borc_odeme_operation_id
    ON donem_aidat_borclari(odeme_operation_id) WHERE odeme_operation_id IS NOT NULL;

-- Tahsilat iptali için yeni izin
INSERT INTO izinler (anahtar, kategori, aciklama)
VALUES ('hissedar.cuzdan.iptal', 'hissedar', 'Hissedar cüzdan tahsilatını iptal et')
ON CONFLICT (anahtar) DO NOTHING;

-- Admin rolüne ata
INSERT INTO rol_izinleri (rol, izin_id)
SELECT 'admin', id FROM izinler WHERE anahtar = 'hissedar.cuzdan.iptal'
ON CONFLICT DO NOTHING;
