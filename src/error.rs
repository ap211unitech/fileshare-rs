use axum::{http::StatusCode, response::IntoResponse, Json};
use mongodb::error::Error as MongoDbError;
use serde::Serialize;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database Error: {0}")]
    Database(#[from] MongoDbError),

    #[error("Request validation Error: {0}")]
    Validation(#[from] ValidationErrors),

    #[error("Internal Server Error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    kind: String,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::Database(e) => {
                let error = ErrorResponse {
                    kind: "Database".to_string(),
                    message: e.to_string(),
                };

                return (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response();
            }
            AppError::Validation(e) => {
                let error = ErrorResponse {
                    kind: "Validation".to_string(),
                    message: e.to_string(),
                };

                return (StatusCode::BAD_REQUEST, Json(error)).into_response();
            }
            AppError::Internal(e) => {
                let error = ErrorResponse {
                    kind: "Internal".to_string(),
                    message: e,
                };

                return (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response();
            }
        }
    }
}
