//! The HTTP surface: GET /health, and every store route under /api/v1/tenants/{tenant}/users/{user}.

mod conversation;
mod memory;
mod pending;
mod profile;

use std::sync::Arc;

use axum::extract::{DefaultBodyLimit, FromRequestParts, Path, Request, State};
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;
use tower_http::trace::TraceLayer;

use crate::core::config::Store;
use crate::core::types::Scope;
use crate::core::types::error::Error;
use crate::db::Db;

/// What each handler needs: the database, the token, and the bounds on one call.
#[derive(Clone)]
pub struct AppState {
    db: Db,
    token: Arc<SecretString>,
    bounds: Bounds,
}

/// Bounds on one call, from the store section.
#[derive(Debug, Clone, Copy)]
struct Bounds {
    rows: usize,
    ttl_secs: u64,
    key_chars: usize,
}

impl AppState {
    /// State over db, guarded by token, bounded by the store section.
    pub fn new(db: Db, token: SecretString, cfg: &Store) -> Self {
        Self {
            db,
            token: Arc::new(token),
            bounds: Bounds {
                rows: cfg.max_limit,
                ttl_secs: cfg.max_ttl_secs,
                key_chars: cfg.max_key_chars,
            },
        }
    }

    /// Rejects a limit of 0 or above store.max_limit.
    fn limit(&self, limit: usize) -> Result<usize, Error> {
        if (1..=self.bounds.rows).contains(&limit) {
            Ok(limit)
        } else {
            Err(Error::Invalid(format!(
                "limit must be from 1 to {}",
                self.bounds.rows
            )))
        }
    }

    /// Rejects an empty key, one past store.max_key_chars, or one holding a control character.
    fn key<'a>(&self, name: &str, value: &'a str) -> Result<&'a str, Error> {
        let chars = value.chars().count();
        if chars == 0 || chars > self.bounds.key_chars || value.chars().any(char::is_control) {
            return Err(Error::Invalid(format!(
                "{name} must be 1 to {} printable characters",
                self.bounds.key_chars
            )));
        }
        Ok(value)
    }
}

/// The router over state, every body capped at max_body_bytes.
pub fn router(state: AppState, max_body_bytes: usize) -> Router {
    let user = Router::new()
        .route(
            "/conversations/{conversation}",
            put(conversation::ensure).get(conversation::owns),
        )
        .route(
            "/conversations/{conversation}/messages",
            get(conversation::load).post(conversation::append),
        )
        .route(
            "/conversations/{conversation}/summary",
            post(conversation::summarize),
        )
        .route("/channels/{channel}/latest", get(conversation::latest))
        .route("/channels/{channel}/end", post(conversation::end))
        .route("/memories", get(memory::recall))
        .route("/pending/{token}", put(pending::hold))
        .route("/pending/{token}/claim", post(pending::claim))
        .route("/profile", get(profile::show))
        .route("/profile/facts", post(profile::upsert))
        .route("/profile/matching", get(profile::matching))
        .route("/profile/relations", delete(profile::drop_relation))
        .route("/profile/nodes", delete(profile::forget))
        .route("/data", delete(profile::forget_all));
    let api = Router::new()
        .nest("/api/v1/tenants/{tenant}/users/{user}", user)
        .route_layer(middleware::from_fn_with_state(state.clone(), authorize));
    Router::new()
        .route("/health", get(health))
        .merge(api)
        .layer(DefaultBodyLimit::max(max_body_bytes))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// 200 when the database answers, 503 otherwise. Needs no token.
async fn health(State(state): State<AppState>) -> Response {
    if state.db.ready().await {
        (StatusCode::OK, Json(json!({"status": "ok"}))).into_response()
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"status": "unavailable"})),
        )
            .into_response()
    }
}

/// Lets a request through only with the bearer token.
async fn authorize(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let presented = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or_default();
    if equal(presented.as_bytes(), state.token.expose_secret().as_bytes()) {
        next.run(request).await
    } else {
        Error::Unauthorized.into_response()
    }
}

/// Compares two byte strings in time that depends only on their lengths.
pub fn equal(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

impl FromRequestParts<AppState> for Scope {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Path(scope) = Path::<Scope>::from_request_parts(parts, state)
            .await
            .map_err(|e| Error::Invalid(e.body_text()))?;
        state.key("tenant", &scope.tenant)?;
        state.key("user", &scope.user)?;
        Ok(scope)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status =
            StatusCode::from_u16(self.status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let message = match &self {
            Self::Database(error) => {
                tracing::error!(error = %error, "store database call failed");
                "store database failed".to_owned()
            }
            other => other.to_string(),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
