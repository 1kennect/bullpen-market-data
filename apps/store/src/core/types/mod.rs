//! Wire shapes, rows, and the error every route answers with.

pub mod conversation;
pub mod error;
pub mod memory;
pub mod pending;
pub mod profile;

use serde::Deserialize;

/// The tenant and user every route under /api/v1 is scoped to, as the path carries them.
#[derive(Debug, Clone, Deserialize)]
pub struct Scope {
    /// The tenant, the guild the engine serves.
    pub tenant: String,
    /// The user within the tenant.
    pub user: String,
}

/// A read bounded by limit.
#[derive(Debug, Deserialize)]
pub struct LimitQuery {
    /// Most rows returned.
    pub limit: usize,
}
