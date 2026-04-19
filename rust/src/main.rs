use adk_rust::prelude::*;
use adk_rust::Launcher;
use std::sync::Arc;
mod thaillm;

mod filesystem_tool;
mod weather_tool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let api_key = std::env::var("THAILLM_API_KEY")?;

    let model = thaillm::ThaiLLMModel::new(&api_key, "typhoon");

    let mut builder = LlmAgentBuilder::new("agent-thaillm")
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
