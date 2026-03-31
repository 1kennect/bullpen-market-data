//! Conversation routes.

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde_json::{Value, json};

use super::AppState;
use crate::core::types::conversation::{
    AppendBody, ChannelPath, ConversationPath, EnsureBody, SummaryBody, VisibilityQuery,
};
use crate::core::types::error::Error;
use crate::core::types::{LimitQuery, Scope};

/// PUT /conversations/{conversation}: creates it, or 409 when another user or visibility holds it.
pub async fn ensure(
    State(state): State<AppState>,
    scope: Scope,
    Path(path): Path<ConversationPath>,
    Json(body): Json<EnsureBody>,
) -> Result<Json<Value>, Error> {
    let channel = state.key("channel_id", &body.channel_id)?;
    state
        .db
        .ensure(&scope, path.conversation, channel, body.visibility)
        .await?;
    Ok(Json(json!({ "id": path.conversation })))
}

/// GET /conversations/{conversation}?visibility=: whether scope holds it at that visibility.
pub async fn owns(
    State(state): State<AppState>,
    scope: Scope,
    Path(path): Path<ConversationPath>,
    Query(query): Query<VisibilityQuery>,
) -> Result<Json<Value>, Error> {
    let owned = state
        .db
        .owns(&scope, path.conversation, query.visibility)
        .await?;
    Ok(Json(json!({ "owned": owned })))
}

/// GET /conversations/{conversation}/messages?limit=: the newest summary and the messages after it.
pub async fn load(
    State(state): State<AppState>,
    scope: Scope,
    Path(path): Path<ConversationPath>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<Value>, Error> {
    let limit = state.limit(query.limit)?;
    let messages = state.db.load(&scope, path.conversation, limit).await?;
    Ok(Json(json!({ "messages": messages })))
}

/// POST /conversations/{conversation}/messages: appends in order.
pub async fn append(
    State(state): State<AppState>,
    scope: Scope,
    Path(path): Path<ConversationPath>,
    Json(body): Json<AppendBody>,
) -> Result<(StatusCode, Json<Value>), Error> {
    if body.messages.is_empty() {
        return Err(Error::Invalid("messages is empty".into()));
    }
    let positions = state
        .db
        .append(&scope, path.conversation, body.messages)
        .await?;
    Ok((StatusCode::CREATED, Json(json!({ "positions": positions }))))
}

/// POST /conversations/{conversation}/summary: stores a summary over everything up to covers.
pub async fn summarize(
    State(state): State<AppState>,
    scope: Scope,
    Path(path): Path<ConversationPath>,
    Json(body): Json<SummaryBody>,
) -> Result<(StatusCode, Json<Value>), Error> {
    let position = state
        .db
        .summarize(&scope, path.conversation, body.content, body.covers)
        .await?;
    Ok((StatusCode::CREATED, Json(json!({ "position": position }))))
}

/// GET /channels/{channel}/latest?visibility=: the open conversation updated last, or null.
pub async fn latest(
    State(state): State<AppState>,
    scope: Scope,
    Path(path): Path<ChannelPath>,
    Query(query): Query<VisibilityQuery>,
) -> Result<Json<Value>, Error> {
    let channel = state.key("channel", &path.channel)?;
    let id = state.db.latest(&scope, channel, query.visibility).await?;
    Ok(Json(json!({ "id": id })))
}

/// POST /channels/{channel}/end: ends every open conversation of scope there.
pub async fn end(
    State(state): State<AppState>,
    scope: Scope,
    Path(path): Path<ChannelPath>,
) -> Result<Json<Value>, Error> {
    let channel = state.key("channel", &path.channel)?;
    let ended = state.db.end(&scope, channel).await?;
    Ok(Json(json!({ "ended": ended })))
}
