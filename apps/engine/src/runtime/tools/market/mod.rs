//! Market tools: quotes, symbol search, company profiles, news, and exchange status. ReadPublic.

pub mod client;
pub mod render;

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{Days, Utc};
use serde_json::{Value, json};

use crate::core::config::Market as MarketConfig;
use crate::core::traits::tools::Tool;
use crate::core::traits::tools::market::MarketData;
use crate::core::types::agent::context::RequestContext;
use crate::core::types::tools::market::{
    MARKET_NEWS, MARKET_PROFILE, MARKET_QUOTE, MARKET_SEARCH, MARKET_STATUS, MarketError, Metrics,
};
use crate::core::types::tools::{RiskClass, ToolDefinition, ToolError, ToolOutput};
use crate::runtime::tools::market::client::FinnhubClient;
use crate::runtime::tools::market::render::{
    news_output, profile_output, quote_output, search_output, status_output,
};
use crate::runtime::tools::{query_arg, query_schema};

/// Name of the symbol parameter.
const SYMBOL: &str = "symbol";
/// Name of the exchange parameter.
const EXCHANGE: &str = "exchange";
/// Longest symbol accepted.
const MAX_SYMBOL_CHARS: usize = 20;

/// Which read a market tool performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Query {
    /// The latest price of one symbol.
    Quote,
    /// Symbols matching a company name or ticker.
    Search,
    /// What a company is, with its valuation figures.
    Profile,
    /// Recent news about one symbol or the market.
    News,
    /// Whether an exchange is trading now.
    Status,
}

/// Every read a market tool is built for.
pub const ALL: [Query; 5] = [
    Query::Quote,
    Query::Search,
    Query::Profile,
    Query::News,
    Query::Status,
];

impl Query {
    /// The tool name the model calls.
    pub fn name(self) -> &'static str {
        match self {
            Self::Quote => MARKET_QUOTE,
            Self::Search => MARKET_SEARCH,
            Self::Profile => MARKET_PROFILE,
            Self::News => MARKET_NEWS,
            Self::Status => MARKET_STATUS,
        }
    }

    /// What the tool does, for the model.
    fn description(self) -> &'static str {
        match self {
            Self::Quote => {
                "Get the latest price of one stock, ETF, or index symbol: last price, change on \
                 the day, open, high, low, previous close, and when it was last updated. Call it \
                 once per symbol; several calls in one step run at the same time."
            }
            Self::Search => {
                "Find the ticker symbol of a company or fund by name or partial ticker. Use it \
                 before market_quote whenever the user names a company rather than a symbol."
            }
            Self::Profile => {
                "Describe the company behind a symbol: name, exchange, industry, country, market \
                 cap, IPO date, website, and valuation figures such as P/E, EPS, 52-week range, \
                 dividend yield, and beta."
            }
            Self::News => {
                "Recent news headlines with summaries and links, newest first. Give a symbol for \
                 news about one company; leave it out for general market news."
            }
            Self::Status => {
                "Whether an exchange is open now, which session it is in, and any holiday it is \
                 closed for."
            }
        }
    }

    /// The JSON schema of its arguments.
    fn parameters(self) -> Value {
        let symbol = json!({
            "type": "string",
            "description": "Ticker symbol as the exchange lists it, such as AAPL, BRK.B or SPY.",
        });
        match self {
            Self::Quote | Self::Profile => json!({
                "type": "object",
                "properties": { SYMBOL: symbol },
                "required": [SYMBOL],
            }),
            Self::Search => query_schema("Company or fund name, or part of a ticker."),
            Self::News => json!({
                "type": "object",
                "properties": { SYMBOL: symbol },
            }),
            Self::Status => json!({
                "type": "object",
                "properties": { EXCHANGE: {
                    "type": "string",
                    "description": "Exchange code, such as US, L or T. Leave out for US.",
                } },
            }),
        }
    }
}

/// How a market tool reads and how much it hands back.
#[derive(Debug, Clone)]
pub struct Limits {
    /// Most rows one list reply holds.
    pub max_items: usize,
    /// Days of company news read back from today.
    pub news_days: u32,
    /// Exchange market_status reports when a call names none.
    pub default_exchange: String,
    /// Longest news summary kept, in characters.
    pub summary_chars: usize,
}

impl From<&MarketConfig> for Limits {
    fn from(cfg: &MarketConfig) -> Self {
        Self {
            max_items: cfg.max_items,
            news_days: cfg.news_days,
            default_exchange: cfg.default_exchange.clone(),
            summary_chars: cfg.summary_chars,
        }
    }
}

/// One market read, offered to the model.
pub struct MarketTool {
    query: Query,
    client: Arc<dyn MarketData>,
    limits: Limits,
}

impl MarketTool {
    /// Builds a tool over the market data client.
    pub fn new(query: Query, client: Arc<dyn MarketData>, limits: Limits) -> Self {
        Self {
            query,
            client,
            limits,
        }
    }
}

