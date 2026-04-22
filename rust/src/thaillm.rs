use adk_core::{
    AdkError, Content, FinishReason, Llm, LlmRequest, LlmResponse, LlmResponseStream, Part,
    UsageMetadata,
};
use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// ─── Request / Response types (OpenAI-compatible) ────────────────────────────

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Tool {
    #[serde(rename = "type")]
    r#type: String,
    function: FunctionDefinition,
}

#[derive(Debug, Serialize, Deserialize)]
struct FunctionDefinition {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    parameters: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ToolCall {
    id: String,
    #[serde(rename = "type")]
    r#type: String,
    function: ToolFunction,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ToolFunction {
    name: String,
    arguments: String,
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
    #[serde(default)]
    tool_calls: Option<Vec<ToolCallChunk>>,
}

#[derive(Debug, Deserialize)]
struct ToolCallChunk {
    index: u32,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    function: Option<ToolFunctionChunk>,
}

#[derive(Debug, Deserialize, Default)]
struct ToolFunctionChunk {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<String>,
}

struct ToolCallAggregator {
    id: String,
    name: String,
    arguments: String,
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
    #[allow(dead_code)]
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Override the default temperature (0.3).
    #[allow(dead_code)]
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    // ── Helpers ────────────────────────────────────────────────────────────

    /// Convert ADK `Content` messages into OpenAI-style `ChatMessage` list.
    fn build_messages(&self, request: &LlmRequest) -> Vec<ChatMessage> {
        let mut messages: Vec<ChatMessage> = Vec::new();

        for content in &request.contents {
            let role = match content.role.as_str() {
                "model" => "assistant".to_string(),
                other => other.to_string(),
            };

            let mut text_parts = Vec::new();
            let mut tool_calls = Vec::new();

            for part in &content.parts {
                match part {
                    Part::Text { text } => {
                        text_parts.push(text.clone());
                    }
                    Part::FunctionCall {
                        name,
                        args,
                        id,
                        ..
                    } => {
                        tool_calls.push(ToolCall {
                            id: id.clone().unwrap_or_default(),
                            r#type: "function".to_string(),
                            function: ToolFunction {
                                name: name.clone(),
                                arguments: args.to_string(),
                            },
                        });
                    }
                    Part::FunctionResponse {
                        id,
                        function_response,
                        ..
                    } => {
                        let response_str = serde_json::to_string(function_response).unwrap_or_default();
                        messages.push(ChatMessage {
                            role: "tool".to_string(),
                            content: Some(response_str),
                            tool_calls: None,
                            tool_call_id: id.clone(),
                            name: None,
                        });
                    }
                    _ => {}
                }
            }

            if !text_parts.is_empty() || !tool_calls.is_empty() {
                let content_str = if text_parts.is_empty() {
                    None
                } else {
                    Some(text_parts.join("\n"))
                };

                let tcs = if tool_calls.is_empty() {
                    None
                } else {
                    Some(tool_calls)
                };

                messages.push(ChatMessage {
                    role,
                    content: content_str,
                    tool_calls: tcs,
                    tool_call_id: None,
                    name: None,
                });
            }
        }

        messages
    }

    /// Map an optional `finish_reason` string to an ADK `FinishReason`.
    fn map_finish_reason(reason: Option<&str>) -> Option<FinishReason> {
        match reason {
            Some("stop") => Some(FinishReason::Stop),
            Some("length") => Some(FinishReason::MaxTokens),
            Some("tool_calls") => Some(FinishReason::Stop),
            _ => None,
        }
    }

