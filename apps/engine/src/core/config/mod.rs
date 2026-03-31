//! Configuration. Every external service and harness knob is configured here, nowhere else.

pub mod harness;
pub mod http;
pub mod integrations;
pub mod services;

use figment::Figment;
use figment::providers::{Env, Format, Toml};
use secrecy::ExposeSecret;
use serde::Deserialize;

use crate::core::types::agent::AgentConfig;
use crate::core::types::agent::assemble::Budget;

pub use self::harness::*;
pub use self::http::*;
pub use self::integrations::*;
pub use self::services::*;

/// TOML layer read when BULLPEN_CONFIG_FILE is unset. Missing is not an error.
pub const DEFAULT_CONFIG_FILE: &str = "bullpen.toml";

/// Root configuration, loaded from a TOML file and BULLPEN_* environment variables.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Process-level settings.
    pub app: App,
    /// What clients must present to call the engine.
    pub engine: Engine,
    /// Discord bot settings.
    pub discord: Discord,
    /// Chat model endpoint.
    pub model: Model,
    /// OpenTelemetry export.
    #[serde(default)]
    pub telemetry: Telemetry,
    /// Loop limits and prompt budgets.
    #[serde(default)]
    pub agent: Agent,
    /// The text the harness writes around every prompt.
    #[serde(default)]
    pub prompt: Prompt,
    /// What the risk policy allows, denies, and holds for confirmation.
    #[serde(default)]
    pub policy: Policy,
    /// Which tools are registered.
    #[serde(default)]
    pub tools: Tools,
    /// How history that no longer fits is compacted.
    #[serde(default)]
    pub compaction: Compaction,
    /// What the guardrail refuses.
    #[serde(default)]
    pub guardrail: Guardrail,
    /// How a sandboxed command runs.
    #[serde(default)]
    pub sandbox: SandboxSettings,
    /// How a turn becomes a profile.
    #[serde(default)]
    pub profile: Profile,
    /// JSONL trace recording.
    #[serde(default)]
    pub trace: Trace,
    /// HTTP surface limits.
    #[serde(default)]
    pub http: Http,
    /// MCP servers exposed as tools.
    #[serde(default)]
    pub mcp: Mcp,
    /// Market data: quotes, symbol search, company profiles, metrics, and news.
    #[serde(default)]
    pub market: Market,
    /// Wikipedia lookups.
    #[serde(default)]
    pub wikipedia: Wikipedia,
    /// The store service that holds conversations, memories, the profile graph, and confirmations.
    #[serde(default)]
    pub store: Store,
}

/// Default budgets. Every field comes from the agent section.
impl Default for Budget {
    fn default() -> Self {
        Agent::default().budget()
    }
}

/// Default loop settings. Every field comes from the agent section.
impl Default for AgentConfig {
    fn default() -> Self {
        let agent = Agent::default();
        AgentConfig {
            provider_name: Telemetry::default().provider_name.into(),
            model_name: std::sync::Arc::from(""),
            max_steps: agent.max_steps,
            max_model_retries: agent.max_model_retries,
            tool_timeout: std::time::Duration::from_secs(agent.tool_timeout_secs),
            tool_result_to_file_chars: agent.tool_result_to_file_chars,
            confirmation_ttl: std::time::Duration::from_secs(agent.confirmation_ttl_secs),
            temperature: agent.temperature,
            upload_preview_chars: agent.upload_preview_chars,
            upload_match_chars: agent.upload_match_chars,
            upload_ocr_pages: agent.upload_ocr_pages,
            upload_ocr_lang: agent.upload_ocr_lang.clone(),
            history_turns: agent.history_turns,
            history_keep: Compaction::default().keep_tokens(agent.history_budget_tokens),
            memory_recall_limit: agent.memory_recall_limit,
            recall_in_public: agent.recall_in_public,
            retry_base_ms: agent.retry_base_ms,
            retry_cap_ms: agent.retry_cap_ms,
            max_span_value_chars: agent.max_span_value_chars,
            budget: agent.budget(),
            stream: agent.stream,
            stream_block_chars: agent.stream_block_chars,
            thinking: agent.thinking,
            // The model section has no defaults; these are fixed values.
            max_tokens: 1024,
            max_tokens_without_thinking: 1024,
            usd_per_m_prompt: 0.0,
            usd_per_m_completion: 0.0,
        }
    }
}

/// Why configuration was rejected.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A layer could not be read or a field could not be parsed.
    #[error("config: {0}")]
    Load(String),
    /// The values parsed but cannot work together.
    #[error("config: {0}")]
    Invalid(String),
}

/// Settings that moved to another section. A variable named here fails boot.
const RENAMED: [(&str, &str); 2] = [
    ("BULLPEN_AGENT__TRACE_DIR", "BULLPEN_TRACE__DIR"),
    ("BULLPEN_MODEL__THINKING", "BULLPEN_AGENT__THINKING__MODE"),
];

