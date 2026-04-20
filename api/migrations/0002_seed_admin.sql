-- İlk admin kullanıcısı. Giriş: admin@koop.local / admin123
-- GÜVENLİK: İlk girişten sonra bu şifreyi MUTLAKA değiştirin.
INSERT INTO kullanicilar (ad, email, sifre_hash, rol, aktif)
VALUES (
    'Admin',
    'admin@koop.local',
    '$2b$12$pkA/tqsrO/mqNc4UJkUaD.NB84wX.Ex0QxHKrfMp7EHBbmk.LmqfC',
    'admin',
    true
)
ON CONFLICT (email) DO NOTHING;
