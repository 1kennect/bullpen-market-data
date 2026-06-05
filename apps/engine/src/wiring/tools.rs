//! The tool set: market data, sandbox, MCP servers, and the enabled integrations.

use std::sync::Arc;
use std::time::Duration;

use secrecy::ExposeSecret;

use crate::core::config::Config;
use crate::core::traits::tools::Tool;
use crate::core::types::tools::sandbox::Limits as SandboxLimits;
use crate::runtime::harness::tools::ToolSet;
use crate::runtime::tools::mcp::{self, McpLimits};
use crate::runtime::tools::sandbox::{
    ContainerSandbox, SandboxTool, Wording as SandboxWording, reap_sessions,
};
use crate::runtime::tools::{market, wiki};

/// Builds the tools of one integration, or says why it could not.
type Build<'a> = Box<dyn FnOnce() -> Result<Vec<Arc<dyn Tool>>, String> + 'a>;

/// One integration: its name, whether it is enabled, and how its tools are built.
type Integration<'a> = (&'static str, bool, Build<'a>);

/// Every tool the model may call, with tools.disabled removed at registration.
pub(crate) async fn build_tools(
    cfg: &Config,
    sandbox: Option<Arc<ContainerSandbox>>,
) -> anyhow::Result<(ToolSet, Vec<String>)> {
    let disabled = |name: &str| cfg.tools.disabled.iter().any(|d| d == name);
    let mut tools = ToolSet::new();
    let mut mcp_names = Vec::new();
    if let Some(sandbox) = sandbox {
        let tool: Arc<dyn Tool> = Arc::new(SandboxTool::new(
            sandbox,
            cfg.sandbox.risk,
            SandboxWording::from(&cfg.sandbox),
        ));
        if !disabled(&tool.definition().name) {
            tracing::info!(
                runtime = %cfg.sandbox.runtime,
                image = %cfg.sandbox.image,
                "sandbox registered"
            );
            tools = tools.with(tool);
        }
    }
    for server in cfg.mcp.resolved_servers() {
        let limits = McpLimits {
            required_props_only: server
                .required_props_only
                .unwrap_or(cfg.mcp.required_props_only),
            tool_timeout_secs: server.tool_timeout_secs,
            ..McpLimits::from(&cfg.mcp)
        };
        let remote = mcp::connect(&server.url, &server.tools, &server.risks, &limits)
            .await
            .map_err(|e| anyhow::anyhow!("mcp server {} at {}: {e}", server.name, server.url))?;
        let mut registered = 0;
        for tool in remote {
            if disabled(&tool.definition().name) {
                continue;
            }
            mcp_names.push(tool.definition().name);
            tools = tools.with(tool);
            registered += 1;
        }
        tracing::info!(
            server = %server.name,
            url = %server.url,
            count = registered,
            "mcp tools registered"
        );
    }
    if let Some(url) = cfg.mcp.hosted.endpoint() {
        let token = cfg
            .mcp
            .hosted
            .token
            .as_ref()
            .map(ExposeSecret::expose_secret);
        let remote = mcp::connect_hosted(
            url,
            token,
            &cfg.mcp.hosted.tools,
            &cfg.mcp.hosted.private,
            &McpLimits::from(&cfg.mcp),
        )
        .await
        .map_err(|e| anyhow::anyhow!("hosted mcp at {url}: {e}"))?;
        let mut registered = 0;
        for tool in remote {
            if disabled(&tool.definition().name) {
                continue;
            }
            mcp_names.push(tool.definition().name);
            tools = tools.with(tool);
            registered += 1;
        }
        tracing::info!(url = %url, count = registered, "hosted mcp tools registered");
    }
    for (integration, enabled, build) in integrations(cfg) {
        if !enabled {
            continue;
        }
        let built = build().map_err(|e| anyhow::anyhow!("{integration}: {e}"))?;
        tools = register(tools, built, &disabled, integration);
    }
    tracing::info!(tools = ?tools, "tool set");
    Ok((tools, mcp_names))
}

/// Every integration that adds tools, in registration order.
fn integrations(cfg: &Config) -> Vec<Integration<'_>> {
    vec![
        (
            "market",
            cfg.market.enabled,
            Box::new(move || market::tools(&cfg.market).map_err(|e| e.to_string())),
        ),
        (
            "wikipedia",
            cfg.wikipedia.enabled,
            Box::new(move || wiki::tools(&cfg.wikipedia)),
        ),
    ]
}

/// Adds each built tool that is not disabled to the set, logging how many an integration added.
fn register(
    mut tools: ToolSet,
    built: Vec<Arc<dyn Tool>>,
    disabled: &dyn Fn(&str) -> bool,
    integration: &str,
) -> ToolSet {
    let mut count = 0;
    for tool in built {
        if disabled(&tool.definition().name) {
            continue;
        }
        tools = tools.with(tool);
        count += 1;
    }
    tracing::info!(integration, count, "tools registered");
    tools
}

/// The sandbox, when it is enabled and its runtime answers. Starts the session sweeper.
pub(super) async fn sandbox(cfg: &Config) -> anyhow::Result<Option<Arc<ContainerSandbox>>> {
    if !cfg.sandbox.enabled {
        return Ok(None);
    }
    let sandbox = ContainerSandbox::new(SandboxLimits::from(&cfg.sandbox));
    if let Err(error) = sandbox.probe().await {
        if cfg.sandbox.required {
            anyhow::bail!(
                "sandbox.enabled is on but the container runtime is unreachable: {error}. \
                 Give the engine a runtime, or set sandbox.required = false to run without it."
            );
        }
        tracing::warn!(%error, "the container runtime is unreachable; run_sandbox is not offered");
        return Ok(None);
    }
    // The sweep interval is the idle budget: a session lives at most twice it.
    let every = Duration::from_secs(cfg.sandbox.session_idle_secs.max(1));
    // Session containers a previous engine left running are nobody's to reap but this one's.
    sandbox.remove_orphans().await;
    tokio::spawn(reap_sessions(sandbox.clone(), every));
    Ok(Some(Arc::new(sandbox)))
}