impl Config {
    /// Fails on a variable that has moved, naming what replaced it.
    pub fn reject_renamed(is_set: impl Fn(&str) -> bool) -> Result<(), ConfigError> {
        match RENAMED.into_iter().find(|(old, _)| is_set(old)) {
            Some((old, new)) => Err(ConfigError::Invalid(format!(
                "{old} moved to {new}; set that instead"
            ))),
            None => Ok(()),
        }
    }

    /// Loads the TOML layer then BULLPEN_* variables, __ separating nesting: BULLPEN_PLATFORM__URL.
    pub fn load() -> Result<Self, ConfigError> {
        Self::reject_renamed(|key| std::env::var_os(key).is_some())?;
        let path =
            std::env::var("BULLPEN_CONFIG_FILE").unwrap_or_else(|_| DEFAULT_CONFIG_FILE.to_owned());
        let cfg: Self = Figment::new()
            .merge(Toml::file(path))
            .merge(Env::prefixed("BULLPEN_").split("__"))
            .extract()
            .map_err(|e| ConfigError::Load(e.to_string()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Rejects combinations that would leave the engine unable to answer.
    pub fn validate(&self) -> Result<(), ConfigError> {
        let invalid = |m: String| Err(ConfigError::Invalid(m));
        if !(-14..=14).contains(&self.prompt.utc_offset_hours) {
            return invalid(format!(
                "prompt.utc_offset_hours must be between -14 and 14, got {}",
                self.prompt.utc_offset_hours
            ));
        }
        validate_agent(&self.agent)?;
        if !(0.0..=1.0).contains(&self.profile.min_confidence) {
            return invalid("profile.min_confidence must be between 0 and 1".into());
        }
        if self.profile.list_limit == 0 {
            return invalid("profile.list_limit must be at least 1".into());
        }
        if self.profile.request_timeout_secs == 0 {
            return invalid("profile.request_timeout_secs must be at least 1".into());
        }
        validate_thinking(&self.agent.thinking)?;
        validate_telemetry(&self.telemetry)?;
        validate_sandbox(&self.sandbox)?;
        validate_model(&self.model)?;
        if self.compaction.enabled && self.compaction.max_tokens == 0 {
            return invalid("compaction.max_tokens must be at least 1".into());
        }
        validate_compaction(&self.compaction, self.agent.history_budget_tokens)?;
        validate_mcp_risks(&self.mcp)?;
        validate_integrations(self)?;
        validate_store(self)?;
        if self.agent.retry_cap_ms < self.agent.retry_base_ms {
            return invalid("agent.retry_cap_ms is below agent.retry_base_ms".into());
        }
        let mut names = std::collections::BTreeSet::new();
        for server in &self.mcp.servers {
            if !names.insert(server.name.as_str()) {
                return invalid(format!(
                    "mcp.servers has two servers named {:?}",
                    server.name
                ));
            }
        }
        // Each section budget fits within the total.
        for (name, value) in [
            ("history", self.agent.history_budget_tokens),
            ("memory", self.agent.memory_budget_tokens),
            ("capabilities", self.agent.capabilities_budget_tokens),
        ] {
            if value > self.agent.prompt_budget_tokens {
                return invalid(format!(
                    "agent.{name}_budget_tokens ({value}) exceeds agent.prompt_budget_tokens ({})",
                    self.agent.prompt_budget_tokens
                ));
            }
        }
        Ok(())
    }

    /// System instructions: prompt.system_file, else prompt.system, else fallback.
    pub fn system_prompt(&self, fallback: &str) -> Result<String, ConfigError> {
        if let Some(path) = self
            .prompt
            .system_file
            .as_deref()
            .map(str::trim)
            .filter(|p| !p.is_empty())
        {
            return std::fs::read_to_string(path)
                .map(|s| s.trim().to_owned())
                .map_err(|e| ConfigError::Invalid(format!("prompt.system_file {path}: {e}")));
        }
        Ok(self
            .prompt
            .system
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(fallback)
            .to_owned())
    }
}

/// Rejects a store section the engine cannot reach, a hosted MCP server it cannot reach, or a
/// confirmation the store does not hold that long.
fn validate_store(cfg: &Config) -> Result<(), ConfigError> {
    let invalid = |m: String| Err(ConfigError::Invalid(m));
    let http = |u: &str| {
        url::Url::parse(u).is_ok() && (u.starts_with("https://") || u.starts_with("http://"))
    };
    let store = &cfg.store;
    if !http(store.url.as_deref().map(str::trim).unwrap_or_default()) {
        return invalid("set BULLPEN_STORE__URL, an http or https URL".into());
    }
    if store
        .token
        .as_ref()
        .is_none_or(|t| t.expose_secret().trim().is_empty())
    {
        return invalid("set BULLPEN_STORE__TOKEN, the store bearer token".into());
    }
    if store.timeout_secs == 0 {
        return invalid("store.timeout_secs must be at least 1".into());
    }
    if !(1..=store.max_ttl_secs).contains(&cfg.agent.confirmation_ttl_secs) {
        return invalid(format!(
            "agent.confirmation_ttl_secs must be from 1 to store.max_ttl_secs ({})",
            store.max_ttl_secs
        ));
    }
    let hosted = &cfg.mcp.hosted;
    if let Some(url) = hosted.endpoint()
        && !http(url)
    {
        return invalid("mcp.hosted.url must be an http or https URL".into());
    }
    if hosted.private.iter().any(|p| p.trim().is_empty()) {
        return invalid(
            "mcp.hosted.private holds an empty prefix, which would match every tool".into(),
        );
    }
    Ok(())
}

/// Rejects a compaction whose kept turns and summary overflow the history budget.
fn validate_compaction(compaction: &Compaction, history: usize) -> Result<(), ConfigError> {
    if !compaction.enabled {
        return Ok(());
    }
    if !(compaction.keep_share > 0.0 && compaction.keep_share < 1.0) {
        return Err(ConfigError::Invalid(
            "compaction.keep_share must be above 0 and below 1".into(),
        ));
    }
    let needed = compaction.keep_tokens(history) + compaction.max_tokens as usize;
    if needed > history {
        return Err(ConfigError::Invalid(format!(
            "a compaction keeps {} tokens of turns and writes a summary of up to {} \
             (compaction.keep_share and compaction.max_tokens), {needed} in all, over \
             agent.history_budget_tokens = {history}; the summary would be trimmed from the prompt",
            compaction.keep_tokens(history),
            compaction.max_tokens
        )));
    }
    Ok(())
}

fn validate_sandbox(sandbox: &SandboxSettings) -> Result<(), ConfigError> {
    if !sandbox.enabled {
        return Ok(());
    }
    if sandbox.runtime.trim().is_empty() {
        return Err(ConfigError::Invalid("sandbox.runtime is empty".into()));
    }
    if sandbox.recent_commands == 0 {
        return Err(ConfigError::Invalid(
            "sandbox.recent_commands is 0, which keeps no record of what the agent ran".into(),
        ));
    }
    if sandbox.max_sessions == 0 {
        return Err(ConfigError::Invalid(
            "sandbox.max_sessions is 0, which lets no caller hold a session".into(),
        ));
    }
    if sandbox.egress {
        for (key, value) in [
            ("sandbox.egress_network", &sandbox.egress_network),
            ("sandbox.egress_proxy_image", &sandbox.egress_proxy_image),
            ("sandbox.egress_proxy_name", &sandbox.egress_proxy_name),
        ] {
            if value.trim().is_empty() {
                return Err(ConfigError::Invalid(format!(
                    "{key} is empty and sandbox.egress is on"
                )));
            }
        }
    }
    Ok(())
}

/// A pinned risk must name a tool the server entry exposes.
fn validate_mcp_risks(mcp: &Mcp) -> Result<(), ConfigError> {
    for server in &mcp.servers {
        if server.tools.is_empty() {
            continue;
        }
        if let Some(name) = server.risks.keys().find(|k| !server.tools.contains(k)) {
            return Err(ConfigError::Invalid(format!(
                "mcp server {} pins a risk for {name}, which is not in its tools",
                server.name
            )));
        }
    }
    Ok(())
}

fn validate_integrations(cfg: &Config) -> Result<(), ConfigError> {
    let market = &cfg.market;
    if market.enabled {
        if !market.base_url.starts_with("https://") {
            return Err(ConfigError::Invalid("market.base_url must be https".into()));
        }
        if market
            .api_key
            .as_ref()
            .is_none_or(|k| k.expose_secret().trim().is_empty())
        {
            return Err(ConfigError::Invalid(
                "market.enabled is on; set BULLPEN_MARKET__API_KEY, or set market.enabled = false"
                    .into(),
            ));
        }
        if market.timeout_secs == 0 || market.max_items == 0 || market.news_days == 0 {
            return Err(ConfigError::Invalid(
                "market: timeout_secs, max_items and news_days must be at least 1".into(),
            ));
        }
    }
    Ok(())
}

/// Rejects telemetry settings that cannot export.
fn validate_telemetry(telemetry: &Telemetry) -> Result<(), ConfigError> {
    let invalid = |m: String| Err(ConfigError::Invalid(m));
    if !(0.0..=1.0).contains(&telemetry.sample_ratio) {
        return invalid(format!(
            "telemetry.sample_ratio must be between 0 and 1, got {}",
            telemetry.sample_ratio
        ));
    }
    if telemetry.provider_name.trim().is_empty() {
        return invalid("telemetry.provider_name is empty".into());
    }
    if telemetry.project_name.trim().is_empty() {
        return invalid("telemetry.project_name is empty".into());
    }
    Ok(())
}
