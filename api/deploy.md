# Ubuntu VDS Deploy Rehberi

## 1. VDS hazirlik (Ubuntu 22.04/24.04)

```bash
# Sistem guncelle
sudo apt update && sudo apt upgrade -y

# Gerekli paketler
sudo apt install -y build-essential curl git nginx certbot python3-certbot-nginx postgresql postgresql-contrib

# Rust kurulumu (deploy makinesinde derlemek icin)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

## 2. PostgreSQL kurulum

```bash
sudo -u postgres psql <<EOF
CREATE DATABASE koop_db;
CREATE USER koop_user WITH PASSWORD 'buraya-guclu-sifre-yaz';
GRANT ALL PRIVILEGES ON DATABASE koop_db TO koop_user;
\c koop_db
GRANT ALL ON SCHEMA public TO koop_user;
EOF
```

## 3. Uygulama kullanicisi ve dizin

```bash
sudo useradd -r -m -d /opt/koop-api -s /bin/bash koop
sudo mkdir -p /opt/koop-api
sudo chown koop:koop /opt/koop-api
```

## 4. Kodu derle ve kopyala

Lokal makinede (Linux) veya VDS'de:

```bash
git clone https://github.com/sugenstone/koop-yonetim.git
cd koop-yonetim
git checkout api-migration
cd api
cargo build --release

# Binary'i VDS'e kopyala (veya VDS'de zaten buradaysa)
sudo cp target/release/koop-api /opt/koop-api/
sudo cp -r migrations /opt/koop-api/
sudo chown -R koop:koop /opt/koop-api
```

## 5. .env dosyasi olustur

```bash
sudo -u koop tee /opt/koop-api/.env <<EOF
DATABASE_URL=postgresql://koop_user:buraya-guclu-sifre-yaz@localhost:5432/koop_db
JWT_SECRET=$(openssl rand -hex 64)
PORT=3000
RUST_LOG=info
EOF
sudo chmod 600 /opt/koop-api/.env
```

## 6. systemd servisi

```bash
sudo cp /opt/koop-api/koop-api.service /etc/systemd/system/
# (koop-api.service repo'da mevcut, gerekirse elle kopyala)
sudo systemctl daemon-reload
sudo systemctl enable --now koop-api
sudo systemctl status koop-api
sudo journalctl -u koop-api -f    # canli log
```

## 7. nginx reverse proxy

`/etc/nginx/sites-available/koop-api`:

```nginx
server {
    listen 80;
    server_name api.sizinalaniniz.com;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

```bash
sudo ln -s /etc/nginx/sites-available/koop-api /etc/nginx/sites-enabled/
sudo nginx -t && sudo systemctl reload nginx
```

## 8. HTTPS (Let's Encrypt)

```bash
sudo certbot --nginx -d api.sizinalaniniz.com
# Otomatik yenileme: sudo certbot renew --dry-run
```

## 9. Firewall

```bash
sudo ufw allow OpenSSH
sudo ufw allow 'Nginx Full'
sudo ufw enable
```

## 10. Frontend yapilandirmasi

Frontend `.env`:

```
VITE_API_URL=https://api.sizinalaniniz.com
```

## Guncelleme

```bash
cd ~/koop-yonetim && git pull
cd api && cargo build --release
sudo systemctl stop koop-api
sudo cp target/release/koop-api /opt/koop-api/
sudo cp -r migrations /opt/koop-api/
sudo systemctl start koop-api
```

## Yedekleme

Gunluk PostgreSQL yedegi icin cron:

```bash
sudo -u postgres crontab -e
# Ekle:
0 3 * * * pg_dump koop_db | gzip > /var/backups/koop_$(date +\%F).sql.gz
```
