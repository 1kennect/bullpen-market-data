//! Conversation and message wire shapes.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Who can see a conversation: a direct message is private, a guild thread is public.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// Seen by the guild.
    Public,
    /// Seen by the user alone.
    Private,
}

impl Visibility {
    /// Column value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
        }
    }
}

/// The role of an appended message. Summaries go through their own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System turn.
    System,
    /// User turn.
    User,
    /// Assistant turn.
    Assistant,
    /// Tool turn.
    Tool,
}

impl Role {
    /// Column value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }
}

/// Column value of a summary row.
pub const SUMMARY_ROLE: &str = "summary";

/// The conversation id in the path.
#[derive(Debug, Deserialize)]
pub struct ConversationPath {
    /// Conversation id.
    pub conversation: Uuid,
}

/// The channel id in the path.
#[derive(Debug, Deserialize)]
pub struct ChannelPath {
    /// Channel id.
    pub channel: String,
}

/// PUT /conversations/{id}: the channel and visibility of a new conversation.
#[derive(Debug, Deserialize)]
pub struct EnsureBody {
    /// Channel the conversation lives in.
    pub channel_id: String,
    /// Visibility of the conversation.
    pub visibility: Visibility,
}

/// A read at one visibility.
#[derive(Debug, Deserialize)]
pub struct VisibilityQuery {
    /// Visibility asked about.
    pub visibility: Visibility,
}

/// One message to append.
#[derive(Debug, Deserialize)]
pub struct NewMessage {
    /// Row role.
    pub role: Role,
    /// The whole message as the engine serializes it.
    pub content: Value,
}

/// POST /conversations/{id}/messages.
#[derive(Debug, Deserialize)]
pub struct AppendBody {
    /// Messages in order.
    pub messages: Vec<NewMessage>,
}

/// POST /conversations/{id}/summary.
#[derive(Debug, Deserialize)]
pub struct SummaryBody {
    /// The summary message as the engine serializes it.
    pub content: Value,
    /// Position of the last message the summary stands in for.
    pub covers: i64,
}

/// One stored message as GET /conversations/{id}/messages returns it.
#[derive(Debug, Serialize)]
pub struct StoredMessage {
    /// Position in the conversation, from 1.
    pub position: i64,
    /// Row role.
    pub role: String,
    /// The message as the engine sent it.
    pub content: Value,
}
