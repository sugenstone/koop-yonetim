# Koop API — Axum REST API

Kooperatif yonetim sistemi icin PostgreSQL + Axum REST API.

## Gereksinimler

- Rust 1.75+
- PostgreSQL 14+

## Lokal gelistirme

```bash
# 1. PostgreSQL'de veritabani olustur
psql -U postgres -c "CREATE DATABASE koop_db;"
psql -U postgres -c "CREATE USER koop_user WITH PASSWORD 'guclu_sifre';"
psql -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE koop_db TO koop_user;"
psql -U postgres -d koop_db -c "GRANT ALL ON SCHEMA public TO koop_user;"

# 2. .env dosyasi olustur
cp .env.example .env
# JWT_SECRET degerini guclu bir random string ile degistir

# 3. Derle ve calistir
cargo run
# Sunucu http://localhost:3000 adresinde baslar
# Migration'lar otomatik calisir
```

## Varsayilan giris

Migration `0002_seed_admin.sql` ilk admin kullaniciyi olusturur:

- E-posta: `admin@koop.local`
- Sifre: `admin123`

**GUVENLIK: Ilk girisin ardindan sifreyi mutlaka degistirin!**

## Endpoint'ler

| Metod  | Yol                           | Aciklama                       | Yetki   |
|--------|-------------------------------|--------------------------------|---------|
| GET    | `/health`                     | Saglik kontrolu                | Herkes  |
| POST   | `/api/auth/giris`             | Giris, JWT token doner         | Herkes  |
| GET    | `/api/kullanicilar`           | Kullanici listesi              | admin   |
| POST   | `/api/kullanicilar`           | Kullanici olustur              | admin   |
| PUT    | `/api/kullanicilar/{id}`      | Kullanici guncelle             | admin   |
| DELETE | `/api/kullanicilar/{id}`      | Kullanici pasifize et          | admin   |
| PUT    | `/api/kullanicilar/{id}/sifre`| Sifre degistir                 | self    |
| ...    | `/api/kasalar`                | Kasa CRUD                      | -       |
| ...    | `/api/hissedarlar`            | Hissedar CRUD                  | -       |
| ...    | `/api/donemler`               | Donem/Toplanti/Karar           | -       |
| ...    | `/api/hisseler`               | Hisse/Atama/Satis              | -       |
| ...    | `/api/gelir-gider`            | Gelir/Gider islemleri          | -       |

JWT token'i tum `/api/*` isteklerinde `Authorization: Bearer <token>` header'i ile gonderin.

## Production build

```bash
cargo build --release
# ./target/release/koop-api
```

## VDS deploy

Bkz: `deploy.md`
