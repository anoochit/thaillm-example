mod agent;
mod bot;
mod runner;
mod cli;
mod serve;

use std::sync::Arc;

use adk_session::InMemorySessionService;
use clap::{Parser, Subcommand};
use runner::AgentRunner;

#[derive(Parser)]
#[command(name = "agent-app")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Bot,
    Cli,
    Server,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    // pretty_env_logger::init();

    let cli = Cli::parse();

    // shared setup
    let agent = agent::build_agent()?;
    let sessions = Arc::new(InMemorySessionService::new());

    match cli.command {
        Commands::Bot => {
            let runner = Arc::new(
                AgentRunner::new(agent, sessions.clone(), "app")
            );
            bot::run_bot(runner, sessions.clone()).await?;
        }
        Commands::Cli => {
            cli::run_cli(agent).await?;
        }
        Commands::Server => {
            serve::run_serve(agent).await?;
        }
    }

    Ok(())
}

