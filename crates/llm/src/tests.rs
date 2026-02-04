//! LLM integration tests

use super::*;
use crate::prompts::CommonPrompts;
use std::collections::HashMap;

#[test]
fn test_prompt_template_extraction() {
    let template = PromptTemplate::new("Hello {{name}}, you are {{age}} years old.".to_string());

    let vars = template.variables();
    assert_eq!(vars.len(), 2);
    assert!(vars.contains(&"name".to_string()));
    assert!(vars.contains(&"age".to_string()));
}

#[test]
fn test_prompt_template_render() {
    let template = PromptTemplate::new("Hello {{name}}!".to_string());

    let mut values = HashMap::new();
    values.insert("name".to_string(), "World".to_string());

    let result = template.render(&values).unwrap();
    assert_eq!(result, "Hello World!");
}

#[test]
fn test_prompt_template_missing_variable() {
    let template = PromptTemplate::new("Hello {{name}}!".to_string());

    let values = HashMap::new();
    let result = template.render(&values);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Missing required variable"));
}

#[test]
fn test_schema_builder() {
    let schema = JsonSchema::builder()
        .add_string("name", "The person's name", true)
        .add_number("age", "The person's age", true)
        .add_boolean("active", "Whether active", false)
        .build();

    let value = schema.as_value();
    assert!(value["properties"]["name"].is_object());
    assert!(value["properties"]["age"].is_object());
    assert!(value["properties"]["active"].is_object());

    let required = value["required"].as_array().unwrap();
    assert_eq!(required.len(), 2);
}

#[test]
fn test_schema_with_array() {
    let schema = JsonSchema::builder()
        .add_array("tags", "List of tags", "string", true)
        .build();

    let value = schema.as_value();
    assert_eq!(value["properties"]["tags"]["type"], "array");
    assert_eq!(value["properties"]["tags"]["items"]["type"], "string");
}

#[test]
fn test_schema_with_enum() {
    let schema = JsonSchema::builder()
        .add_enum("status", "The status", vec!["active", "inactive"], true)
        .build();

    let value = schema.as_value();
    assert_eq!(value["properties"]["status"]["type"], "string");

    let enum_values = value["properties"]["status"]["enum"].as_array().unwrap();
    assert_eq!(enum_values.len(), 2);
}

#[tokio::test]
async fn test_mock_client() {
    let client = MockLlmClient::new("Test response".to_string());

    let request = LlmRequest::new("Test prompt".to_string());
    let response = client.complete(request).await.unwrap();

    assert_eq!(response.content, "Test response");
    assert_eq!(response.model, "mock");
}

#[test]
fn test_model_config_defaults() {
    let config = ModelConfig::default();
    assert!(config.model.contains("claude"));
    assert_eq!(config.temperature, 0.7);
}

#[test]
fn test_model_config_fast() {
    let config = ModelConfig::fast();
    assert!(config.model.contains("haiku"));
    assert_eq!(config.max_tokens, Some(2048));
}

#[test]
fn test_model_config_powerful() {
    let config = ModelConfig::powerful();
    assert!(config.model.contains("opus"));
    assert_eq!(config.max_tokens, Some(8192));
}

#[test]
fn test_llm_request_builder() {
    let request = LlmRequest::new("Test".to_string())
        .system("You are helpful".to_string())
        .config(ModelConfig::fast())
        .stop("END".to_string());

    assert_eq!(request.prompt, "Test");
    assert_eq!(request.system_prompt, Some("You are helpful".to_string()));
    assert!(request.config.model.contains("haiku"));
    assert_eq!(request.stop_sequences.len(), 1);
}

#[test]
fn test_common_prompts() {
    let code_review = CommonPrompts::code_review();
    assert!(code_review.variables().contains(&"language".to_string()));
    assert!(code_review.variables().contains(&"code".to_string()));

    let decision = CommonPrompts::decision();
    assert!(decision
        .variables()
        .contains(&"decision_topic".to_string()));

    let decompose = CommonPrompts::decompose_task();
    assert!(decompose.variables().contains(&"task".to_string()));

    let risk = CommonPrompts::risk_analysis();
    assert!(risk.variables().contains(&"proposal".to_string()));
}
