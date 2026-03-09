//! Memories.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::Db;
use crate::core::types::Scope;
use crate::core::types::error::Error;
use crate::core::types::memory::Memory;

/// One memory row.
type Row = (
    Uuid,
    String,
    String,
    f32,
    DateTime<Utc>,
    Option<DateTime<Utc>>,
);

impl Db {
    /// Unexpired memories of scope in kinds (all when empty), newest and most confident first.
    pub async fn recall(
        &self,
        scope: &Scope,
        kinds: &[String],
        limit: usize,
    ) -> Result<Vec<Memory>, Error> {
        let rows: Vec<Row> = sqlx::query_as(
            "select id, kind, content, confidence, created_at, expires_at from memories
             where tenant_id = $1 and user_id = $2
               and (expires_at is null or expires_at > now())
               and (cardinality($3::text[]) = 0 or kind = any($3))
             order by created_at desc, confidence desc
             limit $4",
        )
        .bind(&scope.tenant)
        .bind(&scope.user)
        .bind(kinds)
        .bind(i64::try_from(limit).unwrap_or(i64::MAX))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(id, kind, content, confidence, created_at, expires_at)| Memory {
                    id,
                    kind,
                    content,
                    confidence,
                    created_at,
                    expires_at,
                },
            )
            .collect())
    }
}
