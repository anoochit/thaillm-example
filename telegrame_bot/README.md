# README

ADK-Rust doesn't have a built-in Telegram integration, but you can build a Telegram bot on top of it by combining `adk-rust` with the `teloxide` crate (the standard Telegram bot library for Rust). Here's a complete guide:

## Edit `.env`

```text
THAILLM_API_KEY=your-api-key-here
TELOXIDE_TOKEN=your_telegram_bot_token
```

Get your bot token from [@BotFather](https://t.me/BotFather) on Telegram

## Run the application

The application supports multiple modes:

### Telegram Bot

```bash
cargo run -- bot
```

### Console CLI

```bash
cargo run -- cli
```

### HTTP Server

```bash
cargo run -- server
```

## How it Works

The architecture is straightforward:

```
Telegram User
     │  (sends message)
     ▼
  teloxide  ──────────────────────────────────▶  Telegram API
     │  (routes update)                              ▲
     ▼                                               │
handle_message()                          bot.send_message()
     │
     ▼
 adk-rust Runner
  (per-user session via InMemorySessionService)
     │
     ▼
  LlmAgent  ──▶  Anthropic / OpenAI / Gemini
```

- **`teloxide`** handles all the Telegram polling and message routing
- **`adk-rust`** handles the AI agent logic and conversation memory
- **`InMemorySessionService`** keyed by `chat_id` gives each user their own conversation history

## Workspace Sandbox

The bot includes a filesystem tool that operates within a "workspace" directory. By default, this is located at:

- **`~/workspace`** (in your home directory)

The bot will automatically create this directory if it doesn't exist. Files created by the agent will be stored here.

## Key Tips

**Switch LLM providers** — just change the client configuration in `src/agent.rs`. For example, using ThaiLLM:

```rust
let config = OpenAIConfig::compatible(
    &api_key,
    "https://thaillm.or.th/api/v1",
    "typhoon-s-thaillm-8b-instruct",
);
let model = OpenAIClient::new(config)?;
```

**Add tools** — attach ADK function tools to your agent in `src/agent.rs`:

```rust
let mut builder = LlmAgentBuilder::new("agent")
    .model(Arc::new(model));

for t in my_custom_tools() {
    builder = builder.tool(t).into();
}
let agent = builder.build()?;
```

**Persistent sessions** — replace `InMemorySessionService` with `SqliteSessionService` from `adk-session` so conversations survive restarts:

```toml
adk-session = { version = "0.7.0", features = ["sqlite"] }
```

**Webhooks vs polling** — `teloxide` defaults to long polling (easiest for dev). For production, switch to webhooks using `teloxide`'s `axum_no_setup` feature.
