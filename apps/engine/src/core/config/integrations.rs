//! Settings for the public integrations the tools reach.

use secrecy::SecretString;
use serde::Deserialize;

/// Market data over the Finnhub REST API: quotes, symbol search, profiles, metrics, and news.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Market {
    /// Whether the market tools are registered at boot.
    pub enabled: bool,
    /// Base URL of the market data API, no trailing path.
    pub base_url: String,
    /// API key sent with every request. Lives in .env.
    pub api_key: Option<SecretString>,
    /// Budget for one request.
    pub timeout_secs: u64,
    /// Most rows one list tool returns to the model.
    pub max_items: usize,
    /// Days of company news a news call reads back from today.
    pub news_days: u32,
    /// Exchange code market_status reports when a call names none.
    pub default_exchange: String,
    /// Longest news summary kept, in characters.
    pub summary_chars: usize,
}

impl Default for Market {
    fn default() -> Self {
        Self {
            enabled: true,
            base_url: "https://finnhub.io/api/v1".into(),
            api_key: None,
            timeout_secs: 15,
            max_items: 8,
            news_days: 7,
            default_exchange: "US".into(),
            summary_chars: 280,
        }
    }
}

/// Wikipedia lookups over the MediaWiki API. No key; a public read.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Wikipedia {
    /// Whether the Wikipedia tool is registered at boot.
    pub enabled: bool,
    /// The MediaWiki API endpoint.
    pub base_url: String,
    /// Budget for one request.
    pub timeout_secs: u64,
    /// Most characters of the summary returned to the model.
    pub max_chars: usize,
}

impl Default for Wikipedia {
    fn default() -> Self {
        Self {
            enabled: true,
            base_url: "https://en.wikipedia.org/w/api.php".into(),
            timeout_secs: 20,
            max_chars: 1500,
        }
    }
}
