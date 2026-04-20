use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Unauthorized,
    Internal(anyhow::Error),
    Database(sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Yetkisiz erişim".to_string()),
            AppError::Internal(e) => {
                tracing::error!("Dahili hata: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Sunucu hatası".to_string())
            }
            AppError::Database(e) => {
                tracing::error!("Veritabanı hatası: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Veritabanı hatası".to_string())
            }
        };

        (status, Json(json!({ "hata": message }))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => AppError::NotFound("Kayıt bulunamadı".to_string()),
            other => AppError::Database(other),
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e)
    }
}

pub type AppResult<T> = Result<T, AppError>;
