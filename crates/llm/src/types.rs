//! LLM type definitions

use serde::{Deserialize, Serialize};

/// LLM error types
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid schema: {0}")]
    InvalidSchema(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Backpressure detected: Retry in {0:?}")]
    Backpressure(std::time::Duration),
}

pub type Result<T> = std::result::Result<T, LlmError>;

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model: String,
    pub temperature: f32,
    pub max_tokens: Option<usize>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model: "anthropic/claude-3.5-sonnet".to_string(),
            temperature: 0.7,
            max_tokens: Some(4096),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
        }
    }
}

impl ModelConfig {
    /// Create config for fast, cheap model (Haiku)
    pub fn fast() -> Self {
        Self {
            model: "anthropic/claude-3-haiku".to_string(),
            temperature: 0.5,
            max_tokens: Some(2048),
            ..Default::default()
        }
    }

    /// Create config for powerful model (Opus)
    pub fn powerful() -> Self {
        Self {
            model: "anthropic/claude-3-opus".to_string(),
            temperature: 0.7,
            max_tokens: Some(8192),
            ..Default::default()
        }
    }

    /// Set temperature
    pub fn temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }

    /// Set max tokens
    pub fn max_tokens(mut self, tokens: usize) -> Self {
        self.max_tokens = Some(tokens);
        self
    }
}

/// LLM request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub config: ModelConfig,
    pub json_schema: Option<serde_json::Value>,
    pub stop_sequences: Vec<String>,
    pub retry_count: u8,
    pub retry_cause: Option<RetryCause>,
    pub agent_id: Option<String>,
}

impl LlmRequest {
    /// Create a new request
    pub fn new(prompt: String) -> Self {
        Self {
            prompt,
            system_prompt: None,
            config: ModelConfig::default(),
            json_schema: None,
            stop_sequences: Vec::new(),
            retry_count: 0,
            retry_cause: None,
            agent_id: None,
        }
    }

    /// Set agent ID
    pub fn agent(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    /// Set system prompt
    pub fn system(mut self, system: String) -> Self {
        self.system_prompt = Some(system);
        self
    }

    /// Set model config
    pub fn config(mut self, config: ModelConfig) -> Self {
        self.config = config;
        self
    }

    /// Set JSON schema constraint
    pub fn schema(mut self, schema: serde_json::Value) -> Self {
        self.json_schema = Some(schema);
        self
    }

    /// Add stop sequence
    pub fn stop(mut self, sequence: String) -> Self {
        self.stop_sequences.push(sequence);
        self
    }
    
    /// Set retry count
    pub fn retry_count(mut self, count: u8) -> Self {
        self.retry_count = count;
        self
    }

    /// Set retry cause
    pub fn retry_cause(mut self, cause: RetryCause) -> Self {
        self.retry_cause = Some(cause);
        self
    }
}

/// Reason for retrying a request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryCause {
    RateLimit,
    ServerBusy,
    ValidationFailure,
    Other(String),
}

/// LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub usage: Usage,
    pub finish_reason: String,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

/// Streaming chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub content: String,
    pub is_final: bool,
}
/// Embedding request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub input: String,
    pub model: String,
}

impl EmbeddingRequest {
    pub fn new(input: String) -> Self {
        Self {
            input,
            model: "text-embedding-3-small".to_string(), // High-efficiency default
        }
    }
}

/// Embedding response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embedding: Vec<f32>,
    pub model: String,
    pub usage: Usage,
}
