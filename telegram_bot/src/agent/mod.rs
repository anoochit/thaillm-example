use std::sync::Arc;
use adk_rust::prelude::*;
// use adk_rust::model::{OpenAIClient, OpenAIConfig};

pub mod utils;
pub mod database;
pub mod current_datetime_tool;
pub mod filesystem_tool;
pub mod km_tool;
pub mod shell_tool;
pub mod weather_tool;

pub async fn build_agent() -> anyhow::Result<Arc<dyn Agent>> {

    // Sample for ThaiLLM OpenAI-compatible API
    // Load the API key from an environment variable
    // let api_key = std::env::var("THAILLM_API_KEY")?;

    // Create the OpenAI client with the custom configuration
    // let config = OpenAIConfig::compatible(
    //     &api_key,
    //     "https://thaillm.or.th/api/v1",
    //     "typhoon-s-thaillm-8b-instruct",
    // );

    // Create the OpenAI client with the custom configuration
    // let model = OpenAIClient::new(config)?;

    // Sample for Gemini
    let api_key = std::env::var("GOOGLE_API_KEY")?;
    let model = GeminiModel::new(&api_key, "gemini-2.5-flash")?;

    // Build the agent with the model and tools
    let mut builder = LlmAgentBuilder::new("agent")
        .description("A helpful AI assistant")
        .instruction("You are a professional, highly capable, and secure AI agent assistant.
Your core persona is collaborative, proactive, and precise. 
When interacting:
1. Always prioritize checking internal knowledge (memory and local docs) and local skills first. Only use external internet searches if you cannot find the answer locally.
2. Always prioritize tool usage for file operations, system tasks, or memory retrieval.
3. Be concise but thorough in technical responses.
4. If an action is requested that you cannot safely perform or do not have tools for, explicitly state the limitation.
5. If you are unsure of the answer, do not hallucinate; explain what you do know and where the ambiguity lies.
6. Adhere to security best practices; never expose or log sensitive environment data or credentials.
7. Whenever you need to present lists or structured data, use a clear Markdown table format if appropriate for readability.
8. Maintain a compact, high-level summary of the conversation thread. Frequently synthesize past interactions into concise summaries to ensure the context window remains efficient for long-running sessions.")
        .model(Arc::new(model))
        .with_skills_from_root("./skills")?;

    // add tools to the agent
    let mut tools = weather_tool::weather_tools();
    tools.extend(filesystem_tool::filesystem_tools());
    tools.extend(current_datetime_tool::datetime_tools());
    tools.extend(km_tool::km_tools());
    tools.extend(shell_tool::shell_tools());

    for t in tools {
        builder = builder.tool(t).into();
    }

    // Build and return the agent
    Ok(Arc::new(builder.build()?))
}
