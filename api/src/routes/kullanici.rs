use axum::{
    extract::{Path, State},
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::{AuthUser, Kullanici};
use crate::errors::{AppError, AppResult};

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/", get(list_kullanicilar).post(create_kullanici))
        .route("/{id}", put(update_kullanici).delete(delete_kullanici))
        .route("/{id}/sifre", put(change_sifre))
        .with_state(pool)
}

// ─── Inputs ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateKullaniciInput {
    pub ad: String,
    pub email: String,
    pub sifre: String,
    pub rol: String, // admin | muhasebe | uye | izleyici
}

#[derive(Debug, Deserialize)]
pub struct UpdateKullaniciInput {
    pub ad: Option<String>,
    pub email: Option<String>,
    pub rol: Option<String>,
    pub aktif: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ChangeSifreInput {
    pub eski_sifre: Option<String>, // admin kendisi degilse gerek yok
    pub yeni_sifre: String,
}

#[derive(Debug, Serialize)]
struct MesajResponse {
    mesaj: String,
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

async fn list_kullanicilar(
    user: AuthUser,
    State(pool): State<PgPool>,
) -> AppResult<Json<Vec<Kullanici>>> {
    user.require_rol(&["admin"])?;
    let rows = sqlx::query_as::<_, Kullanici>(
        "SELECT id, ad, email, rol, aktif, created_at
         FROM kullanicilar ORDER BY id",
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

async fn create_kullanici(
    user: AuthUser,
    State(pool): State<PgPool>,
    Json(input): Json<CreateKullaniciInput>,
) -> AppResult<Json<Kullanici>> {
    user.require_rol(&["admin"])?;

    let rol_gecerli = ["admin", "muhasebe", "uye", "izleyici"].contains(&input.rol.as_str());
    if !rol_gecerli {
        return Err(AppError::BadRequest("Gecersiz rol".into()));
    }

    let hash = bcrypt::hash(&input.sifre, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(e.into()))?;

    let k = sqlx::query_as::<_, Kullanici>(
        "INSERT INTO kullanicilar (ad, email, sifre_hash, rol, aktif)
         VALUES ($1, $2, $3, $4, true)
         RETURNING id, ad, email, rol, aktif, created_at",
    )
    .bind(input.ad)
    .bind(input.email)
    .bind(hash)
    .bind(input.rol)
    .fetch_one(&pool)
    .await?;

    Ok(Json(k))
}

async fn update_kullanici(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateKullaniciInput>,
) -> AppResult<Json<Kullanici>> {
    user.require_rol(&["admin"])?;

    if let Some(rol) = &input.rol {
        if !["admin", "muhasebe", "uye", "izleyici"].contains(&rol.as_str()) {
            return Err(AppError::BadRequest("Gecersiz rol".into()));
        }
    }

    let k = sqlx::query_as::<_, Kullanici>(
        "UPDATE kullanicilar SET
            ad    = COALESCE($1, ad),
            email = COALESCE($2, email),
            rol   = COALESCE($3, rol),
            aktif = COALESCE($4, aktif),
            updated_at = NOW()
         WHERE id = $5
         RETURNING id, ad, email, rol, aktif, created_at",
    )
    .bind(input.ad)
    .bind(input.email)
    .bind(input.rol)
    .bind(input.aktif)
    .bind(id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(k))
}

async fn delete_kullanici(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<MesajResponse>> {
    user.require_rol(&["admin"])?;
    if user.id() == id {
        return Err(AppError::BadRequest("Kendi hesabinizi pasifize edemezsiniz".into()));
    }
    sqlx::query("UPDATE kullanicilar SET aktif = false, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(Json(MesajResponse { mesaj: "Kullanici pasifize edildi".into() }))
}

async fn change_sifre(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(input): Json<ChangeSifreInput>,
) -> AppResult<Json<MesajResponse>> {
    let is_admin = user.rol() == "admin";
    let is_self = user.id() == id;

    if !is_admin && !is_self {
        return Err(AppError::Forbidden("Sadece kendi sifrenizi degistirebilirsiniz".into()));
    }

    // Kendi sifresini degistirirken eski sifre zorunlu
    if is_self && !is_admin {
        let eski = input.eski_sifre.as_ref()
            .ok_or_else(|| AppError::BadRequest("Eski sifre gerekli".into()))?;
        let mevcut: String = sqlx::query_scalar("SELECT sifre_hash FROM kullanicilar WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await?;
        let ok = bcrypt::verify(eski, &mevcut).map_err(|e| AppError::Internal(e.into()))?;
        if !ok {
            return Err(AppError::BadRequest("Eski sifre hatali".into()));
        }
    }

    if input.yeni_sifre.len() < 6 {
        return Err(AppError::BadRequest("Sifre en az 6 karakter olmali".into()));
    }

    let yeni_hash = bcrypt::hash(&input.yeni_sifre, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(e.into()))?;

    sqlx::query("UPDATE kullanicilar SET sifre_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(yeni_hash)
        .bind(id)
        .execute(&pool)
        .await?;

    Ok(Json(MesajResponse { mesaj: "Sifre guncellendi".into() }))
}
