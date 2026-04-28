use std::sync::Arc;

use adk_rust::Agent;
use adk_rust::Launcher;

pub(crate) async fn run_cli(agent: Arc<dyn Agent>) -> anyhow::Result<()> {
    Launcher::new(agent).run_console_directly().await?;

    Ok(())
}
