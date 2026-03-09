//! Store tests. Database tests run when BULLPEN_STORE_TEST_DATABASE_URL names a Postgres to use.

mod config;
mod db;
mod routes;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use secrecy::SecretString;
use serde_json::Value;
use tower::ServiceExt;

use crate::core::config::Store;
use crate::db::Db;
use crate::routes::{AppState, router};

/// The token every test router accepts.
const TOKEN: &str = "test-token-0123456789";

/// A router over db with the default store section.
fn app(db: Db) -> Router {
    let cfg = Store::default();
    router(
        AppState::new(db, SecretString::from(TOKEN), &cfg),
        cfg.max_body_bytes,
    )
}

/// Sends one request with the token and returns the status and the JSON body, Null when empty.
async fn send(app: &Router, method: &str, path: &str, body: Option<Value>) -> (StatusCode, Value) {
    send_as(app, method, path, body, Some(TOKEN)).await
}

/// Sends one request with token, or none, and returns the status and the JSON body.
async fn send_as(
    app: &Router,
    method: &str,
    path: &str,
    body: Option<Value>,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let mut request = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let request = match body {
        Some(body) => request
            .header("content-type", "application/json")
            .body(Body::from(body.to_string())),
        None => request.body(Body::empty()),
    };
    let Ok(request) = request else {
        unreachable!("the test request builds")
    };
    let Ok(response) = app.clone().oneshot(request).await;
    let status = response.status();
    let Ok(bytes) = axum::body::to_bytes(response.into_body(), usize::MAX).await else {
        unreachable!("the body reads")
    };
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}
