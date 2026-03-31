//! Store configuration: bullpen.toml, then BULLPEN_* env. Only what the store needs.

use std::net::SocketAddr;

use figment::Figment;
use figment::providers::{Env, Format, Toml};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

/// Store configuration.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Process-level settings.
    pub app: App,
    /// The store service and its database.
    #[serde(default)]
    pub store: Store,
    /// Trace export.
    #[serde(default)]
    pub telemetry: Telemetry,
}

/// Process-level settings.
#[derive(Debug, Deserialize)]
pub struct App {
    /// One of development, staging, or production.
    pub env: String,
    /// The tracing filter directive.
    pub log_level: String,
}

/// The store service: where it listens, its database, and what one call may ask for.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Store {
    /// Address the HTTP surface binds.
    pub bind: String,
    /// Bearer token every route except health requires. Shared with the engine.
    pub token: Option<SecretString>,
    /// Postgres connection URL.
    pub database_url: Option<SecretString>,
    /// Pool size.
    pub max_connections: u32,
    /// Budget for taking a connection from the pool.
    pub acquire_timeout_secs: u64,
    /// Longest hold of a pending action, in seconds.
    pub max_ttl_secs: u64,
    /// Largest limit one read accepts.
    pub max_limit: usize,
    /// Largest request body, in bytes.
    pub max_body_bytes: usize,
    /// Longest label, relation, channel, tenant or user id accepted, in characters.
    pub max_key_chars: usize,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1:8090".into(),
            token: None,
            database_url: None,
            max_connections: 10,
            acquire_timeout_secs: 5,
            max_ttl_secs: 86_400,
            max_limit: 1_000,
            max_body_bytes: 4_194_304,
            max_key_chars: 200,
        }
    }
}

/// Span export to Phoenix over OTLP/HTTP. An empty phoenix_url disables it.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Telemetry {
    /// Phoenix base URL for the trace UI; unset or empty exports nothing.
    pub phoenix_url: Option<String>,
    /// Bearer token for a Phoenix that authenticates. Empty sends no Authorization header.
    pub phoenix_api_key: SecretString,
    /// The Phoenix project exported spans land in.
    pub project_name: String,
    /// Fraction of traces exported, 0.0 to 1.0.
    pub sample_ratio: f64,
    /// Budget for one export batch.
    pub export_timeout_secs: u64,
}

impl Default for Telemetry {
    fn default() -> Self {
        Self {
            phoenix_url: None,
            phoenix_api_key: SecretString::from(String::new()),
            project_name: "bullpen".into(),
            sample_ratio: 1.0,
            export_timeout_secs: 10,
        }
    }
}

/// Why the configuration cannot run.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A layer did not parse or a required value is absent.
    #[error("config: {0}")]
    Load(#[from] Box<figment::Error>),
    /// The values parse but cannot be served.
    #[error("config: {0}")]
    Invalid(String),
}

/// TOML layer read when BULLPEN_CONFIG_FILE is unset. Missing is not an error.
pub const DEFAULT_CONFIG_FILE: &str = "bullpen.toml";

/// Shortest store.token accepted.
pub const MIN_TOKEN_CHARS: usize = 16;

impl Config {
    /// Loads the TOML layer then BULLPEN_* vars, __ separates nesting. Environment values win.
    pub fn load() -> Result<Self, ConfigError> {
        let path = match std::env::var("BULLPEN_CONFIG_FILE") {
            Ok(path) => path,
            Err(std::env::VarError::NotPresent) => DEFAULT_CONFIG_FILE.to_owned(),
            Err(e) => return Err(ConfigError::Invalid(format!("BULLPEN_CONFIG_FILE: {e}"))),
        };
        let cfg: Self = Figment::new()
            .merge(Toml::file(path))
            .merge(Env::prefixed("BULLPEN_").split("__"))
            .extract()
            .map_err(Box::new)?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Rejects a combination the store cannot serve.
    pub fn validate(&self) -> Result<(), ConfigError> {
        let invalid = |m: &str| Err(ConfigError::Invalid(m.to_owned()));
        let store = &self.store;
        if store.bind.parse::<SocketAddr>().is_err() {
            return invalid("store.bind must be an address and port, like 127.0.0.1:8090");
        }
        if store
            .token
            .as_ref()
            .is_none_or(|t| t.expose_secret().trim().chars().count() < MIN_TOKEN_CHARS)
        {
            return Err(ConfigError::Invalid(format!(
                "set BULLPEN_STORE__TOKEN, at least {MIN_TOKEN_CHARS} characters"
            )));
        }
        if store.database_url.as_ref().is_none_or(|u| {
            let u = u.expose_secret().trim();
            !(u.starts_with("postgres://") || u.starts_with("postgresql://"))
        }) {
            return invalid("set BULLPEN_STORE__DATABASE_URL, a postgres:// URL");
        }
        if store.max_connections == 0 {
            return invalid("store.max_connections must be at least 1");
        }
        if store.acquire_timeout_secs == 0 {
            return invalid("store.acquire_timeout_secs must be at least 1");
        }
        if store.max_ttl_secs == 0 {
            return invalid("store.max_ttl_secs must be at least 1");
        }
        if store.max_limit == 0 {
            return invalid("store.max_limit must be at least 1");
        }
        if store.max_body_bytes == 0 {
            return invalid("store.max_body_bytes must be at least 1");
        }
        if store.max_key_chars == 0 {
            return invalid("store.max_key_chars must be at least 1");
        }
        if !(0.0..=1.0).contains(&self.telemetry.sample_ratio) {
            return invalid("telemetry.sample_ratio must be between 0 and 1");
        }
        if self.telemetry.project_name.trim().is_empty() {
            return invalid("telemetry.project_name is empty");
        }
        Ok(())
    }
}