/// A symbol as the provider takes it: upper case, letters, digits, and . - : ^ = only.
pub fn normalize_symbol(raw: &str) -> Result<String, String> {
    let symbol = raw.trim().trim_start_matches('$').to_ascii_uppercase();
    if symbol.is_empty() {
        return Err(format!("{SYMBOL} is empty; give a ticker such as AAPL"));
    }
    if symbol.chars().count() > MAX_SYMBOL_CHARS {
        return Err(format!(
            "{SYMBOL} {symbol:?} is longer than {MAX_SYMBOL_CHARS} characters; \
             find the ticker with market_search"
        ));
    }
    if !symbol
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':' | '^' | '='))
    {
        return Err(format!(
            "{SYMBOL} {symbol:?} is not a ticker; find it with market_search"
        ));
    }
    Ok(symbol)
}

/// The symbol a call carries. None when it is optional and left out. Err is shown to the model.
fn symbol_arg(args: &Value, required: bool) -> Result<Option<String>, ToolError> {
    match args.get(SYMBOL) {
        Some(Value::String(text)) if !text.trim().is_empty() => normalize_symbol(text)
            .map(Some)
            .map_err(ToolError::InvalidArguments),
        None | Some(Value::Null | Value::String(_)) if !required => Ok(None),
        _ => Err(ToolError::InvalidArguments(format!(
            "{SYMBOL} is required: a ticker such as AAPL"
        ))),
    }
}

/// Maps a provider failure to the text the model reads.
fn failed(error: &MarketError) -> ToolError {
    ToolError::Failed(error.to_string())
}

#[async_trait]
impl Tool for MarketTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.query.name().to_owned(),
            description: self.query.description().to_owned(),
            parameters: self.query.parameters(),
            risk: RiskClass::ReadPublic,
            sequential: false,
            timeout_secs: None,
        }
    }

    async fn call(&self, _ctx: &RequestContext, args: Value) -> Result<ToolOutput, ToolError> {
        let limits = &self.limits;
        match self.query {
            Query::Quote => {
                let symbol = symbol_arg(&args, true)?.unwrap_or_default();
                let quote = self.client.quote(&symbol).await.map_err(|e| failed(&e))?;
                Ok(quote_output(&symbol, quote))
            }
            Query::Search => {
                let query = query_arg(&args)?;
                let found = self.client.search(&query).await.map_err(|e| failed(&e))?;
                Ok(search_output(&query, found, limits.max_items))
            }
            Query::Profile => {
                let symbol = symbol_arg(&args, true)?.unwrap_or_default();
                let (profile, metrics) =
                    tokio::join!(self.client.profile(&symbol), self.client.metrics(&symbol));
                let profile = profile.map_err(|e| failed(&e))?;
                // Figures are extra; a profile without them still answers.
                let metrics = metrics.unwrap_or_else(|error| {
                    tracing::warn!(%error, symbol, "market figures unavailable");
                    Metrics::default()
                });
                Ok(profile_output(&symbol, profile, &metrics))
            }
            Query::News => {
                let symbol = symbol_arg(&args, false)?;
                let to = Utc::now().date_naive();
                let from = to
                    .checked_sub_days(Days::new(u64::from(limits.news_days)))
                    .unwrap_or(to);
                let news = self
                    .client
                    .news(symbol.as_deref(), from, to)
                    .await
                    .map_err(|e| failed(&e))?;
                Ok(news_output(
                    symbol.as_deref(),
                    news,
                    limits.max_items,
                    limits.summary_chars,
                ))
            }
            Query::Status => {
                let exchange = match args.get(EXCHANGE) {
                    Some(Value::String(code)) if !code.trim().is_empty() => {
                        code.trim().to_ascii_uppercase()
                    }
                    _ => limits.default_exchange.clone(),
                };
                let status = self
                    .client
                    .status(&exchange)
                    .await
                    .map_err(|e| failed(&e))?;
                Ok(status_output(&status))
            }
        }
    }
}

/// The market tools over one client.
pub fn tools_over(client: &Arc<dyn MarketData>, limits: &Limits) -> Vec<Arc<dyn Tool>> {
    ALL.into_iter()
        .map(|q| Arc::new(MarketTool::new(q, client.clone(), limits.clone())) as Arc<dyn Tool>)
        .collect()
}

/// The market tools over the Finnhub client. Config::validate guarantees the key.
pub fn tools(cfg: &MarketConfig) -> Result<Vec<Arc<dyn Tool>>, MarketError> {
    let Some(key) = cfg.api_key.clone() else {
        return Err(MarketError::Unauthorized);
    };
    let client: Arc<dyn MarketData> = Arc::new(FinnhubClient::new(
        &cfg.base_url,
        key,
        Duration::from_secs(cfg.timeout_secs),
    )?);
    Ok(tools_over(&client, &Limits::from(cfg)))
}
