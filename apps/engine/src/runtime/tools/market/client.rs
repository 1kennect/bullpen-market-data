//! Finnhub REST client over reqwest. Read-only; the key travels in a header, never in a URL.

use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use reqwest::{Client, StatusCode};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::core::traits::tools::market::MarketData;
use crate::core::types::tools::market::{
    CompanyProfile, MarketError, MarketStatus, Metrics, NewsItem, Quote, SymbolMatch,
};
use crate::runtime::tools::http;

/// Header Finnhub reads the API key from.
const KEY_HEADER: &str = "X-Finnhub-Token";

/// A Finnhub endpoint reached over HTTPS.
pub struct FinnhubClient {
    http: Client,
    base_url: String,
    api_key: SecretString,
}

/// A quote as Finnhub returns it. Every price is zero for a symbol it does not know.
#[derive(Debug, Deserialize)]
struct RawQuote {
    #[serde(default)]
    c: Option<f64>,
    #[serde(default)]
    d: Option<f64>,
    #[serde(default)]
    dp: Option<f64>,
    #[serde(default)]
    h: Option<f64>,
    #[serde(default)]
    l: Option<f64>,
    #[serde(default)]
    o: Option<f64>,
    #[serde(default)]
    pc: Option<f64>,
    #[serde(default)]
    t: Option<i64>,
}

/// A symbol search response.
#[derive(Debug, Deserialize)]
struct RawSearch {
    #[serde(default = "Vec::new")]
    result: Vec<RawMatch>,
}

/// One symbol search hit.
#[derive(Debug, Deserialize)]
struct RawMatch {
    #[serde(default)]
    symbol: String,
    #[serde(default)]
    description: String,
    #[serde(default, rename = "type")]
    kind: String,
}

/// A company profile. Empty for a symbol Finnhub holds no profile for.
#[derive(Debug, Deserialize)]
struct RawProfile {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    exchange: Option<String>,
    #[serde(default, rename = "finnhubIndustry")]
    industry: Option<String>,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    currency: Option<String>,
    #[serde(default, rename = "marketCapitalization")]
    market_cap: Option<f64>,
    #[serde(default)]
    ipo: Option<String>,
    #[serde(default)]
    weburl: Option<String>,
}

/// The basic financials response: a flat map of named figures.
#[derive(Debug, Deserialize)]
struct RawMetrics {
    #[serde(default)]
    metric: HashMap<String, Value>,
}

/// One news story as Finnhub returns it.
#[derive(Debug, Deserialize)]
struct RawNews {
    #[serde(default)]
    headline: String,
    #[serde(default)]
    source: String,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    datetime: Option<i64>,
}

/// The market status response.
#[derive(Debug, Deserialize)]
struct RawStatus {
    #[serde(default)]
    exchange: Option<String>,
    #[serde(default, rename = "isOpen")]
    is_open: bool,
    #[serde(default)]
    session: Option<String>,
    #[serde(default)]
    holiday: Option<String>,
    #[serde(default)]
    timezone: Option<String>,
}

impl FinnhubClient {
    /// Builds the client over the API base URL, its key, and the request budget.
    pub fn new(
        base_url: &str,
        api_key: SecretString,
        timeout: Duration,
    ) -> Result<Self, MarketError> {
        let http = http::client(timeout)
            .map_err(|e| MarketError::Unreachable(http::failure(&e).to_owned()))?;
        Ok(Self {
            http,
            base_url: base_url.trim_end_matches('/').to_owned(),
            api_key,
        })
    }

    /// One authenticated GET of an API path, deserialized into T.
    async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T, MarketError> {
        let url = reqwest::Url::parse_with_params(&format!("{}/{path}", self.base_url), query)
            .map_err(|e| MarketError::Unreachable(e.to_string()))?;
        let response = self
            .http
            .get(url)
            .header(KEY_HEADER, self.api_key.expose_secret())
            .send()
            .await
            .map_err(|e| MarketError::Unreachable(http::failure(&e).to_owned()))?;
        match response.status() {
            status if status.is_success() => {}
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                return Err(MarketError::Unauthorized);
            }
            StatusCode::TOO_MANY_REQUESTS => return Err(MarketError::RateLimited),
            status => return Err(MarketError::Refused(status.as_u16())),
        }
        let body = response
            .text()
            .await
            .map_err(|e| MarketError::Unreachable(http::failure(&e).to_owned()))?;
        serde_json::from_str(&body).map_err(|e| MarketError::Malformed(e.to_string()))
    }
}

/// A unix timestamp in seconds as a UTC time. None for zero or out of range.
fn instant(seconds: Option<i64>) -> Option<DateTime<Utc>> {
    seconds
        .filter(|s| *s > 0)
        .and_then(|s| DateTime::from_timestamp(s, 0))
}

/// The first named figure that is a number.
fn figure(metrics: &HashMap<String, Value>, names: &[&str]) -> Option<f64> {
    names
        .iter()
        .find_map(|name| metrics.get(*name).and_then(Value::as_f64))
}

