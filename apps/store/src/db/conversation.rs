//! Conversations and their messages.

use serde_json::Value;
use sqlx::types::Json;
use uuid::Uuid;

use super::{Db, is_owner, owner};
use crate::core::types::Scope;
use crate::core::types::conversation::{NewMessage, SUMMARY_ROLE, StoredMessage, Visibility};
use crate::core::types::error::Error;

/// Answer when the conversation belongs to another user or visibility.
const NOT_OWNED: &str = "conversation is not owned by this user";

impl Db {
    /// Creates the conversation for scope, or touches it when scope already holds it.
    pub async fn ensure(
        &self,
        scope: &Scope,
        conversation: Uuid,
        channel: &str,
        visibility: Visibility,
    ) -> Result<(), Error> {
        let row: Option<(Uuid,)> = sqlx::query_as(
            "insert into conversations (id, tenant_id, user_id, channel_id, visibility)
             values ($1, $2, $3, $4, $5)
             on conflict (id) do update set updated_at = now()
             where conversations.tenant_id = excluded.tenant_id
               and conversations.user_id = excluded.user_id
               and conversations.visibility = excluded.visibility
             returning id",
        )
        .bind(conversation)
        .bind(&scope.tenant)
        .bind(&scope.user)
        .bind(channel)
        .bind(visibility.as_str())
        .fetch_optional(&self.pool)
        .await?;
        row.map(|_| ()).ok_or(Error::Conflict(NOT_OWNED))
    }

