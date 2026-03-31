//! Memory routes.

use axum::Json;
use axum::extract::{Query, State};
use serde_json::{Value, json};

use super::AppState;
use crate::core::types::Scope;
use crate::core::types::error::Error;
use crate::core::types::memory::{KINDS, RecallQuery};

/// GET /memories?limit=&kinds=: unexpired memories, newest and most confident first.
pub async fn recall(
    State(state): State<AppState>,
    scope: Scope,
    Query(query): Query<RecallQuery>,
) -> Result<Json<Value>, Error> {
    let limit = state.limit(query.limit)?;
    let kinds: Vec<String> = query
        .kinds
        .as_deref()
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .map(str::to_owned)
        .collect();
    if let Some(unknown) = kinds.iter().find(|k| !KINDS.contains(&k.as_str())) {
        return Err(Error::Invalid(format!("memory kind {unknown} is unknown")));
    }
    let memories = state.db.recall(&scope, &kinds, limit).await?;
    Ok(Json(json!({ "memories": memories })))
}
