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

    #[error("JWT Error: {0}")]
    Jwt(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("BadRequest: {0}")]
    BadRequest(String),

    #[error("Internal Server Error: {0}")]
    Internal(String),
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    kind: String,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status_code, error) = match self {
            AppError::Database(e) => {
                let error = ErrorResponse {
                    kind: "Database".to_string(),
                    message: e.to_string(),
                };

                (StatusCode::INTERNAL_SERVER_ERROR, error)
            }
            AppError::Validation(e) => {
                let error = ErrorResponse {
                    kind: "Validation".to_string(),
                    message: e.to_string(),
                };

                (StatusCode::BAD_REQUEST, error)
            }
            AppError::BadRequest(e) => {
                let error = ErrorResponse {
                    kind: "BadRequest".to_string(),
                    message: e.to_string(),
                };

                (StatusCode::BAD_REQUEST, error)
            }
            AppError::Jwt(e) => {
                let error = ErrorResponse {
                    kind: "JWT".to_string(),
                    message: e,
                };

                (StatusCode::INTERNAL_SERVER_ERROR, error)
            }
            AppError::Unauthorized(e) => {
                let error = ErrorResponse {
                    kind: "Unauthorized".to_string(),
                    message: e,
                };

                (StatusCode::UNAUTHORIZED, error)
            }
            AppError::Internal(e) => {
                let error = ErrorResponse {
                    kind: "Internal Server Error".to_string(),
                    message: e,
                };

                (StatusCode::INTERNAL_SERVER_ERROR, error)
            }
        };

        tracing::error!("{:?}", error);

        (status_code, Json(error)).into_response()
    }
}
