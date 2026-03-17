//! Market data read API.

use async_trait::async_trait;
use chrono::NaiveDate;

use crate::core::types::tools::market::{
    CompanyProfile, MarketError, MarketStatus, Metrics, NewsItem, Quote, SymbolMatch,
};

/// Read-only access to quotes, symbols, company data, and news.
#[async_trait]
pub trait MarketData: Send + Sync {
    /// The latest quote of symbol. None when the provider knows no such symbol.
    async fn quote(&self, symbol: &str) -> Result<Option<Quote>, MarketError>;

    /// Symbols whose ticker or name matches query, best first.
    async fn search(&self, query: &str) -> Result<Vec<SymbolMatch>, MarketError>;

    /// The company behind symbol. None when the provider holds no profile.
    async fn profile(&self, symbol: &str) -> Result<Option<CompanyProfile>, MarketError>;

    /// Valuation and range figures of symbol.
    async fn metrics(&self, symbol: &str) -> Result<Metrics, MarketError>;

    /// News about symbol published from one day to another, or general market news without one.
    async fn news(
        &self,
        symbol: Option<&str>,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<NewsItem>, MarketError>;

    /// Whether exchange is trading now.
    async fn status(&self, exchange: &str) -> Result<MarketStatus, MarketError>;
}
