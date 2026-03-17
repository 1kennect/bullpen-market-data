//! Market data read models and the errors a market data call returns.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Name of the tool that quotes one symbol.
pub const MARKET_QUOTE: &str = "market_quote";
/// Name of the tool that finds a symbol by name.
pub const MARKET_SEARCH: &str = "market_search";
/// Name of the tool that describes a company.
pub const MARKET_PROFILE: &str = "market_profile";
/// Name of the tool that reads recent news.
pub const MARKET_NEWS: &str = "market_news";
/// Name of the tool that reports whether an exchange is open.
pub const MARKET_STATUS: &str = "market_status";

/// The latest trade price of one symbol and how it moved today.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quote {
    /// Symbol as asked, upper case.
    pub symbol: String,
    /// Latest price.
    pub price: f64,
    /// Change from the previous close.
    pub change: Option<f64>,
    /// Change from the previous close, in percent.
    pub change_percent: Option<f64>,
    /// High of the current session.
    pub high: Option<f64>,
    /// Low of the current session.
    pub low: Option<f64>,
    /// Open of the current session.
    pub open: Option<f64>,
    /// Close of the previous session.
    pub previous_close: Option<f64>,
    /// When the price was last updated.
    pub as_of: Option<DateTime<Utc>>,
}

/// One symbol a search matched.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolMatch {
    /// Symbol to quote it under.
    pub symbol: String,
    /// Name of the security.
    pub description: String,
    /// Security type, such as Common Stock or ETP.
    pub kind: String,
}

/// What a company is and where it trades.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompanyProfile {
    /// Symbol, upper case.
    pub symbol: String,
    /// Company name.
    pub name: String,
    /// Listing exchange.
    pub exchange: Option<String>,
    /// Industry classification.
    pub industry: Option<String>,
    /// Country of the headquarters.
    pub country: Option<String>,
    /// Currency the shares trade in.
    pub currency: Option<String>,
    /// Market capitalization in millions of the trading currency.
    pub market_cap_millions: Option<f64>,
    /// IPO date, YYYY-MM-DD.
    pub ipo: Option<String>,
    /// Company website.
    pub website: Option<String>,
}

/// Valuation and range figures for one symbol. Every figure is optional.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Metrics {
    /// 52-week high price.
    pub week52_high: Option<f64>,
    /// 52-week low price.
    pub week52_low: Option<f64>,
    /// Price to earnings, trailing twelve months.
    pub pe_ttm: Option<f64>,
    /// Earnings per share, trailing twelve months.
    pub eps_ttm: Option<f64>,
    /// Indicated annual dividend yield, in percent.
    pub dividend_yield: Option<f64>,
    /// Beta against the market.
    pub beta: Option<f64>,
}

/// One news story.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewsItem {
    /// Headline.
    pub headline: String,
    /// Publisher.
    pub source: String,
    /// Short summary, when the publisher gave one.
    pub summary: Option<String>,
    /// Link to the story.
    pub url: Option<String>,
    /// When it was published.
    pub published: Option<DateTime<Utc>>,
}

/// Whether an exchange is trading now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketStatus {
    /// Exchange code, such as US.
    pub exchange: String,
    /// Whether it is open now.
    pub is_open: bool,
    /// Session now: pre-market, regular, post-market, or none.
    pub session: Option<String>,
    /// Holiday it is closed for today, if any.
    pub holiday: Option<String>,
    /// Time zone of the exchange.
    pub timezone: Option<String>,
}

/// A market data request that could not be answered.
#[derive(Debug, thiserror::Error)]
pub enum MarketError {
    /// The provider could not be reached.
    #[error("market data is unreachable: {0}")]
    Unreachable(String),
    /// The provider refused the key.
    #[error("market data rejected the API key")]
    Unauthorized,
    /// The provider limits requests and this one went over.
    #[error("market data rate limit reached; try again in a minute")]
    RateLimited,
    /// The provider answered with another error status.
    #[error("market data returned {0}")]
    Refused(u16),
    /// The provider answered with something other than the expected shape.
    #[error("market data returned an unexpected response: {0}")]
    Malformed(String),
}
