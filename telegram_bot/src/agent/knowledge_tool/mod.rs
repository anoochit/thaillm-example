use std::sync::Arc;
use adk_rust::serde::Deserialize;
use adk_tool::{tool, AdkError};
use adk_rust::Tool;
use schemars::JsonSchema;
use serde_json::{json, Value};
use crate::agent::utils::get_workspace_root;
use tokio::fs;

#[derive(Deserialize, JsonSchema)]
struct KnowledgeArgs {
    /// The query to search for in local documentation
    query: String,
}

/// Searches local documentation for information.
#[tool]
async fn search_knowledge(args: KnowledgeArgs) -> std::result::Result<Value, AdkError> {
    let workspace_root = get_workspace_root()
        .await
        .map_err(|e| AdkError::tool(format!("Failed to get workspace root: {}", e)))?;
    let memory_dir = workspace_root.join("knowledge");

    if !memory_dir.exists() {
        return Ok(json!({"message": "knowledge directory not found."}));
    }

    let mut read_dir = fs::read_dir(memory_dir)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to read knowledge directory: {}", e)))?;

    let mut found = false;
    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(&path)
                .await
                .unwrap_or_default();
            
            if content.contains(&args.query) {
                found = true;
                break;
            }
        }
    }
        
    if found {
        Ok(json!({"message": format!("Found '{}' in knowledge.", args.query)}))
    } else {
        Ok(json!({"message": "Information not found in knowledge."}))
    }
}

pub fn knowledge_tools() -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(SearchKnowledge)]
}
