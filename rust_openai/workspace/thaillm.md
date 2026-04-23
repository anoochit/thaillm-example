# ThaiLLM Agent in Rust

This tutorial shows how to create a simple agent using the ThaiLLM model in Rust. ThaiLLM is a pre-trained language model optimized for Thai language tasks.

## Prerequisites
- Rust compiler (e.g., `rustc`)
- Cargo (Rust package manager)
- `tokio` for async I/O
- `reqwest` for HTTP requests
- `serde` for serialization
- `serde_json` for JSON handling

## Step 1: Install Dependencies
Create a new Rust project:
```bash
cargo new thaillm_agent
cd thaillm_agent
```

Add the following dependencies to `Cargo.toml`:
```toml
[dependencies]
tokio = { version = "1.0", features = ["rt", "rt-multi-thread"] }
reqwest = { version = "0.11", features = ["json", "blocking"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = { version = "1.0" }
```

## Step 2: Create the Agent
Create a file `src/main.rs` with the following code:

```rust
use tokio::task;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
struct ResponseData {
    text: String,
    tokens: usize,
}

async fn ask_question(question: &str) -> Result<ResponseData, reqwest::Error> {
    let client = Client::new();
    let url = "https://api.thaillm.com/v1/chat/completions";

    let body = serde_json::json!({
        "model": "thaillm",
        "messages": [
            {
                "role": "user",
                "content": question,
            },
        ],
        "max_tokens": 100,
    });

    let response = client.post(url)
        .json(&body)
        .send()
        .await?
        .json::<Value>()
        .await?;

    let text = response["choices"][0]["message"]["content"]["string"];
    let tokens = response["choices"][0]["message"]["content"]["tokens"]["number"];

    Ok(ResponseData { text, tokens })
}

#[tokio::main]
async fn main() {
    let question = "สวัสดี ฉันอยากเรียนรู้เกี่ยวกับ Rust ค่ะ";
    match ask_question(question).await {
        Ok(result) => {
            println!{"Text: {}", result.text};
            println!{"Tokens: {}", result.tokens};
        },
        Err(e) => {
            eprintln!{"Error: {}", e};
        },
    }
}
```

## Step 3: Run the Agent
Make sure you have an API key for ThaiLLM (replace `your_api_key`):
```bash
export THAILLM_API_KEY=your_api_key
```

Run the agent:
```bash
cargo run
```

## Conclusion
You've successfully created a simple agent in Rust using the ThaiLLM model. The agent can ask questions and receive responses in Thai. You can extend this to build more complex applications like chatbots or automated assistants.

---
*Note: This is a simplified example. For production use, consider adding error handling, rate limiting, and authentication.*