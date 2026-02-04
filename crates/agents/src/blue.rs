//! Blue Team agents - Defensive Implementation
//!
//! Blue Team focuses on implementing features safely with tests.

use async_trait::async_trait;
use std::sync::Arc;
use zed42_core::{AgentBehavior, AgentId, Artifact, Result, Task};
use zed42_llm::{ConstrainedGen, LlmClient};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use uuid::Uuid;

/// Agent state machine - enforces valid transitions
#[derive(Debug, Clone)]
pub enum AgentState {
    /// Agent is idle, waiting for a task
    Idle,
    /// Agent is processing a task
    Processing(Task),
    /// Agent has produced an artifact and is awaiting review
    AwaitingReview(Artifact),
    /// Agent has completed its work
    Done(Artifact),
    /// Agent encountered an error
    Failed(String),
}

impl AgentState {
    /// Check if agent can accept a new task
    pub fn can_accept_task(&self) -> bool {
        matches!(self, AgentState::Idle)
    }

    /// Transition to Processing state
    pub fn start_processing(self, task: Task) -> std::result::Result<Self, &'static str> {
        match self {
            AgentState::Idle => Ok(AgentState::Processing(task)),
            _ => Err("Can only start processing from Idle state"),
        }
    }

    /// Transition to AwaitingReview state
    pub fn submit_for_review(self, artifact: Artifact) -> std::result::Result<Self, &'static str> {
        match self {
            AgentState::Processing(_) => Ok(AgentState::AwaitingReview(artifact)),
            _ => Err("Can only submit for review from Processing state"),
        }
    }

    /// Transition to Done state
    pub fn approve(self) -> std::result::Result<Self, &'static str> {
        match self {
            AgentState::AwaitingReview(artifact) => Ok(AgentState::Done(artifact)),
            _ => Err("Can only approve from AwaitingReview state"),
        }
    }
}

/// LLM response schema for code generation
#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
pub struct CodeGenerationResponse {
    /// The generated function or code
    pub code: String,
    /// Generated tests for the code
    pub tests: Option<String>,
    /// Brief explanation of the implementation
    pub explanation: String,
}

/// LLM response schema for self-critique
#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
pub struct CritiqueResponse {
    /// List of identified issues
    pub issues: Vec<String>,
    /// Whether the code passes all standards
    pub pass: bool,
    /// Suggestions for improvement
    pub suggestions: Vec<String>,
}

/// FeatureImplementer agent - implements new functionality with tests
pub struct FeatureImplementer {
    id: AgentId,
    llm_client: Arc<dyn LlmClient>,
    state: AgentState,
    max_reflexion_iterations: u8,
}

impl FeatureImplementer {
    /// Create a new FeatureImplementer agent
    pub fn new(llm_client: Arc<dyn LlmClient>) -> Self {
        Self {
            id: Uuid::new_v4(),
            llm_client,
            state: AgentState::Idle,
            max_reflexion_iterations: 3,
        }
    }

    /// Process a task and return an artifact
    pub async fn process_task(&mut self, task: Task) -> Result<Artifact> {
        // Transition to Processing state
        self.state = self.state.clone()
            .start_processing(task.clone())
            .map_err(|e| zed42_core::Error::Agent(e.to_string()))?;

        tracing::info!(agent_id = %self.id, task_id = %task.id, "Starting task processing");

        let mut current_code: Option<CodeGenerationResponse> = None;
        let mut feedback: Option<String> = None;

        // --- Phase 2.4: Reflexion Loop ---
        for i in 0..self.max_reflexion_iterations {
            tracing::info!(agent_id = %self.id, iteration = i, "Reflexion loop: Generating implementation");
            
            // 1. Generate Proposal
            let mut prompt = self.build_prompt(&task);
            if let Some(ref fb) = feedback {
                prompt.push_str(&format!("\n\nPrevious attempt had the following issues:\n{}\nPlease fix these and provide a new implementation.", fb));
            }

            let response: CodeGenerationResponse = ConstrainedGen::new(self.llm_client.as_ref())
                .system(self.system_prompt())
                .prompt(prompt)
                .generate()
                .await
                .map_err(|e| zed42_core::Error::Llm(e.to_string()))?;

            // 2. Critique Proposal
            tracing::info!(agent_id = %self.id, iteration = i, "Reflexion loop: Critiquing implementation");
            let critique_prompt = format!(
                "Critique the following code implementation based on the task and context.\n\
                Task: {}\n\
                Implementation:\n```rust\n{}\n```\n\
                Evaluate for: correctness, safety (no unwrap), adherence to constraints, and test coverage.",
                task.description, response.code
            );

            let critique: CritiqueResponse = ConstrainedGen::new(self.llm_client.as_ref())
                .system("You are a senior security and quality reviewer. Be strict. Reject any code with unhandled results, inadequate comments, or missing edge cases.")
                .prompt(critique_prompt)
                .generate()
                .await
                .map_err(|e| zed42_core::Error::Llm(e.to_string()))?;

            if critique.pass {
                tracing::info!(agent_id = %self.id, iteration = i, "Reflexion loop: Critique passed");
                current_code = Some(response);
                break;
            } else {
                tracing::warn!(agent_id = %self.id, iteration = i, "Reflexion loop: Critique failed, retrying");
                feedback = Some(critique.issues.join("\n"));
                current_code = Some(response); // Keep latest as fallback
            }
        }

        let final_response = current_code.ok_or_else(|| {
            zed42_core::Error::Agent("Exhausted reflexion iterations without generating code".to_string())
        })?;

        // Create artifact
        let artifact = Artifact::code(
            task.id.clone(),
            final_response.code,
            None, // File path will be set by toolbox
        );

        // Transition to AwaitingReview
        self.state = self.state.clone()
            .submit_for_review(artifact.clone())
            .map_err(|e| zed42_core::Error::Agent(e.to_string()))?;

        Ok(artifact)
    }

