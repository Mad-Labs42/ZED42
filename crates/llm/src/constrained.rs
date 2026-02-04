//! Constrained generation module for type-safe LLM outputs
//!
//! Uses `schemars` to derive JSON schemas from Rust types,
//! enabling structured, validated LLM responses.

use crate::client::LlmClient;
use crate::types::{LlmError, LlmRequest, ModelConfig, Result, RetryCause};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Configuration for constrained generation
#[derive(Debug, Clone)]
pub struct ConstrainedGenConfig {
    /// Maximum retry attempts if validation fails
    pub max_retries: u8,
    /// Model configuration
    pub model_config: ModelConfig,
    /// Include validation error in retry prompt
    pub include_error_in_retry: bool,
}

impl Default for ConstrainedGenConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            model_config: ModelConfig::default(),
            include_error_in_retry: true,
        }
    }
}

/// Constrained generation wrapper for type-safe LLM outputs
///
/// # Example
/// ```ignore
/// #[derive(JsonSchema, Deserialize)]
/// struct FunctionSpec {
///     name: String,
///     parameters: Vec<String>,
///     body: String,
/// }
///
/// let result: FunctionSpec = ConstrainedGen::new(&client)
///     .prompt("Generate a Rust function that adds two numbers")
///     .generate::<FunctionSpec>()
///     .await?;
/// ```
pub struct ConstrainedGen<'a> {
    client: &'a dyn LlmClient,
    prompt: String,
    system_prompt: Option<String>,
    config: ConstrainedGenConfig,
}

impl<'a> ConstrainedGen<'a> {
    /// Create a new constrained generation builder
    pub fn new(client: &'a dyn LlmClient) -> Self {
        Self {
            client,
            prompt: String::new(),
            system_prompt: None,
            config: ConstrainedGenConfig::default(),
        }
    }

    /// Set the user prompt
    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = prompt.into();
        self
    }

    /// Set the system prompt
    pub fn system(mut self, system: impl Into<String>) -> Self {
        self.system_prompt = Some(system.into());
        self
    }

    /// Configure max retries
    pub fn max_retries(mut self, retries: u8) -> Self {
        self.config.max_retries = retries;
        self
    }

    /// Set model configuration
    pub fn model_config(mut self, config: ModelConfig) -> Self {
        self.config.model_config = config;
        self
    }

    /// Generate a structured response of type T
    ///
    /// Uses the JSON schema derived from T to constrain LLM output.
    /// Retries up to `max_retries` times on validation failure.
    pub async fn generate<T>(self) -> Result<T>
    where
        T: JsonSchema + DeserializeOwned,
    {
        let schema = schemars::schema_for!(T);
        let schema_json: Value = serde_json::to_value(&schema)
            .map_err(|e| LlmError::InvalidSchema(e.to_string()))?;

        // Build the schema instruction
        let schema_instruction = format!(
            "You MUST respond with valid JSON matching this schema:\n```json\n{}\n```\nDo not include any text outside the JSON object.",
            serde_json::to_string_pretty(&schema_json)
                .map_err(|e| LlmError::InvalidSchema(e.to_string()))?
        );

        let mut last_error: Option<LlmError> = None;

        for attempt in 0..=self.config.max_retries {
            // Build the prompt with schema and any previous error
            let user_prompt = if attempt == 0 || !self.config.include_error_in_retry {
                format!("{}\n\n{}", self.prompt, schema_instruction)
            } else if let Some(ref err) = last_error {
                format!(
                    "{}\n\n{}\n\nPrevious attempt failed with: {}. Please fix and try again.",
                    self.prompt, schema_instruction, err
                )
            } else {
                format!("{}\n\n{}", self.prompt, schema_instruction)
            };

            // Build request
            let mut request = LlmRequest::new(user_prompt)
                .config(self.config.model_config.clone())
                .schema(schema_json.clone())
                .retry_count(attempt as u8);

            if last_error.is_some() {
                request = request.retry_cause(RetryCause::ValidationFailure);
            }

            if let Some(ref system) = self.system_prompt {
                request = request.system(system.clone());
            }

            // Call LLM
            let response = self.client.complete(request).await?;

            // Attempt to parse response
            match serde_json::from_str::<T>(&response.content) {
                Ok(parsed) => return Ok(parsed),
                Err(e) => {
                    tracing::warn!(
                        attempt = attempt,
                        error = %e,
                        "Constrained generation failed to parse, retrying"
                    );
                    last_error = Some(LlmError::InvalidResponse(format!(
                        "JSON parse error: {}",
                        e
                    )));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            LlmError::InvalidResponse("Exhausted retries without valid response".to_string())
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::MockLlmClient;
    use schemars::JsonSchema;
    use serde::Deserialize;

    #[derive(Debug, JsonSchema, Deserialize, PartialEq)]
    struct SimpleResponse {
        message: String,
        count: i32,
    }

    #[tokio::test]
    async fn test_constrained_gen_success() {
        let mock_response = r#"{"message": "hello", "count": 42}"#;
        let client = MockLlmClient::new(mock_response.to_string());

        let result: SimpleResponse = ConstrainedGen::new(&client)
            .prompt("Generate a test response")
            .max_retries(1)
            .generate()
            .await
            .expect("Should succeed");

        assert_eq!(result.message, "hello");
        assert_eq!(result.count, 42);
    }

    #[tokio::test]
    async fn test_constrained_gen_invalid_json_fails() {
        let mock_response = "not valid json";
        let client = MockLlmClient::new(mock_response.to_string());

        let result: std::result::Result<SimpleResponse, _> = ConstrainedGen::new(&client)
            .prompt("Generate a test response")
            .max_retries(0) // No retries
            .generate()
            .await;

        assert!(result.is_err());
    }
}
