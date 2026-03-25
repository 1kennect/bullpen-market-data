//! The store section: what it needs, and what it rules out.

use figment::Figment;
use figment::providers::{Format, Toml};

use crate::core::config::Config;

/// The sections with no defaults, without the store section.
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
"#;

/// A complete store section.
const ON: &str = r#"
[store]
url = "https://store.example.org"
token = "store_x"
"#;

fn load(extra: &str) -> Result<Config, String> {
    let cfg: Config = Figment::new()
        .merge(Toml::string(BASE))
        .merge(Toml::string(extra))
        .extract()
        .map_err(|e| e.to_string())?;
    cfg.validate().map_err(|e| e.to_string())?;
    Ok(cfg)
}

#[test]
fn a_complete_store_section_boots_with_its_defaults() {
    let Ok(cfg) = load(ON) else {
        unreachable!("a complete store section is valid")
    };
    assert_eq!(cfg.store.timeout_secs, 10);
    assert_eq!(cfg.store.max_ttl_secs, 86_400);
}

#[test]
fn the_store_needs_a_url_and_a_token() {
    assert!(load("").is_err_and(|e| e.contains("BULLPEN_STORE__URL")));
    let no_url = "[store]\ntoken = \"store_x\"\n";
    assert!(load(no_url).is_err_and(|e| e.contains("BULLPEN_STORE__URL")));
    let bad_url = "[store]\nurl = \"ftp://x\"\ntoken = \"store_x\"\n";
    assert!(load(bad_url).is_err_and(|e| e.contains("BULLPEN_STORE__URL")));
    let no_token = "[store]\nurl = \"https://s.example\"\n";
    assert!(load(no_token).is_err_and(|e| e.contains("BULLPEN_STORE__TOKEN")));
}

#[test]
fn settings_the_store_cannot_serve_are_rejected() {
    let with = |more: &str| load(&format!("{ON}{more}"));
    assert!(with("timeout_secs = 0\n").is_err());
    assert!(
        with("[agent]\nconfirmation_ttl_secs = 90000\n")
            .is_err_and(|e| e.contains("confirmation_ttl_secs"))
    );
    assert!(with("max_ttl_secs = 100000\n[agent]\nconfirmation_ttl_secs = 90000\n").is_ok());
}

#[test]
fn no_hosted_tool_is_private_unless_told_otherwise() {
    let Ok(cfg) = load(ON) else {
        unreachable!("a complete store section is valid")
    };
    assert_eq!(cfg.mcp.hosted.private, Vec::<String>::new());
    assert!(load(&format!("{ON}[mcp.hosted]\nprivate = [\"portfolio.\"]\n")).is_ok());
    assert!(
        load(&format!("{ON}[mcp.hosted]\nprivate = [\"\"]\n"))
            .is_err_and(|e| e.contains("mcp.hosted.private"))
    );
}
