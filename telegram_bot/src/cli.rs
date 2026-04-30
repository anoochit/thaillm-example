use futures::StreamExt;
use std::io::{self, Write};
use std::sync::Arc;

use adk_runner::EventsCompactionConfig;
use adk_rust::Agent;
use adk_rust::agent::LlmEventSummarizer;
use adk_rust::prelude::*;
use adk_session::{CreateRequest, GetRequest, SessionService};

pub(crate) async fn run_cli(
    agent: Arc<dyn Agent>,
    sessions: Arc<dyn SessionService>,
    model: Arc<dyn Llm>,
) -> anyhow::Result<()> {
    println!(
        r#"
Type a message to chat. /exit to quit.
"#
    );

    let app_name = "cli";
    let user_id = "default_user";
    let session_id = "cli_session";

    // ensure session exists
    if sessions
        .get(GetRequest {
            app_name: app_name.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            num_recent_events: None,
            after: None,
        })
        .await
        .is_err()
    {
        sessions
            .create(CreateRequest {
                app_name: app_name.to_string(),
                user_id: user_id.to_string(),
                session_id: Some(session_id.to_string()),
                state: Default::default(),
            })
            .await?;
    }

    let summarizer = Arc::new(LlmEventSummarizer::new(model.clone()));
    let compaction_config = EventsCompactionConfig {
        compaction_interval: 10,
        overlap_size: 2,
        summarizer,
    };

    let runner = Runner::builder()
        .app_name(app_name)
        .agent(agent)
        .session_service(sessions)
        .compaction_config(compaction_config)
        .build()?;

    let mut input = String::new();
    loop {
        print!("You> ");
        io::stdout().flush()?;

        input.clear();
        let bytes_read = io::stdin().read_line(&mut input)?;
        if bytes_read == 0 {
            break;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "/exit" || trimmed == "exit" {
            break;
        }

        let content = Content::new("user").with_text(trimmed);
        let mut stream = runner.run_str(user_id, session_id, content).await?;

        print!("");
        io::stdout().flush()?;

        while let Some(result) = stream.next().await {
            let event = result?;

            if let Some(content) = &event.llm_response.content {
                for part in &content.parts {
                    if let Some(text) = part.text() {
                        print!("{}", text);
                        io::stdout().flush()?;
                    }
                }
            }
        }
        println!("\n");
    }

    Ok(())
}
