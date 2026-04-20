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
        .route("/bekleyenler", get(list_bekleyenler))
        .route("/{id}", put(update_kullanici).delete(delete_kullanici))
        .route("/{id}/sifre", put(change_sifre))
        .route("/{id}/onayla", axum::routing::post(onayla))
        .route("/{id}/reddet", axum::routing::post(reddet))
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

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BekleyenKullanici {
    pub id: i64,
    pub ad: String,
    pub email: String,
    pub rol: String,
    pub onay_durumu: String,
    pub kayit_tarihi: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct OnaylaInput {
    pub rol: String,
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

async fn list_kullanicilar(
    user: AuthUser,
    State(pool): State<PgPool>,
) -> AppResult<Json<Vec<Kullanici>>> {
    user.require_izin(&pool, "kullanici.goruntule").await?;
    let rows = sqlx::query_as::<_, Kullanici>(
        "SELECT id, ad, email, rol, aktif, created_at
         FROM kullanicilar WHERE onay_durumu = 'onaylanmis' ORDER BY id",
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
    user.require_izin(&pool, "kullanici.olustur").await?;

    let rol_gecerli = ["admin", "muhasebe", "uye", "izleyici"].contains(&input.rol.as_str());
    if !rol_gecerli {
        return Err(AppError::BadRequest("Gecersiz rol".into()));
    }

    let hash = bcrypt::hash(&input.sifre, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(e.into()))?;

    let k = sqlx::query_as::<_, Kullanici>(
        "INSERT INTO kullanicilar (ad, email, sifre_hash, rol, aktif, onay_durumu)
         VALUES ($1, $2, $3, $4, true, 'onaylanmis')
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
    user.require_izin(&pool, "kullanici.duzenle").await?;

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
    user.require_izin(&pool, "kullanici.sil").await?;
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

// --- Onay akisi ----------------------------------------------------------------


async fn list_bekleyenler(
    user: AuthUser,
    State(pool): State<PgPool>,
) -> AppResult<Json<Vec<BekleyenKullanici>>> {
    user.require_izin(&pool, "kullanici.onayla").await?;
    let rows = sqlx::query_as::<_, BekleyenKullanici>(
        "SELECT id, ad, email, rol, onay_durumu, kayit_tarihi
         FROM kullanicilar
         WHERE onay_durumu = 'beklemede'
         ORDER BY kayit_tarihi DESC",
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

async fn onayla(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(input): Json<OnaylaInput>,
) -> AppResult<Json<Kullanici>> {
    user.require_izin(&pool, "kullanici.onayla").await?;

    if !["admin", "muhasebe", "uye", "izleyici"].contains(&input.rol.as_str()) {
        return Err(AppError::BadRequest("Gecersiz rol".into()));
    }

    let k = sqlx::query_as::<_, Kullanici>(
        "UPDATE kullanicilar
            SET rol = $1, aktif = true, onay_durumu = 'onaylanmis',
                onaylayan_id = $2, onay_tarihi = NOW(), updated_at = NOW()
          WHERE id = $3 AND onay_durumu = 'beklemede'
         RETURNING id, ad, email, rol, aktif, created_at",
    )
    .bind(&input.rol)
    .bind(user.id())
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Bekleyen kullanici bulunamadi".into()))?;

    let email = k.email.clone();
    let ad = k.ad.clone();
    let rol = k.rol.clone();
    tokio::spawn(async move {
        let body = format!(
            "<h3>Merhaba {}</h3>\
             <p>Kayit basvurunuz <b>onaylandi</b>.</p>\
             <p>Rolunuz: <b>{}</b></p>\
             <p>Simdi giris yapabilirsiniz.</p>",
            ad, rol
        );
        crate::mail::send(&email, "Hesabiniz onaylandi", &body).await;
    });

    Ok(Json(k))
}

async fn reddet(
    user: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<MesajResponse>> {
    user.require_izin(&pool, "kullanici.onayla").await?;

    let row: Option<(String, String)> = sqlx::query_as(
        "UPDATE kullanicilar
            SET onay_durumu = 'reddedilmis', aktif = false,
                onaylayan_id = $1, onay_tarihi = NOW(), updated_at = NOW()
          WHERE id = $2 AND onay_durumu = 'beklemede'
         RETURNING ad, email",
    )
    .bind(user.id())
    .bind(id)
    .fetch_optional(&pool)
    .await?;

    let (ad, email) = row.ok_or_else(|| AppError::NotFound("Bekleyen kullanici bulunamadi".into()))?;

    tokio::spawn(async move {
        let body = format!(
            "<h3>Merhaba {}</h3>\
             <p>Uzgunuz, kayit basvurunuz reddedildi.</p>",
            ad
        );
        crate::mail::send(&email, "Kayit basvurunuz reddedildi", &body).await;
    });

    Ok(Json(MesajResponse { mesaj: "Kullanici kaydi reddedildi".into() }))
}
