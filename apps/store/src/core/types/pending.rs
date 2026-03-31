//! Pending action wire shapes.

use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

/// The token in the path.
#[derive(Debug, Deserialize)]
pub struct TokenPath {
    /// Confirmation token.
    pub token: Uuid,
}

/// PUT /pending/{token}: the action held until the user answers.
#[derive(Debug, Deserialize)]
pub struct HoldBody {
    /// The action as the engine serializes it.
    pub action: Value,
    /// Hash of the payload the user is shown.
    pub payload_hash: String,
    /// Seconds until the hold lapses.
    pub ttl_seconds: u64,
}

/// POST /pending/{token}/claim.
#[derive(Debug, Deserialize)]
pub struct ClaimBody {
    /// Whether the user approved. A claim consumes the hold either way.
    pub approved: bool,
}
