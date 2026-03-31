//! Confirmation routes.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde_json::{Value, json};

use super::AppState;
use crate::core::types::Scope;
use crate::core::types::error::Error;
use crate::core::types::pending::{ClaimBody, HoldBody, TokenPath};

/// PUT /pending/{token}: holds an action until the user answers or the ttl lapses.
pub async fn hold(
    State(state): State<AppState>,
    scope: Scope,
    Path(path): Path<TokenPath>,
    Json(body): Json<HoldBody>,
) -> Result<(StatusCode, Json<Value>), Error> {
    if !(1..=state.bounds.ttl_secs).contains(&body.ttl_seconds) {
        return Err(Error::Invalid(format!(
            "ttl_seconds must be from 1 to {}",
            state.bounds.ttl_secs
        )));
    }
    let hash = state.key("payload_hash", &body.payload_hash)?;
    state
        .db
        .hold(&scope, path.token, body.action, hash, body.ttl_seconds)
        .await?;
    Ok((StatusCode::CREATED, Json(json!({ "token": path.token }))))
}

/// POST /pending/{token}/claim: the held action, once; 404 when unowned, answered, or lapsed.
pub async fn claim(
    State(state): State<AppState>,
    scope: Scope,
    Path(path): Path<TokenPath>,
    Json(body): Json<ClaimBody>,
) -> Result<Json<Value>, Error> {
    let action = state
        .db
        .claim(&scope, path.token)
        .await?
        .ok_or(Error::NotFound("no pending action under this token"))?;
    tracing::info!(token = %path.token, approved = body.approved, "pending action claimed");
    Ok(Json(json!({ "action": action })))
}
