//! Market data doubles: canned quotes, symbols, profiles, news, and status, with the calls seen.

use std::sync::Mutex;

use async_trait::async_trait;
use chrono::NaiveDate;

use crate::core::traits::tools::market::MarketData;
use crate::core::types::tools::market::{
    CompanyProfile, MarketError, MarketStatus, Metrics, NewsItem, Quote, SymbolMatch,
};

/// Market data with canned answers. A failure, when set, answers every call.
#[derive(Default)]
pub struct FakeMarket {
    /// Quotes by symbol.
    pub quotes: Vec<Quote>,
    /// What every search returns.
    pub matches: Vec<SymbolMatch>,
    /// The profile every profile call returns.
    pub profile: Option<CompanyProfile>,
    /// The figures every metrics call returns.
    pub metrics: Metrics,
    /// What every news call returns.
    pub news: Vec<NewsItem>,
    /// What every status call returns.
    pub status: Option<MarketStatus>,
    /// Makes every call fail with rate limiting.
    pub rate_limited: bool,
    /// The symbol, or none, and exchange of each call, in order.
    pub seen: Mutex<Vec<String>>,
}

impl FakeMarket {
    /// Records one call, failing when the fake is rate limited.
    fn record(&self, call: String) -> Result<(), MarketError> {
        if let Ok(mut seen) = self.seen.lock() {
            seen.push(call);
        }
        if self.rate_limited {
            return Err(MarketError::RateLimited);
        }
        Ok(())
    }

    /// Every call seen, in order.
    pub fn calls(&self) -> Vec<String> {
        self.seen.lock().map(|s| s.clone()).unwrap_or_default()
    }
}

#[async_trait]
impl MarketData for FakeMarket {
    async fn quote(&self, symbol: &str) -> Result<Option<Quote>, MarketError> {
        self.record(format!("quote {symbol}"))?;
        Ok(self.quotes.iter().find(|q| q.symbol == symbol).cloned())
    }

    async fn search(&self, query: &str) -> Result<Vec<SymbolMatch>, MarketError> {
        self.record(format!("search {query}"))?;
        Ok(self.matches.clone())
    }

    async fn profile(&self, symbol: &str) -> Result<Option<CompanyProfile>, MarketError> {
        self.record(format!("profile {symbol}"))?;
        Ok(self.profile.clone())
    }

    async fn metrics(&self, symbol: &str) -> Result<Metrics, MarketError> {
        self.record(format!("metrics {symbol}"))?;
        Ok(self.metrics.clone())
    }

    async fn news(
        &self,
        symbol: Option<&str>,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<NewsItem>, MarketError> {
        self.record(format!("news {} {from} {to}", symbol.unwrap_or("market")))?;
        Ok(self.news.clone())
    }

    async fn status(&self, exchange: &str) -> Result<MarketStatus, MarketError> {
        self.record(format!("status {exchange}"))?;
        Ok(self.status.clone().unwrap_or(MarketStatus {
            exchange: exchange.to_owned(),
            is_open: false,
            session: None,
            holiday: None,
            timezone: None,
        }))
    }
}
