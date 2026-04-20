# Kooperatif Yönetim Paneli

Kooperatif / şirket ortaklık yönetimi için SvelteKit + Rust (Axum) + PostgreSQL tabanlı tam yığın yönetim uygulaması. Aynı kod tabanı hem **web** (SvelteKit) hem de **masaüstü** (Tauri) olarak çalıştırılabilir.

## İçindekiler

- [Özellikler](#özellikler)
- [Mimari](#mimari)
- [Teknoloji Yığını](#teknoloji-yığını)
- [Kurulum](#kurulum)
- [Geliştirme](#geliştirme)
- [Yetkilendirme Sistemi](#yetkilendirme-sistemi)
- [Proje Yapısı](#proje-yapısı)
- [Üretim Derlemesi](#üretim-derlemesi)

## Özellikler

- **Hissedar yönetimi** — kayıt, cüzdan bakiyesi, para ekleme / tahsilat geçmişi
- **Hisse yönetimi** — hisse tanımı, hissedara atama, sisteme geri satış, hissedarlar arası transfer, ödeme takibi
- **Kasa ve transferler** — çoklu para birimi desteği, kasalar arası kur çevrimli transfer, hareket geçmişi
- **Dönem & toplantı** — dönem bazlı aidat tanımlama, toplantı ve karar kayıtları
- **Aidat borcu ve tahsilat** — dönem bazlı otomatik borç oluşturma, cüzdandan tahsilat
- **Gelir / gider** — kategori bazlı işletme hareketleri
- **Rol tabanlı yetkilendirme** — CASL benzeri `<Can>` bileşeni ile UI seviyesinde ve `require_izin` ile backend seviyesinde zorunlu yetki kontrolü
- **PDF dışa aktarım** — liste ve detay sayfaları için
- **Karanlık tema**, çoklu dil hazırlığı, toast bildirimleri

## Mimari

```
┌────────────────────┐       HTTP/JSON + JWT      ┌────────────────────┐
│  SvelteKit (web)   │  ─────────────────────────▶│   Axum API (Rust)  │
│  Tauri (desktop)   │                            │        │           │
└────────────────────┘                            │        ▼           │
                                                  │   PostgreSQL       │
                                                  └────────────────────┘
```

- Frontend, `src/lib/api-client.ts` içindeki **komut → HTTP rota** tablosu aracılığıyla backend'e istek gönderir. Aynı çağrı yüzeyi Tauri modunda yerel `invoke`'a da yönlendirilebilir.
- Tüm yazma uçları JWT ile korumalı, tüm yetki uçları `require_izin(&pool, "alan.islem")` ile yetki kontrolünden geçer.

## Teknoloji Yığını

**Frontend**
- Svelte 5 (runes) + SvelteKit 2
- TailwindCSS 4 + Flowbite-Svelte + flowbite-svelte-admin-dashboard
- ApexCharts, svelte-sonner (toast)
- Vitest + Playwright

**Backend** (`api/`)
- Rust + Axum 0.8
- SQLx (PostgreSQL, runtime-tokio-rustls)
- JWT (`jsonwebtoken`) + bcrypt
- Lettre (SMTP) — kayıt / onay bildirimleri

**Masaüstü** (`src-tauri/`)
- Tauri 2 + SQLite (gömülü veritabanı modu)

## Kurulum

### Gereksinimler

- Node.js 20+ ve pnpm
- Rust (stable) + Cargo
- PostgreSQL 14+
- (Opsiyonel) Tauri CLI — `cargo install tauri-cli`

### 1) Veritabanı

```bash
createdb koop_db
psql -c "CREATE USER koop_user WITH PASSWORD 'guclu_sifre';"
psql -c "GRANT ALL PRIVILEGES ON DATABASE koop_db TO koop_user;"
```

### 2) Backend (`api/`)

```bash
cd api
cp .env.example .env        # DATABASE_URL, JWT_SECRET, SMTP ayarlarını doldur
cargo run                   # migration'lar otomatik uygulanır, ilk admin seed edilir
```

Varsayılan ilk admin bilgileri `migrations/0002_seed_admin.sql` içinde tanımlıdır.

### 3) Frontend

```bash
pnpm install
pnpm dev                    # http://localhost:5173
```

`src/lib/api-client.ts` içindeki `API_BASE_URL` varsayılan olarak `http://localhost:3000` üzerine bağlanır.

## Geliştirme

| Komut | Açıklama |
| --- | --- |
| `pnpm dev` | SvelteKit dev sunucusu |
| `pnpm build` | Web üretim derlemesi |
| `pnpm check` | Svelte + TypeScript tip kontrolü |
| `pnpm lint` | Prettier + ESLint |
| `pnpm test` | Vitest + Playwright |
| `pnpm tauri:dev` | Masaüstü modu (geliştirme) |
| `pnpm tauri:build` | Masaüstü kurulum paketi |
| `cargo run` *(api/)* | Backend geliştirme sunucusu |
| `cargo build --release` *(api/)* | Backend üretim ikilisi |

## Yetkilendirme Sistemi

İzinler `alan.islem` formatındadır ve `api/migrations/0003_permissions.sql` içinde tanımlıdır.

| Alan | Anahtarlar |
| --- | --- |
| `kasa` | `goruntule`, `olustur`, `duzenle`, `hareket`, `transfer` |
| `hissedar` | `goruntule`, `olustur`, `duzenle`, `sil`, `cuzdan` |
| `hisse` | `goruntule`, `yonet`, `satis`, `transfer` |
| `donem` | `goruntule`, `yonet` |
| `toplanti` | `yonet` |
| `borc` | `goruntule`, `yonet` |
| `gelir_gider` | `goruntule`, `yonet` |

**Varsayılan roller:** `admin` (tam yetki, bypass), `muhasebe` (silme hariç CRUD), `uye` (sadece görüntüleme), `izleyici` (salt okunur).

### UI

```svelte
<Can permission="kasa.transfer">
  <Button onclick={transferAc}>Transfer</Button>
</Can>
```

### Backend

```rust
async fn create_hareket(user: AuthUser, State(pool): State<PgPool>, ...) -> AppResult<...> {
    user.require_izin(&pool, "kasa.hareket").await?;
    // ...
}
```

403 yanıtları, `api-client.ts` tarafından otomatik olarak `notify.forbidden()` toast'ı ile gösterilir.

## Proje Yapısı

```
.
├── api/                     # Axum backend
│   ├── migrations/          # 0001 şema · 0002 admin · 0003 izinler · 0004 onay
│   └── src/
│       ├── auth/            # JWT + require_izin yardımcıları
│       ├── routes/          # kasa, hissedar, hisse, donem, gelir_gider, izin ...
│       ├── errors.rs        # AppError / AppResult
│       ├── mail.rs          # SMTP bildirim gönderimi
│       └── main.rs
├── src/                     # SvelteKit uygulaması
│   ├── lib/
│   │   ├── Can.svelte       # CASL benzeri izin kapısı
│   │   ├── toast.ts         # notify.success / error / apiError / forbidden
│   │   ├── api-client.ts    # Komut → HTTP rota köprüsü
│   │   └── tauri-api.ts     # Tip güvenli backend API sarmalayıcısı
│   └── routes/(sidebar)/
│       ├── dashboard/
│       ├── kasa/            # ve [id]
│       ├── hissedar/        # ve [id]
│       ├── hisse/           # ve [id]
│       ├── donem/           # ve [id]
│       ├── gelir-gider/
│       ├── borclar/
│       ├── kullanicilar/
│       └── roller/
└── src-tauri/               # Tauri kabuk + SQLite komutları
```

## Üretim Derlemesi

### Web + API

```bash
# Frontend
pnpm build                  # build/ dizini

# Backend
cd api && cargo build --release
```

Üretim notları:

- `JWT_SECRET` mutlaka güçlü rastgele bir değer olmalı
- PostgreSQL bağlantısı TLS üzerinden yapılmalı
- API'nin önünde TLS sonlandırıcı (nginx/caddy) olmalı
- `tower-http` CORS ayarları dağıtım alan adına göre kısıtlanmalı

### Masaüstü

```bash
pnpm tauri:build
# src-tauri/target/release/bundle/ altında kurulum paketi
```

## Lisans

MIT
# flowbite-svelte-admin-dashboard

- [demo](https://flowbite-svelte-admin-dashboard.vercel.app/)
- [repo](https://github.com/themesberg/flowbite-svelte-admin-dashboard)

## Installation

```bash
# create a new project in my-app
# install tailwindcss
npx sv create my-app
cd my-app
pnpm i -D flowbite-svelte-admin-dashboard
# it's a good idea to update all dependencies
pnpm update
# run the server
pnpm dev
```
