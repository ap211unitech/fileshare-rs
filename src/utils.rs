use std::time::Duration;

use axum::{body::Body, extract::Request, http::Response};
use tower_http::classify::ServerErrorsFailureClass;
use tracing::Span;

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
