use reqwest::{header, StatusCode};
use serde_json::json;

use crate::{config::AppConfig, error::AppError, models::token::TokenType};

pub struct EmailInfo<'a> {
    pub recipient_email: &'a str,
    pub verification_link: &'a str,
    pub email_type: TokenType,
}

impl<'a> EmailInfo<'a> {
    pub async fn send_email(self) -> Result<bool, AppError> {
        let app_config = AppConfig::load_config();

        let (subject, body) = match self.email_type {
            TokenType::EmailVerification => self.email_verification_template(),
        };

        let body = json!(
            {
                "personalizations": [{
                    "to": [{
                        "email": self.recipient_email
                    }]
                }],
                "from": {
                    "email": &app_config.sendgrid_sender_email,
                    "name": &app_config.sendgrid_sender_name
                },
                "subject": subject,
                "content": [
                    {
                        "type": "text/html",
                        "value": body
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

    fn email_verification_template(&self) -> (String, String) {
        let subject = String::from("Please verify your email");

        let content = format!(
            "<div>Please verify your account: <a href=\"http://{}\" target=\"_blank\">Verify account</a></div>",
            self.verification_link
        );

        return (subject, content);
    }
}
