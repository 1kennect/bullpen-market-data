//! Market tools: argument checks, the replies the model reads, and their citations.

use std::sync::Arc;

use chrono::DateTime;
use serde_json::json;

use crate::core::tests::support::{FakeMarket, ctx};
use crate::core::traits::tools::Tool;
use crate::core::types::tools::market::{
    CompanyProfile, MarketStatus, Metrics, NewsItem, Quote, SymbolMatch,
};
use crate::core::types::tools::{ToolError, ToolOutput};
use crate::runtime::tools::market::client::quote_from_json;
use crate::runtime::tools::market::render::market_cap;
use crate::runtime::tools::market::{Limits, MarketTool, Query, normalize_symbol};

fn limits() -> Limits {
    Limits {
        max_items: 2,
        news_days: 7,
        default_exchange: "US".into(),
        summary_chars: 20,
    }
}

fn aapl() -> Quote {
    Quote {
        symbol: "AAPL".into(),
        price: 227.5,
        change: Some(-1.25),
        change_percent: Some(-0.546),
        high: Some(229.0),
        low: Some(226.1),
        open: Some(228.0),
        previous_close: Some(228.75),
        as_of: DateTime::from_timestamp(1_792_612_800, 0),
    }
}

async fn call(
    query: Query,
    fake: FakeMarket,
    args: serde_json::Value,
) -> Result<ToolOutput, ToolError> {
    MarketTool::new(query, Arc::new(fake), limits())
        .call(&ctx(), args)
        .await
}

fn ok(out: Result<ToolOutput, ToolError>) -> ToolOutput {
    match out {
        Ok(out) => out,
        Err(e) => unreachable!("the call succeeds: {e}"),
    }
}

#[test]
fn a_symbol_is_upper_cased_and_stripped_of_a_cash_tag() {
    assert_eq!(normalize_symbol(" $brk.b ").as_deref(), Ok("BRK.B"));
    assert_eq!(normalize_symbol("^gspc").as_deref(), Ok("^GSPC"));
    assert!(normalize_symbol("apple inc").is_err());
    assert!(normalize_symbol("").is_err());
    assert!(normalize_symbol(&"A".repeat(21)).is_err());
}

#[tokio::test]
async fn a_quote_states_price_move_and_time() {
    let fake = FakeMarket {
        quotes: vec![aapl()],
        ..FakeMarket::default()
    };
    let out = ok(call(Query::Quote, fake, json!({"symbol": "aapl"})).await);
    assert!(
        out.content.starts_with("AAPL last 227.50"),
        "{}",
        out.content
    );
    assert!(out.content.contains("-1.25 (-0.55%)"), "{}", out.content);
    assert!(out.content.contains("as of 2026-10-21"), "{}", out.content);
    assert!(
        out.content.contains("previous close 228.75"),
        "{}",
        out.content
    );
    assert_eq!(out.sources.len(), 1);
    assert_eq!(out.sources[0].key, "market");
}

#[tokio::test]
async fn an_unknown_symbol_points_to_the_search() {
    let out = ok(call(
        Query::Quote,
        FakeMarket::default(),
        json!({"symbol": "ZZZZ"}),
    )
    .await);
    assert!(out.content.contains("No quote for ZZZZ"), "{}", out.content);
    assert!(out.content.contains("market_search"), "{}", out.content);
    assert_eq!(out.sources, Vec::new());
}

#[tokio::test]
async fn a_quote_without_a_symbol_is_refused_before_any_call() {
    let fake = Arc::new(FakeMarket::default());
    let out = MarketTool::new(Query::Quote, fake.clone(), limits())
        .call(&ctx(), json!({}))
        .await;
    assert!(
        matches!(out, Err(ToolError::InvalidArguments(_))),
        "{out:?}"
    );
    assert_eq!(fake.calls(), Vec::<String>::new());
}

#[tokio::test]
async fn search_lists_at_most_max_items_symbols() {
    let hit = |symbol: &str| SymbolMatch {
        symbol: symbol.into(),
        description: format!("{symbol} INC"),
        kind: "Common Stock".into(),
    };
    let fake = FakeMarket {
        matches: vec![hit("PLTR"), hit("PLTR.MX"), hit("PLTR.DE")],
        ..FakeMarket::default()
    };
    let out = ok(call(Query::Search, fake, json!({"query": "palantir"})).await);
    assert!(
        out.content.contains("- PLTR: PLTR INC (Common Stock)"),
        "{}",
        out.content
    );
    assert!(!out.content.contains("PLTR.DE"), "{}", out.content);
}

