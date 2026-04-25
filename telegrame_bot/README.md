## README

ADK-Rust doesn't have a built-in Telegram integration, but you can build a Telegram bot on top of it by combining `adk-rust` with the `teloxide` crate (the standard Telegram bot library for Rust). Here's a complete guide:

## Run the bot

```bash
cargo run
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

## Key Tips

**Switch LLM providers** — just change `adk-model` feature and the client:
```toml
adk-rust = { version = "0.6.0", features = ["openai"] }
```
```rust
let model = OpenAIClient::new(OpenAIConfig::new(api_key, "gpt-4o-mini"))?;
```

**Add tools** — attach ADK function tools to your agent:
```rust
let agent = LlmAgentBuilder::new("bot")
    .tool(my_custom_tool())
    .model(Arc::new(model))
    .build()?;
```

**Persistent sessions** — replace `InMemorySessionService` with `SqliteSessionService` from `adk-session` so conversations survive restarts:
```toml
adk-session = { version = "0.6.0", features = ["sqlite"] }
```

**Webhooks vs polling** — `teloxide` defaults to long polling (easiest for dev). For production, switch to webhooks using `teloxide`'s `axum_no_setup` feature.

