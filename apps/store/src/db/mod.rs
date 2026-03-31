//! Postgres access. The only place a connection is opened; every query is scoped by tenant and user.

mod conversation;
mod memory;
mod pending;
mod profile;

use std::time::Duration;

use secrecy::{ExposeSecret, SecretString};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::core::types::Scope;
use crate::core::types::error::Error;

/// The migrations under apps/store/migrations, embedded at build time.
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// A pool over the store database. Cheap to clone.
#[derive(Clone)]
pub struct Db {
    pool: PgPool,
}

impl Db {
    /// Connects to url with at most max_connections, each taken within acquire_timeout.
    pub async fn connect(
        url: &SecretString,
        max_connections: u32,
        acquire_timeout: Duration,
    ) -> Result<Self, Error> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(acquire_timeout)
            .connect(url.expose_secret())
            .await?;
        Ok(Self { pool })
    }

    /// A pool that opens its first connection on first use.
    #[cfg(test)]
    pub fn lazy(url: &SecretString, acquire_timeout: Duration) -> Result<Self, Error> {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(acquire_timeout)
            .connect_lazy(url.expose_secret())?;
        Ok(Self { pool })
    }

    /// The pool, for tests that write rows no route writes.
    #[cfg(test)]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Applies every migration not yet applied.
    pub async fn migrate(&self) -> Result<(), Error> {
        MIGRATOR
            .run(&self.pool)
            .await
            .map_err(|e| Error::Database(e.into()))
    }

    /// Whether the database answers a trivial query.
    pub async fn ready(&self) -> bool {
        match sqlx::query("select 1").execute(&self.pool).await {
            Ok(_) => true,
            Err(error) => {
                tracing::warn!(error = %error, "readiness: database did not answer");
                false
            }
        }
    }
}

/// The owner of a conversation, when it exists.
async fn owner(
    executor: impl sqlx::PgExecutor<'_>,
    conversation: uuid::Uuid,
) -> Result<Option<(String, String)>, Error> {
    Ok(
        sqlx::query_as("select tenant_id, user_id from conversations where id = $1")
            .bind(conversation)
            .fetch_optional(executor)
            .await?,
    )
}

/// Whether an owner row belongs to scope.
fn is_owner(row: &(String, String), scope: &Scope) -> bool {
    row.0 == scope.tenant && row.1 == scope.user
}
