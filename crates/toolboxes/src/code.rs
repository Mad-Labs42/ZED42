//! Code generation tools
//!
//! Provides structured code generation capabilities using LLMs.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::{Tool, ToolResult};
use zed42_llm::{ConstrainedGen, LlmClient};
use schemars::JsonSchema;

/// Parameters for GenerateFunction tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GenerateFunctionParams {
    /// Description of the function to generate
    pub description: String,
    /// Optional context (e.g., existing code, types)
    pub context: Option<String>,
    /// Target language (default: rust)
    pub language: Option<String>,
}

/// Result of code generation
#[derive(Debug, Serialize, JsonSchema, Deserialize)]
pub struct GenerateFunctionResult {
    /// The generated code
    pub code: String,
    /// Explanation of the implementation
    pub explanation: String,
}

/// GenerateFunction tool - generates a single function based on description
pub struct GenerateFunction {
    llm_client: Arc<dyn LlmClient>,
}

impl GenerateFunction {
    pub fn new(llm_client: Arc<dyn LlmClient>) -> Self {
        Self { llm_client }
    }
}

#[async_trait]
impl Tool for GenerateFunction {
    fn name(&self) -> &str {
        "generate_function"
    }

    fn description(&self) -> &str {
        "Generate a single code function based on a natural language description"
    }

    fn parameter_schema(&self) -> Value {
        // We use schemars to generate the schema for the LLM tool call itself if needed,
        // but the Tool trait requires returning a Value.
        json!({
            "type": "object",
            "properties": {
                "description": {
                    "type": "string",
                    "description": "Description of what the function should do"
                },
                "context": {
                    "type": "string",
                    "description": "Additional context or related code"
                },
                "language": {
                    "type": "string",
                    "description": "Target programming language",
                    "default": "rust"
                }
            },
            "required": ["description"]
        })
    }

    async fn execute(&self, params: Value) -> ToolResult {
        let params: GenerateFunctionParams = serde_json::from_value(params)
            .map_err(|e| anyhow::anyhow!("Invalid parameters: {}", e))?;

        let language = params.language.unwrap_or_else(|| "rust".to_string());
        
        let prompt = format!(
            "Generate a {} function based on this description: {}\n\nContext: {}",
            language,
            params.description,
            params.context.unwrap_or_default()
        );

        let result: GenerateFunctionResult = ConstrainedGen::new(self.llm_client.as_ref())
            .system(format!("You are an expert {} developer.", language))
            .prompt(prompt)
            .generate()
            .await
            .map_err(|e| anyhow::anyhow!("Generation failed: {}", e))?;

        Ok(serde_json::to_value(result)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zed42_llm::MockLlmClient;

    #[tokio::test]
    async fn test_generate_function_tool() {
        let mock_response = r#"{"code": "fn test() {}", "explanation": "Simple test function"}"#;
        let client = Arc::new(MockLlmClient::new(mock_response.to_string()));
        
        let tool = GenerateFunction::new(client);
        let result = tool.execute(json!({
            "description": "create a test function"
        })).await.expect("Tool execution failed");

        assert_eq!(result["code"], "fn test() {}");
    }
}
