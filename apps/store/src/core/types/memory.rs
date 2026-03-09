//! Memory wire shapes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Memory kinds the store holds.
pub const KINDS: [&str; 4] = ["episodic", "semantic", "profile", "task"];

/// GET /memories: how many, and of which kinds.
#[derive(Debug, Deserialize)]
pub struct RecallQuery {
    /// Most rows returned.
    pub limit: usize,
    /// Comma-separated kinds; absent or empty means all.
    #[serde(default)]
    pub kinds: Option<String>,
}

/// One stored memory.
#[derive(Debug, Serialize)]
pub struct Memory {
    /// Row id.
    pub id: Uuid,
    /// One of KINDS.
    pub kind: String,
    /// The remembered text.
    pub content: String,
    /// Confidence at write time, 0 to 1.
    pub confidence: f32,
    /// When it was written.
    pub created_at: DateTime<Utc>,
    /// When it stops being recalled.
    pub expires_at: Option<DateTime<Utc>>,
}
