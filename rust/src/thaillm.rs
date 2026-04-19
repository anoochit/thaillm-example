use adk_core::{
    AdkError, Content, FinishReason, Llm, LlmRequest, LlmResponse, LlmResponseStream, Part,
    UsageMetadata,
};
use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};

// ─── Request / Response types (OpenAI-compatible) ────────────────────────────

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChatMessage,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

// ─── Streaming types ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: Delta,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct Delta {
    #[serde(default)]
    content: Option<String>,
}

// ─── ThaiLLMModel ─────────────────────────────────────────────────────────────

/// A model provider for the ThaiLLM service (http://thaillm.or.th).
///
/// Supported model names: `openthaigpt`, `pathumma`, `typhoon`, `kbtg`.
///
/// # Example
/// ```rust
/// let model = ThaiLLMModel::new("YOUR_API_KEY", "openthaigpt");
/// ```
pub struct ThaiLLMModel {
    api_key: String,
    model_name: String,
    base_url: String,
    client: Client,
    max_tokens: u32,
    temperature: f32,
}

impl ThaiLLMModel {
    /// Create a new ThaiLLMModel.
    ///
    /// * `api_key`    – the `apikey` header value
    /// * `model_name` – one of `openthaigpt`, `pathumma`, `typhoon`, `kbtg`
    pub fn new(api_key: &str, model_name: &str) -> Self {
        let base_url = format!(
            "http://thaillm.or.th/api/{}/v1/chat/completions",
            model_name
        );
        Self {
            api_key: api_key.to_string(),
            model_name: model_name.to_string(),
            base_url,
            client: Client::new(),
            max_tokens: 2048,
            temperature: 0.3,
        }
    }

    /// Override the default max_tokens (2048).
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Override the default temperature (0.3).
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    // ── Helpers ────────────────────────────────────────────────────────────

    /// Convert ADK `Content` messages into OpenAI-style `ChatMessage` list.
    fn build_messages(&self, request: &LlmRequest) -> Vec<ChatMessage> {
        let mut messages: Vec<ChatMessage> = Vec::new();

        // Append conversation turns
        for content in &request.contents {
            let text = content
                .parts
                .iter()
                .filter_map(|p: &Part| p.text())
                .collect::<Vec<_>>()
                .join("\n");

            // ADK uses "model" for assistant turns; OpenAI uses "assistant"
            let role = match content.role.as_str() {
                "model" => "assistant".to_string(),
                other => other.to_string(),
            };

            messages.push(ChatMessage {
                role,
                content: text,
            });
        }

        messages
    }

    /// Map an optional `finish_reason` string to an ADK `FinishReason`.
    fn map_finish_reason(reason: Option<&str>) -> Option<FinishReason> {
        match reason {
            Some("stop") => Some(FinishReason::Stop),
            Some("length") => Some(FinishReason::MaxTokens),
            Some("tool_calls") => Some(FinishReason::Stop), // no tool support yet
            _ => None,
        }
    }

    /// Perform a non-streaming completion and return a single `LlmResponse`.
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, AdkError> {
        let messages = self.build_messages(request);

        let body = ChatRequest {
            model: "/model".to_string(),
            messages,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            stream: None,
        };

        let resp = self
            .client
            .post(&self.base_url)
            .header("Content-Type", "application/json")
            .header("apikey", &self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| AdkError::model(format!("ThaiLLM HTTP error: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AdkError::model(format!(
                "ThaiLLM returned {status}: {body}"
            )));
        }

        let chat: ChatResponse = resp
            .json()
            .await
            .map_err(|e| AdkError::model(format!("ThaiLLM JSON parse error: {e}")))?;

        let choice = chat
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| AdkError::model("ThaiLLM returned no choices".to_string()))?;

        let finish_reason = Self::map_finish_reason(choice.finish_reason.as_deref());

        let usage = chat.usage.map(|u| UsageMetadata {
            prompt_token_count: u.prompt_tokens as i32,
            candidates_token_count: u.completion_tokens as i32,
            total_token_count: u.total_tokens as i32,
            thinking_token_count: None,
            cost: None,
            is_byok: Some(false),
            provider_usage: None,
            audio_input_token_count: None,
            audio_output_token_count: None,
            cache_creation_input_token_count: None,
            cache_read_input_token_count: None,
        });

        let content = Content::new("model").with_text(&choice.message.content);

        Ok(LlmResponse {
            content: Some(content),
            finish_reason,
            usage_metadata: usage,
            partial: false,
            ..Default::default()
        })
    }

    /// Perform a streaming completion, returning an `LlmResponseStream`.
    async fn complete_stream(
        &self,
        request: &LlmRequest,
    ) -> Result<LlmResponseStream, AdkError> {
        let messages = self.build_messages(request);

        let body = ChatRequest {
            model: "/model".to_string(),
            messages,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            stream: Some(true),
        };

        let resp = self
            .client
            .post(&self.base_url)
            .header("Content-Type", "application/json")
            .header("apikey", &self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| AdkError::model(format!("ThaiLLM HTTP error: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AdkError::model(format!(
                "ThaiLLM returned {status}: {body}"
            )));
        }

        // Collect SSE lines and turn each `data: {...}` into an LlmResponse
        let mut byte_stream = resp.bytes_stream();
        let mut buffer = String::new();
        let mut responses: Vec<Result<LlmResponse, AdkError>> = Vec::new();

        while let Some(chunk) = byte_stream.next().await {
            let bytes =
                chunk.map_err(|e| AdkError::model(format!("ThaiLLM stream error: {e}")))?;
            buffer.push_str(&String::from_utf8_lossy(&bytes));

            // Process complete SSE lines
            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim().to_string();
                buffer = buffer[pos + 1..].to_string();

                if let Some(data) = line.strip_prefix("data: ") {
                    if data.trim() == "[DONE]" {
                        break;
                    }
                    match serde_json::from_str::<StreamChunk>(data) {
                        Ok(chunk) => {
                            if let Some(choice) = chunk.choices.into_iter().next() {
                                let text = choice.delta.content.unwrap_or_default();
                                let finish_reason =
                                    Self::map_finish_reason(choice.finish_reason.as_deref());
                                let is_last = finish_reason.is_some();
                                let content = Content::new("model").with_text(&text);

                                responses.push(Ok(LlmResponse {
                                    content: Some(content),
                                    finish_reason,
                                    partial: !is_last,
                                    ..Default::default()
                                }));
                            }
                        }
                        Err(e) => {
                            responses.push(Err(AdkError::model(format!(
                                "ThaiLLM stream parse error: {e}"
                            ))));
                        }
                    }
                }
            }
        }

        let stream: LlmResponseStream = Box::pin(stream::iter(responses));
        Ok(stream)
    }
}

// ─── Llm trait implementation ─────────────────────────────────────────────────

#[async_trait]
impl Llm for ThaiLLMModel {
    fn name(&self) -> &str {
        &self.model_name
    }

    async fn generate_content(
        &self,
        request: LlmRequest,
        stream: bool,
    ) -> Result<LlmResponseStream, AdkError> {
        if stream {
            self.complete_stream(&request).await
        } else {
            let response = self.complete(&request).await?;
            Ok(Box::pin(stream::once(async move { Ok(response) })))
        }
    }
}

