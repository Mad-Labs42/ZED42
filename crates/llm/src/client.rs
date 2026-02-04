//! LLM client implementation

use crate::types::{LlmError, LlmRequest, LlmResponse, Result, StreamChunk, Usage};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

/// LLM client trait
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Generate a completion
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse>;

    /// Generate a streaming completion
    async fn stream(&self, request: LlmRequest) -> Result<Vec<StreamChunk>>;

    /// Generate embeddings
    async fn embed(&self, request: crate::types::EmbeddingRequest) -> Result<crate::types::EmbeddingResponse>;
}

/// OpenRouter client for development phase
pub struct OpenRouterClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenRouterClient {
    /// Create a new OpenRouter client
    ///
    /// # Arguments
    /// - `api_key` - OpenRouter API key
    pub fn new(api_key: String) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            api_key,
            base_url: "https://openrouter.ai/api/v1".to_string(),
        })
    }

    /// Create from environment variable
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .map_err(|_| LlmError::ApiError("OPENROUTER_API_KEY not set".to_string()))?;
        Self::new(api_key)
    }

    /// Build request body
    fn build_request_body(&self, request: &LlmRequest) -> serde_json::Value {
        let mut messages = Vec::new();

        // Add system message if provided
        if let Some(system) = &request.system_prompt {
            messages.push(json!({
                "role": "system",
                "content": system
            }));
        }

        // Add user message
        messages.push(json!({
            "role": "user",
            "content": request.prompt
        }));

        let mut body = json!({
            "model": request.config.model,
            "messages": messages,
            "temperature": request.config.temperature,
        });

        // Add optional fields
        if let Some(max_tokens) = request.config.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }

        if let Some(top_p) = request.config.top_p {
            body["top_p"] = json!(top_p);
        }

        if let Some(freq_penalty) = request.config.frequency_penalty {
            body["frequency_penalty"] = json!(freq_penalty);
        }

        if let Some(pres_penalty) = request.config.presence_penalty {
            body["presence_penalty"] = json!(pres_penalty);
        }

        if !request.stop_sequences.is_empty() {
            body["stop"] = json!(request.stop_sequences);
        }

        // Add JSON schema if provided
        if let Some(schema) = &request.json_schema {
            body["response_format"] = json!({
                "type": "json_schema",
                "json_schema": schema
            });
        }

        body
    }
}

#[async_trait]
impl LlmClient for OpenRouterClient {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let body = self.build_request_body(&request);

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::ApiError(format!(
                "API returned {}: {}",
                status, error_text
            )));
        }

        let response_json: serde_json::Value = response.json().await?;

        // Parse response
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| LlmError::InvalidResponse("Missing content".to_string()))?
            .to_string();

        let model = response_json["model"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        let finish_reason = response_json["choices"][0]["finish_reason"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        let usage = Usage {
            prompt_tokens: response_json["usage"]["prompt_tokens"]
                .as_u64()
                .unwrap_or(0) as usize,
            completion_tokens: response_json["usage"]["completion_tokens"]
                .as_u64()
                .unwrap_or(0) as usize,
            total_tokens: response_json["usage"]["total_tokens"]
                .as_u64()
                .unwrap_or(0) as usize,
        };

        Ok(LlmResponse {
            content,
            model,
            usage,
            finish_reason,
        })
    }

    async fn stream(&self, request: LlmRequest) -> Result<Vec<StreamChunk>> {
        // For now, simulate streaming by breaking up a regular response
        // TODO: Implement actual streaming when needed
        let response = self.complete(request).await?;

        Ok(vec![StreamChunk {
            content: response.content,
            is_final: true,
        }])
    }

    async fn embed(&self, request: crate::types::EmbeddingRequest) -> Result<crate::types::EmbeddingResponse> {
        let body = json!({
            "model": request.model,
            "input": request.input,
        });

        let response = self
            .client
            .post(format!("{}/embeddings", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::ApiError(format!(
                "API returned {}: {}",
                status, error_text
            )));
        }

        let response_json: serde_json::Value = response.json().await?;

        let embedding: Vec<f32> = response_json["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| LlmError::InvalidResponse("Missing embedding data".to_string()))?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        let usage = Usage {
            prompt_tokens: response_json["usage"]["prompt_tokens"]
                .as_u64()
                .unwrap_or(0) as usize,
            completion_tokens: 0,
            total_tokens: response_json["usage"]["total_tokens"]
                .as_u64()
                .unwrap_or(0) as usize,
        };

        Ok(crate::types::EmbeddingResponse {
            embedding,
            model: request.model,
            usage,
        })
    }
}

/// Mock client for testing
pub struct MockLlmClient {
    responses: parking_lot::Mutex<std::collections::VecDeque<String>>,
}

impl MockLlmClient {
    /// Create a mock client with predefined responses
    pub fn new(response: String) -> Self {
        let mut responses = std::collections::VecDeque::new();
        responses.push_back(response);
        Self { 
            responses: parking_lot::Mutex::new(responses) 
        }
    }

    /// Create a mock client with multiple predefined responses
    pub fn with_responses(responses: Vec<String>) -> Self {
        Self {
            responses: parking_lot::Mutex::new(responses.into())
        }
    }
}

#[async_trait]
impl LlmClient for MockLlmClient {
    async fn complete(&self, _request: LlmRequest) -> Result<LlmResponse> {
        let mut responses = self.responses.lock();
        let response = responses.pop_front()
            .ok_or_else(|| LlmError::InvalidResponse("MockLlmClient exhausted responses".to_string()))?;

        Ok(LlmResponse {
            content: response,
            model: "mock".to_string(),
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            },
            finish_reason: "stop".to_string(),
        })
    }

    async fn stream(&self, _request: LlmRequest) -> Result<Vec<StreamChunk>> {
        let mut responses = self.responses.lock();
        let response = responses.pop_front()
            .ok_or_else(|| LlmError::InvalidResponse("MockLlmClient exhausted responses".to_string()))?;

        Ok(vec![StreamChunk {
            content: response,
            is_final: true,
        }])
    }

    async fn embed(&self, request: crate::types::EmbeddingRequest) -> Result<crate::types::EmbeddingResponse> {
        Ok(crate::types::EmbeddingResponse {
            embedding: vec![0.1; 1536], // Mock dimension
            model: request.model,
            usage: Usage {
                prompt_tokens: 5,
                completion_tokens: 0,
                total_tokens: 5,
            },
        })
    }
}