    /// Approve the current artifact and transition to Done
    pub fn approve(&mut self) -> Result<Artifact> {
        let new_state = self.state.clone()
            .approve()
            .map_err(|e| zed42_core::Error::Agent(e.to_string()))?;

        if let AgentState::Done(artifact) = new_state.clone() {
            self.state = new_state;
            Ok(artifact)
        } else {
            Err(zed42_core::Error::Agent("Invalid state after approval".to_string()))
        }
    }

    /// Get current agent state
    pub fn state(&self) -> &AgentState {
        &self.state
    }

    fn system_prompt(&self) -> String {
        "You are a senior software engineer focused on implementing features. \
         Write clean, idiomatic code with proper error handling. \
         Always include documentation comments. \
         Follow SOLID principles and industry best practices.".to_string()
    }

    fn build_prompt(&self, task: &Task) -> String {
        let mut prompt = format!("Task: {}\n", task.description);

        if let Some(ref context) = task.context {
            prompt.push_str(&format!("\nContext: {}\n", context));
        }

        if !task.constraints.is_empty() {
            prompt.push_str("\nConstraints:\n");
            for constraint in &task.constraints {
                prompt.push_str(&format!("- {}\n", constraint));
            }
        }

        prompt.push_str("\nGenerate code that fulfills this task.");
        prompt
    }
}

#[async_trait]
impl AgentBehavior for FeatureImplementer {
    fn id(&self) -> AgentId {
        self.id
    }

    async fn initialize(&mut self) -> Result<()> {
        tracing::info!(agent_id = %self.id, "FeatureImplementer initialized");
        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        // In the full implementation, this would poll for tasks from the blackboard
        tracing::info!(agent_id = %self.id, "FeatureImplementer running");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!(agent_id = %self.id, "FeatureImplementer shutting down");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zed42_llm::MockLlmClient;

    #[test]
    fn test_agent_state_transitions() {
        let state = AgentState::Idle;
        assert!(state.can_accept_task());

        let task = Task::new("Test task");
        let state = state.start_processing(task).unwrap();
        assert!(!state.can_accept_task());

        let artifact = Artifact::code("task-1".to_string(), "fn foo() {}".to_string(), None);
        let state = state.submit_for_review(artifact).unwrap();
        assert!(!state.can_accept_task());

        let state = state.approve().unwrap();
        assert!(matches!(state, AgentState::Done(_)));
    }

    #[test]
    fn test_invalid_state_transition() {
        let state = AgentState::Idle;
        let artifact = Artifact::code("task-1".to_string(), "fn foo() {}".to_string(), None);

        // Cannot submit for review from Idle
        let result = state.submit_for_review(artifact);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_feature_implementer_process_task() {
        let code_response = r#"{"code": "fn add(a: i32, b: i32) -> i32 { a + b }", "tests": null, "explanation": "Simple addition function"}"#;
        let critique_response = r#"{"issues": [], "pass": true, "suggestions": []}"#;
        
        let client = Arc::new(MockLlmClient::with_responses(vec![
            code_response.to_string(),
            critique_response.to_string(),
        ]));

        let mut agent = FeatureImplementer::new(client);
        let task = Task::new("Create a function that adds two numbers");

        let artifact = agent.process_task(task).await.expect("Should succeed");

        assert!(artifact.content.contains("add"));
        assert!(matches!(agent.state(), AgentState::AwaitingReview(_)));
    }

    #[tokio::test]
    async fn test_feature_implementer_with_reflexion() {
        let code_1 = r#"{"code": "fn add(a: i32, b: i32) { a + b }", "tests": null, "explanation": "Missing return type"}"#;
        let critique_1 = r#"{"issues": ["Missing return type"], "pass": false, "suggestions": ["Add -> i32"]}"#;
        let code_2 = r#"{"code": "fn add(a: i32, b: i32) -> i32 { a + b }", "tests": null, "explanation": "Fixed version"}"#;
        let critique_2 = r#"{"issues": [], "pass": true, "suggestions": []}"#;

        let client = Arc::new(MockLlmClient::with_responses(vec![
            code_1.to_string(),
            critique_1.to_string(),
            code_2.to_string(),
            critique_2.to_string(),
        ]));

        let mut agent = FeatureImplementer::new(client);
        let task = Task::new("Create a function that adds two numbers");

        let artifact = agent.process_task(task).await.expect("Should succeed after reflexion");

        assert!(artifact.content.contains("-> i32"));
        assert!(matches!(agent.state(), AgentState::AwaitingReview(_)));
    }
}
