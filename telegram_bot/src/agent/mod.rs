use std::sync::Arc;
use adk_rust::prelude::*;
use adk_rust::model::{OpenAIClient, OpenAIConfig};

pub mod utils;
pub mod datetime_tool;
pub mod filesystem_tool;
pub mod knowledge_tool;
pub mod memory_tool;
pub mod shell_tool;
pub mod weather_tool;

pub fn build_agent() -> anyhow::Result<Arc<dyn Agent>> {

    // Sample for ThaiLLM OpenAI-compatible API
    // Load the API key from an environment variable
    let api_key = std::env::var("THAILLM_API_KEY")?;

    // Create the OpenAI client with the custom configuration
    let config = OpenAIConfig::compatible(
        &api_key,
        "https://thaillm.or.th/api/v1",
        "typhoon-s-thaillm-8b-instruct",
    );

    // Create the OpenAI client with the custom configuration
    let model = OpenAIClient::new(config)?;

    // Sample for Gemini
    // let api_key = std::env::var("GOOGLE_API_KEY")?;
    // let model = GeminiModel::new(&api_key, "gemini-2.5-flash")?;

    // Build the agent with the model and tools
    let mut builder = LlmAgentBuilder::new("agent")
        .description("A helpful AI assistant")
        .instruction("You are a friendly assistant. Be concise and helpful. 
        Always use the tools when relevant. If you don't know the answer, say you don't know instead of making something up.")
        .model(Arc::new(model));

    // add tools to the agent
    let mut tools = weather_tool::weather_tools();
    tools.extend(filesystem_tool::filesystem_tools());
    tools.extend(datetime_tool::datetime_tools());
    tools.extend(knowledge_tool::knowledge_tools());
    tools.extend(memory_tool::memory_tools());
    tools.extend(shell_tool::shell_tools());

    for t in tools {
        builder = builder.tool(t).into();
    }

    // Build and return the agent
    Ok(Arc::new(builder.build()?))
}
