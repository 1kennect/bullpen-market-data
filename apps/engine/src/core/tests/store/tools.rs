//! Tool registration: the market tools register when on, and tools.disabled removes one.

use figment::Figment;
use figment::providers::{Format, Toml};

use crate::core::config::Config;
use crate::core::types::tools::{RiskClass, ToolDefinition};
use crate::wiring::tools::build_tools;

/// The sections with no defaults.
const BASE: &str = r#"
[app]
env = "test"
http_addr = "127.0.0.1:0"
log_level = "info"
[engine]
service_token = "t"
[discord]
guild_id = 1
[model]
base_url = "http://localhost:8000/v1"
api_key = ""
name = "m"
max_tokens = 256
[market]
api_key = "test"
[store]
url = "https://store.example.org"
token = "store_x"
"#;

fn config(extra: &str) -> Config {
    match Figment::new()
        .merge(Toml::string(BASE))
        .merge(Toml::string(extra))
        .extract()
    {
        Ok(cfg) => cfg,
        Err(e) => unreachable!("the test config parses: {e}"),
    }
}

/// Every tool definition the engine registers for cfg.
async fn registered(cfg: &Config) -> Vec<ToolDefinition> {
    match build_tools(cfg, None).await {
        Ok((tools, _)) => tools.definitions(),
        Err(e) => unreachable!("the tools build: {e}"),
    }
}

/// The names of definitions, in order.
fn names(definitions: &[ToolDefinition]) -> Vec<String> {
    definitions.iter().map(|d| d.name.clone()).collect()
}

#[tokio::test]
async fn every_market_tool_registers_as_a_public_read() {
    let definitions = registered(&config("")).await;
    let market: Vec<&ToolDefinition> = definitions
        .iter()
        .filter(|d| d.name.starts_with("market_"))
        .collect();
    assert_eq!(market.len(), 5, "{:?}", names(&definitions));
    assert!(market.iter().all(|d| d.risk == RiskClass::ReadPublic));
}

#[tokio::test]
async fn market_tools_are_absent_when_turned_off() {
    let definitions = registered(&config("[market]\nenabled = false\n")).await;
    assert!(
        !names(&definitions).iter().any(|n| n.starts_with("market_")),
        "{:?}",
        names(&definitions)
    );
}

#[tokio::test]
async fn a_disabled_tool_is_not_registered() {
    let definitions = registered(&config("[tools]\ndisabled = [\"market_news\"]\n")).await;
    let names = names(&definitions);
    assert!(!names.iter().any(|n| n == "market_news"), "{names:?}");
    assert!(names.iter().any(|n| n == "market_quote"), "{names:?}");
}
