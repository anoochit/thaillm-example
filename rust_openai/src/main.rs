use adk_rust::prelude::*;
use adk_rust::Launcher;
use std::sync::Arc;

use adk_rust::model::OpenAIConfig;
use adk_rust::model::OpenAIClient;

mod filesystem_tool;
mod weather_tool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let api_key = std::env::var("THAILLM_API_KEY")?;
    let config = OpenAIConfig::compatible(
        &api_key,
        "https://thaillm.or.th/api/v1",
        "typhoon-s-thaillm-8b-instruct"
    );

    let model = OpenAIClient::new(config)?;

    let mut builder = LlmAgentBuilder::new("rust_openai")
        .description("A helpful AI assistant")
        .instruction("You are a friendly assistant. Be concise and helpful.")
        .model(Arc::new(model));

    
    for t in filesystem_tool::filesystem_tools() {
        builder = builder.tool(t).into();
    }

    for t in weather_tool::weather_tools() {
        builder = builder.tool(t).into();
    }

    let agent = builder.build()?;

    Launcher::new(Arc::new(agent)).run().await?;
    Ok(())
}
