//! Memories on the store routes.

use async_trait::async_trait;
use reqwest::Method;
use serde_json::Value;

use super::client::{Call, ClientError, StoreClient, field, optional_timestamp, timestamp};
use crate::core::traits::memory::MemoryStore;
use crate::core::types::agent::context::RequestContext;
use crate::core::types::memory::{Memory, MemoryKind, MemoryQuery};
use crate::core::types::store::StoreError;

/// Maps a client error to a StoreError.
#[allow(clippy::needless_pass_by_value)]
fn store(e: ClientError) -> StoreError {
    StoreError::Database(e.to_string())
}

/// Memories held by the store.
pub struct HttpMemory {
    client: StoreClient,
}

impl HttpMemory {
    /// Wraps a client.
    pub fn new(client: StoreClient) -> Self {
        Self { client }
    }
}

/// One memory from its store row.
fn memory(row: &Value) -> Result<Memory, ClientError> {
    let kind: String = field(row, "kind")?;
    let kind = MemoryKind::parse(&kind)
        .ok_or_else(|| ClientError::Body(format!("memory kind {kind} is unknown")))?;
    let confidence: f64 = field(row, "confidence")?;
    // Confidence is a fraction from 0 to 1.
    #[allow(clippy::cast_possible_truncation)]
    Ok(Memory {
        id: field(row, "id")?,
        kind,
        content: field(row, "content")?,
        confidence: confidence as f32,
        created_at: timestamp(row, "created_at")?,
        expires_at: optional_timestamp(row, "expires_at")?,
    })
}

#[async_trait]
impl MemoryStore for HttpMemory {
    async fn recall(
        &self,
        ctx: &RequestContext,
        q: &MemoryQuery,
    ) -> Result<Vec<Memory>, StoreError> {
        let mut query = vec![("limit", q.limit.max(1).to_string())];
        if !q.kinds.is_empty() {
            let kinds: Vec<&str> = q.kinds.iter().map(|k| k.as_str()).collect();
            query.push(("kinds", kinds.join(",")));
        }
        let path = StoreClient::scoped(ctx, "/memories");
        let value = self
            .client
            .call(&Call::new(Method::GET, &path).query(&query))
            .await
            .map_err(store)?;
        let rows: Vec<Value> = field(&value, "memories").map_err(store)?;
        rows.iter().map(|r| memory(r).map_err(store)).collect()
    }
}
