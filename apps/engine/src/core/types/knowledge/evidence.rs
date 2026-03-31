//! A citation names a source an answer rests on.

use serde::{Deserialize, Serialize};

/// One source an answer rests on. A client decides how to show it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Citation {
    /// Key of the source behind it, such as market for market data.
    pub key: String,
    /// Human-readable source name.
    pub title: String,
    /// Canonical page URL, when the source has one.
    pub url: Option<String>,
}