#[tokio::test]
async fn a_profile_carries_its_figures_and_cites_the_website() {
    let fake = FakeMarket {
        profile: Some(CompanyProfile {
            symbol: "MSFT".into(),
            name: "Microsoft Corp".into(),
            exchange: Some("NASDAQ NMS - GLOBAL MARKET".into()),
            industry: Some("Technology".into()),
            country: Some("US".into()),
            currency: Some("USD".into()),
            market_cap_millions: Some(3_120_000.0),
            ipo: Some("1986-03-13".into()),
            website: Some("https://www.microsoft.com".into()),
        }),
        metrics: Metrics {
            pe_ttm: Some(35.124),
            week52_high: Some(468.35),
            ..Metrics::default()
        },
        ..FakeMarket::default()
    };
    let out = ok(call(Query::Profile, fake, json!({"symbol": "MSFT"})).await);
    assert!(
        out.content.contains("Microsoft Corp (MSFT)"),
        "{}",
        out.content
    );
    assert!(
        out.content.contains("market cap: 3.12T USD"),
        "{}",
        out.content
    );
    assert!(out.content.contains("P/E (TTM) 35.12"), "{}", out.content);
    assert!(
        out.sources
            .iter()
            .any(|c| c.url.as_deref() == Some("https://www.microsoft.com"))
    );
}

#[tokio::test]
async fn news_is_capped_clipped_and_cited_by_link() {
    let story = |n: i64| NewsItem {
        headline: format!("Story {n}"),
        source: "Reuters".into(),
        summary: Some("A summary that runs well past twenty characters.".into()),
        url: Some(format!("https://example.com/{n}")),
        published: DateTime::from_timestamp(1_792_612_800 - n * 60, 0),
    };
    let fake = Arc::new(FakeMarket {
        news: vec![story(1), story(2), story(3)],
        ..FakeMarket::default()
    });
    let out = ok(MarketTool::new(Query::News, fake.clone(), limits())
        .call(&ctx(), json!({"symbol": "TSLA"}))
        .await);
    assert!(
        out.content.contains("2 recent stories about TSLA"),
        "{}",
        out.content
    );
    assert!(
        out.content.contains("A summary that runs..."),
        "{}",
        out.content
    );
    assert!(!out.content.contains("Story 3"), "{}", out.content);
    assert_eq!(out.sources.len(), 2);
    assert!(fake.calls().iter().any(|c| c.starts_with("news TSLA ")));
}

#[tokio::test]
async fn news_without_a_symbol_reads_the_whole_market() {
    let fake = Arc::new(FakeMarket::default());
    let out = ok(MarketTool::new(Query::News, fake.clone(), limits())
        .call(&ctx(), json!({}))
        .await);
    assert!(
        out.content.contains("No recent news about the market"),
        "{}",
        out.content
    );
    assert!(fake.calls().iter().any(|c| c.starts_with("news market ")));
}

#[tokio::test]
async fn status_defaults_to_the_configured_exchange() {
    let fake = Arc::new(FakeMarket {
        status: Some(MarketStatus {
            exchange: "US".into(),
            is_open: true,
            session: Some("regular".into()),
            holiday: None,
            timezone: Some("America/New_York".into()),
        }),
        ..FakeMarket::default()
    });
    let out = ok(MarketTool::new(Query::Status, fake.clone(), limits())
        .call(&ctx(), json!({}))
        .await);
    assert!(
        out.content
            .contains("The US market is open, session: regular"),
        "{}",
        out.content
    );
    assert_eq!(fake.calls(), vec!["status US".to_owned()]);
}

#[tokio::test]
async fn a_rate_limit_reaches_the_model_as_a_failure() {
    let fake = FakeMarket {
        rate_limited: true,
        ..FakeMarket::default()
    };
    let out = call(Query::Quote, fake, json!({"symbol": "AAPL"})).await;
    assert!(
        matches!(&out, Err(ToolError::Failed(m)) if m.contains("rate limit")),
        "{out:?}"
    );
}

#[test]
fn an_all_zero_quote_reads_as_an_unknown_symbol() {
    let zero = r#"{"c":0,"d":null,"dp":null,"h":0,"l":0,"o":0,"pc":0,"t":0}"#;
    assert!(matches!(quote_from_json("ZZZZ", zero), Ok(None)));
    let live = r#"{"c":261.74,"d":1.5,"dp":0.58,"h":263.31,"l":260.68,"o":261.07,"pc":260.24,"t":1792612800}"#;
    let quote = match quote_from_json("AAPL", live) {
        Ok(Some(quote)) => quote,
        other => unreachable!("a live quote parses: {other:?}"),
    };
    assert!((quote.price - 261.74).abs() < f64::EPSILON);
    assert!(quote.as_of.is_some());
    assert!(quote_from_json("AAPL", "not json").is_err());
}

#[test]
fn market_cap_scales_to_the_largest_whole_unit() {
    assert_eq!(market_cap(3_120_000.0), "3.12T");
    assert_eq!(market_cap(45_300.0), "45.30B");
    assert_eq!(market_cap(812.25), "812.2M");
}
