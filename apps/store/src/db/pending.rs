//! Actions waiting on the user's answer.

use serde_json::Value;
use sqlx::types::Json;
use uuid::Uuid;

use super::Db;
use crate::core::types::Scope;
use crate::core::types::error::Error;

impl Db {
    /// Holds action under token for ttl seconds, after dropping every lapsed hold.
    pub async fn hold(
        &self,
        scope: &Scope,
        token: Uuid,
        action: Value,
        payload_hash: &str,
        ttl_secs: u64,
    ) -> Result<(), Error> {
        sqlx::query("delete from pending_actions where expires_at <= now()")
            .execute(&self.pool)
            .await?;
        // A ttl is bounded by store.max_ttl_secs, far inside the range of an f64.
        #[allow(clippy::cast_precision_loss)]
        let ttl = ttl_secs as f64;
        let done = sqlx::query(
            "insert into pending_actions (token, tenant_id, user_id, action, payload_hash, expires_at)
             values ($1, $2, $3, $4, $5, now() + make_interval(secs => $6))
             on conflict (token) do nothing",
        )
        .bind(token)
        .bind(&scope.tenant)
        .bind(&scope.user)
        .bind(Json(action))
        .bind(payload_hash)
        .bind(ttl)
        .execute(&self.pool)
        .await?;
        if done.rows_affected() == 0 {
            return Err(Error::Conflict("token is already held"));
        }
        Ok(())
    }

    /// Takes the unexpired hold of scope under token, once. None when there is none.
    pub async fn claim(&self, scope: &Scope, token: Uuid) -> Result<Option<Value>, Error> {
        let row: Option<(Json<Value>,)> = sqlx::query_as(
            "delete from pending_actions
             where token = $1 and tenant_id = $2 and user_id = $3 and expires_at > now()
             returning action",
        )
        .bind(token)
        .bind(&scope.tenant)
        .bind(&scope.user)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|(Json(action),)| action))
    }
}
