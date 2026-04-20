use axum::{
    extract::{Path, State},
    routing::{get, post, put, delete},
    Json, Router,
};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use crate::errors::AppResult;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Donem {
    pub id: i64,
    pub ay: i32,
    pub yil: i32,
    pub hisse_basi_aidat: f64,
    pub aktif: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDonemInput {
    pub ay: i32,
    pub yil: i32,
    pub hisse_basi_aidat: f64,
}

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/", get(get_donemler).post(create_donem))
        .route("/{id}", get(get_donem).put(update_donem).delete(delete_donem))
        .with_state(pool)
}

async fn get_donemler(State(pool): State<PgPool>) -> AppResult<Json<Vec<Donem>>> {
    let liste = sqlx::query_as::<_, Donem>(
        "SELECT id, ay, yil, hisse_basi_aidat, aktif, created_at, updated_at
         FROM donemler ORDER BY yil DESC, ay DESC"
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(liste))
}

async fn get_donem(State(pool): State<PgPool>, Path(id): Path<i64>) -> AppResult<Json<Donem>> {
    let donem = sqlx::query_as::<_, Donem>(
        "SELECT id, ay, yil, hisse_basi_aidat, aktif, created_at, updated_at
         FROM donemler WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(donem))
}

async fn create_donem(
    State(pool): State<PgPool>,
    Json(input): Json<CreateDonemInput>,
) -> AppResult<Json<Donem>> {
    let donem = sqlx::query_as::<_, Donem>(
        "INSERT INTO donemler (ay, yil, hisse_basi_aidat)
         VALUES ($1, $2, $3)
         RETURNING id, ay, yil, hisse_basi_aidat, aktif, created_at, updated_at"
    )
    .bind(input.ay)
    .bind(input.yil)
    .bind(input.hisse_basi_aidat)
    .fetch_one(&pool)
    .await?;
    Ok(Json(donem))
}

async fn update_donem(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(input): Json<CreateDonemInput>,
) -> AppResult<Json<Donem>> {
    let donem = sqlx::query_as::<_, Donem>(
        "UPDATE donemler SET ay=$1, yil=$2, hisse_basi_aidat=$3, updated_at=NOW()
         WHERE id = $4
         RETURNING id, ay, yil, hisse_basi_aidat, aktif, created_at, updated_at"
    )
    .bind(input.ay)
    .bind(input.yil)
    .bind(input.hisse_basi_aidat)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(donem))
}

async fn delete_donem(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    sqlx::query("DELETE FROM donemler WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(Json(serde_json::json!({ "mesaj": "DÃ¶nem silindi" })))
}

