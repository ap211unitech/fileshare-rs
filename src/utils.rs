use std::time::Duration;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use axum::{body::Body, extract::Request, http::Response};
use mongodb::results::InsertOneResult;
use reqwest::{header, StatusCode};
use serde_json::json;
use tower_http::classify::ServerErrorsFailureClass;
use tracing::Span;

use crate::{config::AppConfig, error::AppError};

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
pub struct SendgridUser<'a> {
    pub name: &'a str,
    pub email: &'a str,
}

pub async fn send_email<'a>(recipient: SendgridUser<'a>) -> Result<bool, AppError> {
    let app_config = AppConfig::load_config();

    let sender = SendgridUser {
        name: &app_config.sendgrid_sender_name,
        email: &app_config.sendgrid_sender_email,
    };

    let body = json!(
        {
            "personalizations": [{
                "to": [{
                    "email": recipient.email,
                    "name": recipient.name
                }]
            }],
            "from": {
                "email": sender.email,
                "name": sender.name
            },
            "subject": "Let's Send an Email With Rust and SendGrid",
            "content": [
                {
                    "type": "text/html",
                    "value": "Here is your <strong>AMAZING</strong> email!"
                },
            ]
        }
    );

    let client = reqwest::Client::new(); // Use async version

    let client = client
        .post("https://api.sendgrid.com/v3/mail/send")
        .json(&body)
        .bearer_auth(app_config.sendgrid_api_key)
        .header(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

    let response = client
        .send()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    match response.status() {
        StatusCode::OK | StatusCode::CREATED | StatusCode::ACCEPTED => {
            tracing::info!("Email sent 👍");
            return Ok(true);
        }
        _ => {
            return Err(AppError::Internal(format!(
                "Unable to send your email. Status code was: {}. Body content was: {:?}",
                response.status(),
                response
                    .text()
                    .await
                    .map_err(|_| "Failed to read response body".to_string())
            )));
        }
    }
}
