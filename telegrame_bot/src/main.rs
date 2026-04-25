use adk_rust::prelude::*;
use std::sync::Arc;
use futures::StreamExt;

use adk_rust::model::{OpenAIClient,OpenAIConfig};

use adk_session::{CreateRequest, DeleteRequest, GetRequest, InMemorySessionService, SessionService};
use teloxide::{prelude::*, utils::command::BotCommands};

mod weather_tool;
mod filesystem_tool;


#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
enum Command {
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Show help")]
    Help,
    #[command(description = "Clear chat history")]
    Clear,
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();
    
    let api_key = std::env::var("THAILLM_API_KEY")?;

    let config = OpenAIConfig::compatible(
        &api_key,
        "https://thaillm.or.th/api/v1",
        "typhoon-s-thaillm-8b-instruct"
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

    let agent: Arc<dyn Agent> = Arc::new(builder.build()?);


    // Session service for per-user conversation history
    let session_service = Arc::new(InMemorySessionService::new());

    // --- Start Telegram bot ---
    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(Update::filter_message().filter_command::<Command>().endpoint(
            |bot: Bot, msg: Message, cmd: Command, agent: Arc<dyn Agent>, sessions: Arc<InMemorySessionService>| async move {
                handle_command(bot, msg, cmd, agent, sessions).await
            },
        ))
        .branch(Update::filter_message().endpoint(
            |bot: Bot, msg: Message, agent: Arc<dyn Agent>, sessions: Arc<InMemorySessionService>| async move {
                handle_message(bot, msg, agent, sessions).await
            },
        ));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![agent, session_service])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    
    Ok(())
}


async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    _agent: Arc<dyn Agent>,
    sessions: Arc<InMemorySessionService>,
) -> anyhow::Result<()> {
    let chat_id = msg.chat.id.to_string();

    match cmd {
        Command::Start | Command::Help => {
            bot.send_message(
                msg.chat.id,
                "👋 สวัสดี! ฉันคือผู้ช่วย AI ที่ขับเคลื่อนด้วย ADK-Rust ส่งข้อความมาคุยกันได้เลย!",
            )
            .await?;
        }
        Command::Clear => {
            sessions.delete(DeleteRequest { app_name: "telegram".to_string(), user_id: chat_id.clone(), session_id: chat_id.clone() }).await?;
            bot.send_message(msg.chat.id, "✅ Conversation cleared!").await?;
        }
    }
    Ok(())
}

async fn handle_message(
    bot: Bot,
    msg: Message,
    agent: Arc<dyn Agent>,
    sessions: Arc<InMemorySessionService>,
) -> anyhow::Result<()> {
    let Some(text) = msg.text() else { return Ok(()) };
    let chat_id = msg.chat.id.to_string();

    // Show "typing..." indicator
    bot.send_chat_action(msg.chat.id, teloxide::types::ChatAction::Typing)
        .await?;

    // Ensure session exists — Runner does a plain get() and errors if missing
    let get_result = sessions.get(GetRequest {
        app_name: "telegram".to_string(),
        user_id: chat_id.clone(),
        session_id: chat_id.clone(),
        num_recent_events: None,
        after: None,
    }).await;
    if get_result.is_err() {
        sessions.create(CreateRequest {
            app_name: "telegram".to_string(),
            user_id: chat_id.clone(),
            session_id: Some(chat_id.clone()),
            state: Default::default(),
        }).await?;
    }

    // Run the agent
    let runner = Runner::builder()
        .app_name("telegram")
        .agent(agent)
        .session_service(sessions)
        .build()?;

    let content = Content::new("user").with_text(text);

    let mut response_text = String::new();
    let mut stream = runner
        .run_str(&chat_id, &chat_id, content)
        .await?;

    while let Some(result) = stream.next().await {
        let event = result?;
        if let Some(content) = &event.llm_response.content {
            for part in &content.parts {
                if let Some(text) = part.text() {
                    response_text.push_str(text);
                }
            }
        }
    }

    if response_text.is_empty() {
        response_text = "Sorry, I couldn't generate a response.".to_string();
    }

    bot.send_message(msg.chat.id, response_text).await?;
    Ok(())

}