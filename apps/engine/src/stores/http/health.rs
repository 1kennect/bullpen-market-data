//! Readiness of the store.

use async_trait::async_trait;

use super::client::StoreClient;
use crate::core::traits::health::Probe;

/// Answers readiness from the store health route.
pub struct StoreProbe {
    client: StoreClient,
}

impl StoreProbe {
    /// Wraps a client.
    pub fn new(client: StoreClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl Probe for StoreProbe {
    fn name(&self) -> &'static str {
        "store"
    }

    async fn ready(&self) -> bool {
        self.client.ready().await
    }
}
