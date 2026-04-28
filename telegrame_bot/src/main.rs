mod agent;
mod bot;
mod runner;

use std::sync::Arc;
use adk_session::InMemorySessionService;
use runner::AgentRunner;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    let agent = agent::build_agent()?;
    let sessions = Arc::new(InMemorySessionService::new());

    let runner = Arc::new(
        AgentRunner::new(agent, sessions.clone(), "telegram")
    );

    bot::run_bot(runner, sessions).await?;

    Ok(())
}