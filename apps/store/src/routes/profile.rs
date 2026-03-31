//! Profile graph routes.

use axum::Json;
use axum::extract::{Query, State};
use serde_json::{Value, json};

use super::AppState;
use crate::core::types::error::Error;
use crate::core::types::profile::{FactsBody, LabelQuery, MatchingQuery, RelationKey};
use crate::core::types::{LimitQuery, Scope};

/// POST /profile/facts: writes each fact as two nodes and the edge between them.
pub async fn upsert(
    State(state): State<AppState>,
    scope: Scope,
    Json(body): Json<FactsBody>,
) -> Result<Json<Value>, Error> {
    for fact in &body.facts {
        for (name, value) in [
            ("subject.kind", &fact.subject.kind),
            ("subject.label", &fact.subject.label),
            ("relation", &fact.relation),
            ("object.kind", &fact.object.kind),
            ("object.label", &fact.object.label),
        ] {
            state.key(name, value)?;
        }
        if !(0.0..=1.0).contains(&fact.confidence) {
            return Err(Error::Invalid("confidence must be from 0 to 1".into()));
        }
    }
    state.db.upsert(&scope, &body.facts).await?;
    Ok(Json(json!({ "written": body.facts.len() })))
}

/// GET /profile?limit=: nodes and relations, each up to limit.
pub async fn show(
    State(state): State<AppState>,
    scope: Scope,
    Query(query): Query<LimitQuery>,
) -> Result<Json<Value>, Error> {
    let limit = state.limit(query.limit)?;
    let nodes = state.db.nodes(&scope, limit).await?;
    let relations = state.db.relations(&scope, limit).await?;
    Ok(Json(json!({ "nodes": nodes, "relations": relations })))
}

/// GET /profile/matching?subject=&relation=: relations already recorded from subject.
pub async fn matching(
    State(state): State<AppState>,
    scope: Scope,
    Query(query): Query<MatchingQuery>,
) -> Result<Json<Value>, Error> {
    let subject = state.key("subject", &query.subject)?;
    let relation = state.key("relation", &query.relation)?;
    let relations = state
        .db
        .matching(&scope, subject, relation, state.bounds.rows)
        .await?;
    Ok(Json(json!({ "relations": relations })))
}

/// DELETE /profile/relations: removes one relation by its labels.
pub async fn drop_relation(
    State(state): State<AppState>,
    scope: Scope,
    Json(key): Json<RelationKey>,
) -> Result<Json<Value>, Error> {
    let deleted = state.db.drop_relation(&scope, &key).await?;
    Ok(Json(json!({ "deleted": deleted })))
}

/// DELETE /profile/nodes?label=: removes the nodes under label and the relations through them.
pub async fn forget(
    State(state): State<AppState>,
    scope: Scope,
    Query(query): Query<LabelQuery>,
) -> Result<Json<Value>, Error> {
    let label = state.key("label", &query.label)?;
    let deleted = state.db.forget(&scope, label).await?;
    Ok(Json(json!({ "deleted": deleted })))
}

/// DELETE /data: removes every node and memory of scope.
pub async fn forget_all(State(state): State<AppState>, scope: Scope) -> Result<Json<Value>, Error> {
    let deleted = state.db.forget_all(&scope).await?;
    Ok(Json(json!({ "deleted": deleted })))
}
