//! The store section: what it needs, and what it rules out.

use figment::Figment;
use figment::providers::{Format, Toml};

use crate::core::config::Config;

/// A complete configuration.
const ON: &str = r#"
[app]
env = "test"
log_level = "info"
[store]
token = "0123456789abcdef"
database_url = "postgres://bullpen@localhost/bullpen"
"#;

fn load(extra: &str) -> Result<Config, String> {
    let cfg: Config = Figment::new()
        .merge(Toml::string(ON))
        .merge(Toml::string(extra))
        .extract()
        .map_err(|e| e.to_string())?;
    cfg.validate().map_err(|e| e.to_string())?;
    Ok(cfg)
}

#[test]
fn a_complete_section_boots_with_its_defaults() {
    let Ok(cfg) = load("") else {
        unreachable!("a complete section is valid")
    };
    assert_eq!(cfg.store.bind, "127.0.0.1:8090");
    assert_eq!(cfg.store.max_ttl_secs, 86_400);
}

#[test]
fn the_store_needs_a_long_token_and_a_postgres_url() {
    let short = "[store]\ntoken = \"short\"\n";
    assert!(load(short).is_err_and(|e| e.contains("BULLPEN_STORE__TOKEN")));
    let not_pg = "[store]\ndatabase_url = \"mysql://x\"\n";
    assert!(load(not_pg).is_err_and(|e| e.contains("BULLPEN_STORE__DATABASE_URL")));
}

#[test]
fn settings_the_store_cannot_serve_are_rejected() {
    for bad in [
        "bind = \"nowhere\"",
        "max_connections = 0",
        "max_ttl_secs = 0",
        "max_limit = 0",
        "max_key_chars = 0",
    ] {
        assert!(load(&format!("[store]\n{bad}\n")).is_err(), "{bad}");
    }
}

#[test]
fn engine_keys_in_the_shared_section_are_ignored() {
    assert!(load("[store]\nurl = \"http://127.0.0.1:8090\"\ntimeout_secs = 10\n").is_ok());
}
