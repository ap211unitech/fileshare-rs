use std::time::Duration;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use axum::{body::Body, extract::Request, http::Response};
use mongodb::results::InsertOneResult;
use tower_http::classify::ServerErrorsFailureClass;
use tracing::Span;

use crate::error::AppError;

pub struct Tracing;

impl Tracing {
    pub fn on_request(request: &Request<Body>, _: &Span) {
        tracing::info!(
            "HTTP request: {} {}",
            request.method(),
            request.uri().path()
        )
    }

    pub fn on_response(response: &Response<Body>, latency: Duration, _: &Span) {
        tracing::info!("HTTP response: {} {:?}", response.status(), latency)
    }

    pub fn on_failure(error: ServerErrorsFailureClass, latency: Duration, _: &Span) {
        tracing::error!("Request failed: {:?} after {:?}", error, latency)
    }
}

pub fn hash_password(password: &str) -> Result<String, String> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| e.to_string())?
        .to_string();

    Ok(hashed_password)
}

pub fn verify_password(hashed_password: &str, password: &str) -> Result<bool, String> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(&hashed_password).map_err(|e| e.to_string())?;

    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn get_inserted_id(doc: &InsertOneResult) -> Result<String, AppError> {
    Ok(doc
        .inserted_id
        .as_object_id()
        .ok_or_else(|| AppError::Internal("Cannot get inserted id".to_string()))?
        .to_hex())
}
