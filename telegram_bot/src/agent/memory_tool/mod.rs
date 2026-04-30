use std::sync::Arc;
use adk_rust::serde::Deserialize;
use adk_tool::{tool, AdkError};
use adk_rust::Tool;
use schemars::JsonSchema;
use serde_json::{json, Value};
use tokio::fs;

use crate::agent::utils::get_workspace_root;

#[derive(Deserialize, JsonSchema)]
struct MemoryArgs {
    /// Action to perform: "append" or "read"
    action: String,
    /// Content to log, only used if action is "append"
    #[serde(default)]
    log: String,
}

/// Manages session logs and temporary state.
#[tool]
async fn manage_memory(args: MemoryArgs) -> std::result::Result<Value, AdkError> {
    let root = get_workspace_root().await?;
    let path = root.join("session.log");
    
    match args.action.as_str() {
        "append" => {
            let mut content = fs::read_to_string(&path).await.unwrap_or_default();
            content.push_str(&args.log);
            content.push('\n');
            fs::write(&path, content).await.map_err(|e| AdkError::tool(e.to_string()))?;
            Ok(json!({"message": "Log appended."}))
        }
        "read" => {
            let content = fs::read_to_string(&path).await.map_err(|e| AdkError::tool(e.to_string()))?;
            Ok(json!({"content": content}))
        }
        _ => Err(AdkError::tool("Unknown memory action".to_string())),
    }
}

pub fn memory_tools() -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(ManageMemory)]
}