    /// Perform a non-streaming completion and return a single `LlmResponse`.
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, AdkError> {
        let messages = self.build_messages(request);

        let tools = if request.tools.is_empty() {
            None
        } else {
            Some(
                request
                    .tools
                    .iter()
                    .map(|(name, def)| Tool {
                        r#type: "function".to_string(),
                        function: FunctionDefinition {
                            name: name.clone(),
                            description: def["description"].as_str().map(|s| s.to_string()),
                            parameters: def["parameters"].clone(),
                        },
                    })
                    .collect(),
            )
        };

        let body = ChatRequest {
            model: "/model".to_string(),
            messages,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            stream: None,
            tools,
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

        let mut content = Content::new("model");
        if let Some(text) = choice.message.content {
            content = content.with_text(&text);
        }

        if let Some(tool_calls) = choice.message.tool_calls {
            for tc in tool_calls {
                let args: Value = serde_json::from_str(&tc.function.arguments)
                    .unwrap_or_else(|_| serde_json::json!({}));
                content.parts.push(Part::FunctionCall {
                    name: tc.function.name,
                    args,
                    id: Some(tc.id),
                    thought_signature: None,
                });
            }
        }

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

        let tools = if request.tools.is_empty() {
            None
        } else {
            Some(
                request
                    .tools
                    .iter()
                    .map(|(name, def)| Tool {
                        r#type: "function".to_string(),
                        function: FunctionDefinition {
                            name: name.clone(),
                            description: def["description"].as_str().map(|s| s.to_string()),
                            parameters: def["parameters"].clone(),
                        },
                    })
                    .collect(),
            )
        };

        let body = ChatRequest {
            model: "/model".to_string(),
            messages,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            stream: Some(true),
            tools,
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
        let mut tool_aggregators: HashMap<u32, ToolCallAggregator> = HashMap::new();

        let stream = async_stream::try_stream! {
            while let Some(chunk) = byte_stream.next().await {
                let bytes = chunk.map_err(|e| AdkError::model(format!("ThaiLLM stream error: {e}")))?;
                buffer.push_str(&String::from_utf8_lossy(&bytes));

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
                                    if let Some(text) = choice.delta.content {
                                        yield LlmResponse {
                                            content: Some(Content::new("model").with_text(&text)),
                                            partial: true,
                                            ..Default::default()
                                        };
                                    }

                                    if let Some(tool_calls) = choice.delta.tool_calls {
                                        for tc in tool_calls {
                                            let entry = tool_aggregators.entry(tc.index).or_insert(ToolCallAggregator {
                                                id: String::new(),
                                                name: String::new(),
                                                arguments: String::new(),
                                            });
                                            if let Some(id) = tc.id {
                                                entry.id.push_str(&id);
                                            }
                                            if let Some(func) = tc.function {
                                                if let Some(name) = func.name {
                                                    entry.name.push_str(&name);
                                                }
                                                if let Some(args) = func.arguments {
                                                    entry.arguments.push_str(&args);
                                                }
                                            }
                                        }
                                    }

                                    if let Some(reason) = choice.finish_reason.as_deref() {
                                        let finish_reason = Self::map_finish_reason(Some(reason));
                                        let mut content = Content::new("model");
                                        
                                        if !tool_aggregators.is_empty() {
                                            let mut sorted_indices: Vec<_> = tool_aggregators.keys().cloned().collect();
                                            sorted_indices.sort();
                                            for idx in sorted_indices {
                                                let agg = tool_aggregators.get(&idx).unwrap();
                                                let args: Value = serde_json::from_str(&agg.arguments)
                                                    .unwrap_or_else(|_| serde_json::json!({}));
                                                content.parts.push(Part::FunctionCall {
                                                    name: agg.name.clone(),
                                                    args,
                                                    id: Some(agg.id.clone()),
                                                    thought_signature: None,
                                                });
                                            }
                                        }

                                        yield LlmResponse {
                                            content: if content.parts.is_empty() { None } else { Some(content) },
                                            finish_reason,
                                            partial: false,
                                            ..Default::default()
                                        };
                                    }
                                }
                            }
                            Err(e) => {
                                yield Err(AdkError::model(format!("ThaiLLM stream parse error: {e}")))?;
                            }
                        }
                    }
                }
            }
        };

        Ok(Box::pin(stream))
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
