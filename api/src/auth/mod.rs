use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    routing::post,
    Json, Router,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::env;
use crate::errors::{AppError, AppResult};

// â”€â”€â”€ JWT Claims â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64,          // kullanÄ±cÄ± id
    pub email: String,
    pub rol: String,
    pub exp: i64,          // unix timestamp
}

// â”€â”€â”€ KullanÄ±cÄ± Modeli â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Kullanici {
    pub id: i64,
    pub ad: String,
    pub email: String,
    pub rol: String,
    pub aktif: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// â”€â”€â”€ Request/Response â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[derive(Debug, Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub sifre: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub kullanici: Kullanici,
}

#[derive(Debug, Deserialize)]
pub struct RegisterInput {
    pub ad: String,
    pub email: String,
    pub sifre: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub mesaj: String,
}

// â”€â”€â”€ Router â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/giris", post(login))
        .route("/kayit", post(register))
        .with_state(pool)
}

// â”€â”€â”€ JWT yardÄ±mcÄ±larÄ± â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

pub fn jwt_secret() -> String {
    env::var("JWT_SECRET").unwrap_or_else(|_| "koop-gizli-anahtar-degistir".to_string())
}

pub fn token_olustur(kullanici_id: i64, email: &str, rol: &str) -> anyhow::Result<String> {
    let bitis = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .unwrap()
        .timestamp();

    let claims = Claims {
        sub: kullanici_id,
        email: email.to_string(),
        rol: rol.to_string(),
        exp: bitis,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )?;

    Ok(token)
}

pub fn token_dogrula(token: &str) -> Result<Claims, AppError> {
    let verisi = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;

    Ok(verisi.claims)
}

// â”€â”€â”€ Axum extractor: Authorization header'dan Claims al â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

pub struct AuthUser(pub Claims);

impl AuthUser {
    pub fn id(&self) -> i64 { self.0.sub }
    pub fn rol(&self) -> &str { &self.0.rol }

    /// Belirtilen rollerden birine sahip mi? Degilse Forbidden doner.
    pub fn require_rol(&self, izin_verilen: &[&str]) -> AppResult<()> {
        if izin_verilen.iter().any(|r| *r == self.0.rol) {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!(
                "Bu islem icin yetkiniz yok (rol: {})", self.0.rol
            )))
        }
    }

    /// İzin tabanlı kontrol: rol_izinleri tablosundan okur.
    /// Admin rolü her zaman izinlidir (güvenlik backstop).
    pub async fn require_izin(&self, pool: &sqlx::PgPool, anahtar: &str) -> AppResult<()> {
        if self.0.rol == "admin" {
            return Ok(());
        }
        let var: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM rol_izinleri ri
             JOIN izinler i ON i.id = ri.izin_id
             WHERE ri.rol = $1 AND i.anahtar = $2
             LIMIT 1",
        )
        .bind(&self.0.rol)
        .bind(anahtar)
        .fetch_optional(pool)
        .await?;
        if var.is_some() {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!(
                "'{}' izni gerekli (rol: {})", anahtar, self.0.rol
            )))
        }
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        let claims = token_dogrula(token)?;
        Ok(AuthUser(claims))
    }
}

// â”€â”€â”€ Handler'lar â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

async fn login(
    State(pool): State<PgPool>,
    Json(input): Json<LoginInput>,
) -> AppResult<Json<LoginResponse>> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: i64,
        ad: String,
        email: String,
        rol: String,
        aktif: bool,
        created_at: chrono::DateTime<chrono::Utc>,
        onay_durumu: String,
        sifre_hash: String,
    }

    let row: Option<Row> = sqlx::query_as::<_, Row>(
        "SELECT id, ad, email, rol, aktif, created_at, onay_durumu, sifre_hash
         FROM kullanicilar WHERE email = $1"
    )
    .bind(&input.email)
    .fetch_optional(&pool)
    .await?;

    let r = row.ok_or_else(|| AppError::BadRequest("E-posta veya sifre hatali".into()))?;

    if r.onay_durumu == "beklemede" {
        return Err(AppError::Forbidden(
            "Hesabiniz yonetici onayi bekliyor.".into()
        ));
    }
    if r.onay_durumu == "reddedilmis" {
        return Err(AppError::Forbidden("Kayit basvurunuz reddedildi.".into()));
    }
    if !r.aktif {
        return Err(AppError::Forbidden("Hesabiniz pasif durumda.".into()));
    }

    let dogru = bcrypt::verify(&input.sifre, &r.sifre_hash)
        .map_err(|e| AppError::Internal(e.into()))?;

    if !dogru {
        return Err(AppError::BadRequest("E-posta veya sifre hatali".into()));
    }

    let kullanici = Kullanici {
        id: r.id, ad: r.ad, email: r.email.clone(),
        rol: r.rol.clone(), aktif: r.aktif, created_at: r.created_at,
    };

    let token = token_olustur(r.id, &r.email, &r.rol)
        .map_err(AppError::Internal)?;

    Ok(Json(LoginResponse { token, kullanici }))
}

// --- Kayit (public) -----------------------------------------------------------

async fn register(
    State(pool): State<PgPool>,
    Json(input): Json<RegisterInput>,
) -> AppResult<Json<RegisterResponse>> {
    if input.ad.trim().len() < 2 {
        return Err(AppError::BadRequest("Ad en az 2 karakter olmali".into()));
    }
    if !input.email.contains('@') {
        return Err(AppError::BadRequest("Gecerli bir e-posta girin".into()));
    }
    if input.sifre.len() < 6 {
        return Err(AppError::BadRequest("Sifre en az 6 karakter olmali".into()));
    }

    let var: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM kullanicilar WHERE email = $1"
    )
    .bind(&input.email)
    .fetch_optional(&pool)
    .await?;

    if var.is_some() {
        return Err(AppError::BadRequest("Bu e-posta zaten kayitli".into()));
    }

    let hash = bcrypt::hash(&input.sifre, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(e.into()))?;

    sqlx::query(
        "INSERT INTO kullanicilar (ad, email, sifre_hash, rol, aktif, onay_durumu)
         VALUES ($1, $2, $3, 'izleyici', false, 'beklemede')"
    )
    .bind(&input.ad)
    .bind(&input.email)
    .bind(&hash)
    .execute(&pool)
    .await?;

    let admin_to = crate::mail::admin_email();
    let ad = input.ad.clone();
    let email = input.email.clone();
    tokio::spawn(async move {
        let body = format!(
            "<h3>Yeni kullanici kaydi</h3>\
             <p><b>Ad:</b> {}</p>\
             <p><b>E-posta:</b> {}</p>\
             <p>Panelde onaylayin.</p>",
            ad, email
        );
        crate::mail::send(&admin_to, "Yeni kullanici onay bekliyor", &body).await;
    });

    Ok(Json(RegisterResponse {
        mesaj: "Kayit basvurunuz alindi. Yonetici onayindan sonra giris yapabilirsiniz.".into()
    }))
}
