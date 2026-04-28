use std::sync::Arc;
use adk_rust::prelude::*;
use adk_rust::model::{OpenAIClient, OpenAIConfig};

mod weather_tool;
mod filesystem_tool;

pub fn build_agent() -> anyhow::Result<Arc<dyn Agent>> {
    let api_key = std::env::var("THAILLM_API_KEY")?;

    let config = OpenAIConfig::compatible(
        &api_key,
        "https://thaillm.or.th/api/v1",
        "typhoon-s-thaillm-8b-instruct",
    );

    let model = OpenAIClient::new(config)?;

    let mut builder = LlmAgentBuilder::new("agent")
        .description("A helpful AI assistant")
        .instruction("You are a friendly assistant. Be concise and helpful. Answer in User language.")
        .model(Arc::new(model));

    for t in weather_tool::weather_tools() {
        builder = builder.tool(t).into();
    }

    for t in filesystem_tool::filesystem_tools() {
        builder = builder.tool(t).into();
    }

    Ok(Arc::new(builder.build()?))
}