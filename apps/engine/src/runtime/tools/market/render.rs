//! Rendering of the quotes, symbols, profiles, news, and status the market tools hand back.

use std::fmt::Write as _;

use chrono::{DateTime, Utc};

use crate::core::types::knowledge::evidence::Citation;
use crate::core::types::tools::ToolOutput;
use crate::core::types::tools::market::{
    CompanyProfile, MarketStatus, Metrics, NewsItem, Quote, SymbolMatch,
};
use crate::runtime::tools::structured;

/// Key every market citation carries.
const SOURCE_KEY: &str = "market";

/// Name of the provider, as a citation of its data is labelled.
const PROVIDER: &str = "Finnhub";

/// A time as the model reads it, in UTC to the minute.
fn stamp(at: DateTime<Utc>) -> String {
    at.format("%Y-%m-%d %H:%M UTC").to_string()
}

/// A price or figure to two decimals.
fn money(value: f64) -> String {
    format!("{value:.2}")
}

/// A signed change to two decimals.
fn signed(value: f64) -> String {
    format!("{value:+.2}")
}

/// A market capitalization given in millions, scaled to millions, billions, or trillions.
pub fn market_cap(millions: f64) -> String {
    if millions >= 1_000_000.0 {
        format!("{:.2}T", millions / 1_000_000.0)
    } else if millions >= 1_000.0 {
        format!("{:.2}B", millions / 1_000.0)
    } else {
        format!("{millions:.1}M")
    }
}

/// Text held to max characters, with an ellipsis when cut.
fn clip(text: &str, max: usize) -> String {
    if text.chars().count() > max {
        text.chars()
            .take(max)
            .collect::<String>()
            .trim_end()
            .to_owned()
            + "..."
    } else {
        text.to_owned()
    }
}

/// The citation for data read from the provider itself, with no page of its own.
fn provider(title: String) -> Citation {
    Citation {
        key: SOURCE_KEY.to_owned(),
        title,
        url: None,
    }
}

/// The quote reply. None means the provider knows no such symbol.
pub fn quote_output(symbol: &str, quote: Option<Quote>) -> ToolOutput {
    let Some(q) = quote else {
        return ToolOutput {
            content: format!(
                "No quote for {symbol}. The symbol may be wrong or not covered; \
                 look it up with market_search."
            ),
            data: None,
            sources: Vec::new(),
        };
    };
    let mut text = format!("{} last {}", q.symbol, money(q.price));
    if let (Some(change), Some(percent)) = (q.change, q.change_percent) {
        let _ = write!(
            text,
            ", {} ({}%) on the day",
            signed(change),
            signed(percent)
        );
    }
    match q.as_of {
        Some(at) => {
            let _ = write!(text, ", as of {}", stamp(at));
        }
        None => text.push_str(", time of last trade not given"),
    }
    text.push('.');
    let mut session = Vec::new();
    for (label, value) in [
        ("open", q.open),
        ("high", q.high),
        ("low", q.low),
        ("previous close", q.previous_close),
    ] {
        if let Some(value) = value.filter(|v| *v != 0.0) {
            session.push(format!("{label} {}", money(value)));
        }
    }
    if !session.is_empty() {
        let _ = write!(text, "\nSession: {}.", session.join(", "));
    }
    ToolOutput {
        content: text,
        data: structured(&q),
        sources: vec![provider(format!("{PROVIDER} quote for {}", q.symbol))],
    }
}

/// The symbol search reply.
pub fn search_output(query: &str, matches: Vec<SymbolMatch>, max: usize) -> ToolOutput {
    let shown: Vec<SymbolMatch> = matches.into_iter().take(max).collect();
    let text = if shown.is_empty() {
        format!("No symbols match {query:?}. Try the company name spelled differently.")
    } else {
        let mut lines = format!("{} symbols match {query:?}:", shown.len());
        for m in &shown {
            let kind = if m.kind.trim().is_empty() {
                String::new()
            } else {
                format!(" ({})", m.kind.trim())
            };
            let _ = write!(lines, "\n- {}: {}{kind}", m.symbol, m.description.trim());
        }
        lines
    };
    ToolOutput {
        content: text,
        data: structured(&shown),
        sources: Vec::new(),
    }
}

