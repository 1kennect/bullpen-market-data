//! Built-in tools and MCP-backed tools. Each declares a RiskClass.

pub mod files;
pub mod http;
pub mod market;
pub mod mcp;
pub mod sandbox;
pub mod wiki;

use serde_json::{Value, json};

use crate::core::types::tools::ToolError;

/// Name of the query parameter of a tool taking one text query.
const QUERY: &str = "query";

/// The structured payload a tool hands back beside its text; unserializable values log, yield None.
pub fn structured<T: serde::Serialize>(value: &T) -> Option<serde_json::Value> {
    match serde_json::to_value(value) {
        Ok(value) => Some(value),
        Err(error) => {
            tracing::error!(error = %error, "tool payload would not serialize");
            None
        }
    }
}

/// The query a search call carries, trimmed. Err is shown to the model.
pub fn query_arg(args: &Value) -> Result<String, ToolError> {
    match args.get(QUERY) {
        Some(Value::String(text)) if !text.trim().is_empty() => Ok(text.trim().to_owned()),
        _ => Err(ToolError::InvalidArguments(format!(
            "{QUERY} is required: the words to search for"
        ))),
    }
}

/// The JSON schema of a tool taking one required text query.
pub fn query_schema(description: &str) -> Value {
    json!({
        "type": "object",
        "properties": { QUERY: { "type": "string", "description": description } },
        "required": [QUERY],
    })
}