    /// Whether the conversation exists, belongs to scope, and has this visibility.
    pub async fn owns(
        &self,
        scope: &Scope,
        conversation: Uuid,
        visibility: Visibility,
    ) -> Result<bool, Error> {
        let (owned,): (bool,) = sqlx::query_as(
            "select exists (
                 select 1 from conversations
                 where id = $1 and tenant_id = $2 and user_id = $3 and visibility = $4
             )",
        )
        .bind(conversation)
        .bind(&scope.tenant)
        .bind(&scope.user)
        .bind(visibility.as_str())
        .fetch_one(&self.pool)
        .await?;
        Ok(owned)
    }

    /// The newest summary, then up to limit messages after what it covers, oldest first.
    pub async fn load(
        &self,
        scope: &Scope,
        conversation: Uuid,
        limit: usize,
    ) -> Result<Vec<StoredMessage>, Error> {
        match owner(&self.pool, conversation).await? {
            None => return Ok(Vec::new()),
            Some(row) if !is_owner(&row, scope) => return Err(Error::Conflict(NOT_OWNED)),
            Some(_) => {}
        }
        let summary: Option<(i64, Json<Value>, Option<i64>)> = sqlx::query_as(
            "select position, content, covers from messages
             where conversation_id = $1 and role = $2
             order by position desc limit 1",
        )
        .bind(conversation)
        .bind(SUMMARY_ROLE)
        .fetch_optional(&self.pool)
        .await?;
        // A summary without covers stands in for every message before it.
        let after = summary
            .as_ref()
            .map_or(0, |(position, _, covers)| covers.unwrap_or(position - 1));
        let rows: Vec<(i64, String, Json<Value>)> = sqlx::query_as(
            "select position, role, content from (
                 select position, role, content from messages
                 where conversation_id = $1 and role <> $2 and position > $3
                 order by position desc limit $4
             ) newest
             order by position",
        )
        .bind(conversation)
        .bind(SUMMARY_ROLE)
        .bind(after)
        .bind(i64::try_from(limit).unwrap_or(i64::MAX))
        .fetch_all(&self.pool)
        .await?;
        let mut out = Vec::with_capacity(rows.len() + 1);
        if let Some((position, Json(content), _)) = summary {
            out.push(StoredMessage {
                position,
                role: SUMMARY_ROLE.to_owned(),
                content,
            });
        }
        out.extend(
            rows.into_iter()
                .map(|(position, role, Json(content))| StoredMessage {
                    position,
                    role,
                    content,
                }),
        );
        Ok(out)
    }

    /// Appends messages in order and returns their positions.
    pub async fn append(
        &self,
        scope: &Scope,
        conversation: Uuid,
        messages: Vec<NewMessage>,
    ) -> Result<Vec<i64>, Error> {
        let count = i64::try_from(messages.len())
            .map_err(|_| Error::Invalid("too many messages".into()))?;
        let mut tx = self.pool.begin().await?;
        let first = reserve(&mut tx, scope, conversation, count).await?;
        let mut positions = Vec::with_capacity(messages.len());
        for (position, message) in (first..).zip(messages) {
            sqlx::query(
                "insert into messages (conversation_id, position, role, content)
                 values ($1, $2, $3, $4)",
            )
            .bind(conversation)
            .bind(position)
            .bind(message.role.as_str())
            .bind(Json(message.content))
            .execute(&mut *tx)
            .await?;
            positions.push(position);
        }
        tx.commit().await?;
        Ok(positions)
    }

    /// Stores a summary standing in for every message up to covers; returns its position.
    pub async fn summarize(
        &self,
        scope: &Scope,
        conversation: Uuid,
        content: Value,
        covers: i64,
    ) -> Result<i64, Error> {
        let mut tx = self.pool.begin().await?;
        let position = reserve(&mut tx, scope, conversation, 1).await?;
        if !(0..position).contains(&covers) {
            return Err(Error::Invalid(format!(
                "covers must be from 0 to {}",
                position - 1
            )));
        }
        sqlx::query(
            "insert into messages (conversation_id, position, role, content, covers)
             values ($1, $2, $3, $4, $5)",
        )
        .bind(conversation)
        .bind(position)
        .bind(SUMMARY_ROLE)
        .bind(Json(content))
        .bind(covers)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(position)
    }

    /// The most recently updated open conversation of scope in channel at visibility.
    pub async fn latest(
        &self,
        scope: &Scope,
        channel: &str,
        visibility: Visibility,
    ) -> Result<Option<Uuid>, Error> {
        let row: Option<(Uuid,)> = sqlx::query_as(
            "select id from conversations
             where tenant_id = $1 and user_id = $2 and channel_id = $3 and visibility = $4
               and ended_at is null
             order by updated_at desc limit 1",
        )
        .bind(&scope.tenant)
        .bind(&scope.user)
        .bind(channel)
        .bind(visibility.as_str())
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|(id,)| id))
    }

    /// Ends every open conversation of scope in channel, any visibility; returns the count.
    pub async fn end(&self, scope: &Scope, channel: &str) -> Result<u64, Error> {
        let done = sqlx::query(
            "update conversations set ended_at = now(), updated_at = now()
             where tenant_id = $1 and user_id = $2 and channel_id = $3 and ended_at is null",
        )
        .bind(&scope.tenant)
        .bind(&scope.user)
        .bind(channel)
        .execute(&self.pool)
        .await?;
        Ok(done.rows_affected())
    }
}

/// Reserves count positions in a conversation of scope and returns the first.
async fn reserve(
    tx: &mut sqlx::PgConnection,
    scope: &Scope,
    conversation: Uuid,
    count: i64,
) -> Result<i64, Error> {
    let row: Option<(i64,)> = sqlx::query_as(
        "update conversations set last_position = last_position + $4, updated_at = now()
         where id = $1 and tenant_id = $2 and user_id = $3
         returning last_position",
    )
    .bind(conversation)
    .bind(&scope.tenant)
    .bind(&scope.user)
    .bind(count)
    .fetch_optional(&mut *tx)
    .await?;
    match row {
        Some((last,)) => Ok(last - count + 1),
        None => match owner(&mut *tx, conversation).await? {
            None => Err(Error::NotFound("no such conversation")),
            Some(_) => Err(Error::Conflict(NOT_OWNED)),
        },
    }
}