/// The company profile reply, with its metrics.
pub fn profile_output(
    symbol: &str,
    profile: Option<CompanyProfile>,
    metrics: &Metrics,
) -> ToolOutput {
    let mut text = String::new();
    let mut sources = Vec::new();
    match &profile {
        Some(p) => {
            let _ = write!(text, "{} ({})", p.name, p.symbol);
            for (label, value) in [
                ("exchange", &p.exchange),
                ("industry", &p.industry),
                ("country", &p.country),
                ("IPO", &p.ipo),
            ] {
                if let Some(value) = value {
                    let _ = write!(text, "\n{label}: {value}");
                }
            }
            if let Some(cap) = p.market_cap_millions {
                let currency = p.currency.as_deref().unwrap_or("");
                let _ = write!(text, "\nmarket cap: {} {currency}", market_cap(cap));
                text.truncate(text.trim_end().len());
            }
            if let Some(site) = &p.website {
                let _ = write!(text, "\nwebsite: {site}");
                sources.push(Citation {
                    key: SOURCE_KEY.to_owned(),
                    title: p.name.clone(),
                    url: Some(site.clone()),
                });
            }
        }
        None => {
            let _ = write!(text, "No company profile for {symbol}.");
        }
    }
    let mut figures = Vec::new();
    for (label, value) in [
        ("52-week high", metrics.week52_high),
        ("52-week low", metrics.week52_low),
        ("P/E (TTM)", metrics.pe_ttm),
        ("EPS (TTM)", metrics.eps_ttm),
        ("dividend yield %", metrics.dividend_yield),
        ("beta", metrics.beta),
    ] {
        if let Some(value) = value {
            figures.push(format!("{label} {}", money(value)));
        }
    }
    if !figures.is_empty() {
        let _ = write!(text, "\nFigures: {}.", figures.join(", "));
        sources.push(provider(format!("{PROVIDER} figures for {symbol}")));
    }
    ToolOutput {
        content: text,
        data: structured(&(profile, metrics)),
        sources,
    }
}

/// The news reply, newest first.
pub fn news_output(
    symbol: Option<&str>,
    news: Vec<NewsItem>,
    max: usize,
    summary_chars: usize,
) -> ToolOutput {
    let about = symbol.map_or_else(|| "the market".to_owned(), str::to_owned);
    let shown: Vec<NewsItem> = news.into_iter().take(max).collect();
    let text = if shown.is_empty() {
        format!("No recent news about {about}.")
    } else {
        let mut lines = format!(
            "{} recent stories about {about}, newest first:",
            shown.len()
        );
        for n in &shown {
            let when = n
                .published
                .map(|at| format!("{}, ", stamp(at)))
                .unwrap_or_default();
            let _ = write!(lines, "\n- {when}{}: {}", n.source.trim(), n.headline);
            if let Some(summary) = &n.summary {
                let _ = write!(lines, "\n  {}", clip(summary, summary_chars));
            }
            if let Some(url) = &n.url {
                let _ = write!(lines, "\n  {url}");
            }
        }
        lines
    };
    let sources = shown
        .iter()
        .filter(|n| n.url.is_some())
        .map(|n| Citation {
            key: SOURCE_KEY.to_owned(),
            title: n.headline.clone(),
            url: n.url.clone(),
        })
        .collect();
    ToolOutput {
        content: text,
        data: structured(&shown),
        sources,
    }
}

/// The market status reply.
pub fn status_output(status: &MarketStatus) -> ToolOutput {
    let mut text = format!(
        "The {} market is {}",
        status.exchange,
        if status.is_open { "open" } else { "closed" }
    );
    if let Some(session) = &status.session {
        let _ = write!(text, ", session: {session}");
    }
    if let Some(holiday) = &status.holiday {
        let _ = write!(text, ", closed for {holiday}");
    }
    if let Some(zone) = &status.timezone {
        let _ = write!(text, " (exchange time zone {zone})");
    }
    text.push('.');
    ToolOutput {
        content: text,
        data: structured(status),
        sources: vec![provider(format!("{PROVIDER} market status"))],
    }
}
