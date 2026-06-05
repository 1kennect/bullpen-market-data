//! Store composition: every store over the store HTTP API.

use std::sync::Arc;
use std::time::Duration;

use super::Stores;
use crate::core::config::Config;
use crate::core::traits::memory::profile::ProfileGraph;
use crate::stores::http::{
    HttpConfirmations, HttpConversations, HttpMemory, HttpProfileGraph, StoreClient, StoreProbe,
};

/// Every store over the store service. Config::validate guarantees the url and the token.
pub(super) fn stores(cfg: &Config) -> anyhow::Result<Stores> {
    let (Some(url), Some(token)) = (cfg.store.url.as_deref(), cfg.store.token.clone()) else {
        anyhow::bail!("set BULLPEN_STORE__URL and BULLPEN_STORE__TOKEN");
    };
    let client = StoreClient::new(url, token, Duration::from_secs(cfg.store.timeout_secs))
        .map_err(|e| anyhow::anyhow!("store client: {e}"))?;
    let profile_graph = cfg
        .profile
        .enabled
        .then(|| Arc::new(HttpProfileGraph::new(client.clone())) as Arc<dyn ProfileGraph>);
    tracing::info!(url, "stores are over the store service");
    Ok(Stores {
        conversations: Arc::new(HttpConversations::new(client.clone())),
        memory: Arc::new(HttpMemory::new(client.clone())),
        confirmations: Arc::new(HttpConfirmations::new(client.clone())),
        profile_graph,
        probes: vec![Arc::new(StoreProbe::new(client))],
    })
}
