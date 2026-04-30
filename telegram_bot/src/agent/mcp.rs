use adk_rust::prelude::*;
use adk_tool::mcp::McpServerManager;
use std::sync::Arc;

/// Loads MCP tools from `mcp.json` if it exists and attaches them to the agent builder.
pub async fn load_mcp_tools(mut builder: LlmAgentBuilder) -> anyhow::Result<LlmAgentBuilder> {
    if std::path::Path::new("mcp.json").exists() {
        let mcp_manager = McpServerManager::from_json_file("mcp.json")?;
        let mcp_manager = Arc::new(mcp_manager);

        // Start all servers and handle potential failures per server
        let results = mcp_manager.start_all().await;
        for (name, res) in results {
            if let Err(e) = res {
                log::error!("Failed to start MCP server '{}': {}", name, e);
            } else {
                log::info!("Started MCP server '{}'", name);
            }
        }

        builder = builder.toolset(mcp_manager).into();
    }

    Ok(builder)
}