/// Text that is present and not blank.
fn present(text: Option<String>) -> Option<String> {
    text.map(|t| t.trim().to_owned()).filter(|t| !t.is_empty())
}

/// The quote a raw JSON body describes. None when every price is zero, as for an unknown symbol.
#[cfg(test)]
pub fn quote_from_json(symbol: &str, body: &str) -> Result<Option<Quote>, MarketError> {
    let raw: RawQuote =
        serde_json::from_str(body).map_err(|e| MarketError::Malformed(e.to_string()))?;
    Ok(to_quote(symbol, &raw))
}

/// The quote a raw reply describes, or None for an unknown symbol.
fn to_quote(symbol: &str, raw: &RawQuote) -> Option<Quote> {
    let price = raw.c.unwrap_or_default();
    if price == 0.0 && raw.pc.unwrap_or_default() == 0.0 {
        return None;
    }
    Some(Quote {
        symbol: symbol.to_owned(),
        price,
        change: raw.d,
        change_percent: raw.dp,
        high: raw.h,
        low: raw.l,
        open: raw.o,
        previous_close: raw.pc,
        as_of: instant(raw.t),
    })
}

#[async_trait]
impl MarketData for FinnhubClient {
    async fn quote(&self, symbol: &str) -> Result<Option<Quote>, MarketError> {
        let raw: RawQuote = self.get("quote", &[("symbol", symbol)]).await?;
        Ok(to_quote(symbol, &raw))
    }

    async fn search(&self, query: &str) -> Result<Vec<SymbolMatch>, MarketError> {
        let raw: RawSearch = self.get("search", &[("q", query)]).await?;
        Ok(raw
            .result
            .into_iter()
            .filter(|m| !m.symbol.trim().is_empty())
            .map(|m| SymbolMatch {
                symbol: m.symbol,
                description: m.description,
                kind: m.kind,
            })
            .collect())
    }

    async fn profile(&self, symbol: &str) -> Result<Option<CompanyProfile>, MarketError> {
        let raw: RawProfile = self.get("stock/profile2", &[("symbol", symbol)]).await?;
        let Some(name) = present(raw.name) else {
            return Ok(None);
        };
        Ok(Some(CompanyProfile {
            symbol: symbol.to_owned(),
            name,
            exchange: present(raw.exchange),
            industry: present(raw.industry),
            country: present(raw.country),
            currency: present(raw.currency),
            market_cap_millions: raw.market_cap.filter(|m| *m > 0.0),
            ipo: present(raw.ipo),
            website: present(raw.weburl),
        }))
    }

    async fn metrics(&self, symbol: &str) -> Result<Metrics, MarketError> {
        let raw: RawMetrics = self
            .get("stock/metric", &[("symbol", symbol), ("metric", "all")])
            .await?;
        let m = &raw.metric;
        Ok(Metrics {
            week52_high: figure(m, &["52WeekHigh"]),
            week52_low: figure(m, &["52WeekLow"]),
            pe_ttm: figure(m, &["peTTM", "peBasicExclExtraTTM", "peExclExtraTTM"]),
            eps_ttm: figure(
                m,
                &[
                    "epsTTM",
                    "epsBasicExclExtraItemsTTM",
                    "epsExclExtraItemsTTM",
                ],
            ),
            dividend_yield: figure(
                m,
                &["dividendYieldIndicatedAnnual", "currentDividendYieldTTM"],
            ),
            beta: figure(m, &["beta"]),
        })
    }

    async fn news(
        &self,
        symbol: Option<&str>,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<NewsItem>, MarketError> {
        let raw: Vec<RawNews> = match symbol {
            Some(symbol) => {
                let from = from.format("%Y-%m-%d").to_string();
                let to = to.format("%Y-%m-%d").to_string();
                self.get(
                    "company-news",
                    &[("symbol", symbol), ("from", &from), ("to", &to)],
                )
                .await?
            }
            None => self.get("news", &[("category", "general")]).await?,
        };
        let mut items: Vec<NewsItem> = raw
            .into_iter()
            .filter(|n| !n.headline.trim().is_empty())
            .map(|n| NewsItem {
                headline: n.headline.trim().to_owned(),
                source: n.source,
                summary: present(n.summary),
                url: present(n.url),
                published: instant(n.datetime),
            })
            .collect();
        items.sort_by_key(|n| std::cmp::Reverse(n.published));
        Ok(items)
    }

    async fn status(&self, exchange: &str) -> Result<MarketStatus, MarketError> {
        let raw: RawStatus = self
            .get("stock/market-status", &[("exchange", exchange)])
            .await?;
        Ok(MarketStatus {
            exchange: present(raw.exchange).unwrap_or_else(|| exchange.to_owned()),
            is_open: raw.is_open,
            session: present(raw.session),
            holiday: present(raw.holiday),
            timezone: present(raw.timezone),
        })
    }
}
