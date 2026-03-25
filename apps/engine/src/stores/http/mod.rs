//! Adapters over the store HTTP API: every store the engine needs, behind the same traits.
//!
//! Every path carries the tenant_id and user_id of the request.

mod client;
mod confirmation;
mod conversation;
mod health;
mod memory;
mod profile;

pub use client::StoreClient;
pub use confirmation::HttpConfirmations;
pub use conversation::HttpConversations;
pub use health::StoreProbe;
pub use memory::HttpMemory;
pub use profile::HttpProfileGraph;
