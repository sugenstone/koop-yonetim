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

// â”€â”€â”€ Router â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/giris", post(login))
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
    // KullanÄ±cÄ±yÄ± e-posta ile bul
    let kullanici = sqlx::query_as::<_, Kullanici>(
        "SELECT id, ad, email, rol, aktif, created_at
         FROM kullanicilar WHERE email = $1 AND aktif = true"
    )
    .bind(&input.email)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::BadRequest("E-posta veya ÅŸifre hatalÄ±".to_string()))?;

    // Åifre hash'ini ayrÄ± al
    let sifre_hash: String = sqlx::query_scalar(
        "SELECT sifre_hash FROM kullanicilar WHERE id = $1"
    )
    .bind(kullanici.id)
    .fetch_one(&pool)
    .await?;

    let dogru = bcrypt::verify(&input.sifre, &sifre_hash)
        .map_err(|e| AppError::Internal(e.into()))?;

    if !dogru {
        return Err(AppError::BadRequest("E-posta veya ÅŸifre hatalÄ±".to_string()));
    }

    // JWT Ã¼ret
    let token = token_olustur(kullanici.id, &kullanici.email, &kullanici.rol)
        .map_err(AppError::Internal)?;

    Ok(Json(LoginResponse { token, kullanici }))
}

