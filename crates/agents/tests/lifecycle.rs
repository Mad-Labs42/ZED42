use std::sync::Arc;
use tempfile::tempdir;
use zed42_agents::blue::FeatureImplementer;
use zed42_core::{AgentStatus, Artifact, Task};
use zed42_llm::MockLlmClient;
use zed42_toolboxes::file_manipulation::WriteFile;
use zed42_toolboxes::Tool;
use serde_json::json;

#[tokio::test]
async fn test_end_to_end_agent_task_lifecycle() {
    // 1. Setup Environment
    let temp = tempdir().expect("Failed to create temp dir");
    let sandbox_root = temp.path().to_path_buf();
    
    // 2. Mock LLM Responses (Generation + Critique)
    let code_res = r#"{"code": "pub fn hello() { println!(\"Hello\"); }", "tests": null, "explanation": "Simple hello function"}"#;
    let critique_res = r#"{"issues": [], "pass": true, "suggestions": []}"#;
    
    let llm_client = Arc::new(MockLlmClient::with_responses(vec![
        code_res.to_string(),
        critique_res.to_string(),
    ]));

    // 3. Initialize Agent
    let mut agent = FeatureImplementer::new(llm_client.clone());
    
    // 4. Assign Task
    let task = Task::new("Implement a hello function")
        .with_context("The function should be public and use println!")
        .with_constraint("No unwrap allowed");

    // 5. Execute Task (Process -> Reflexion -> Artifact)
    let artifact = agent.process_task(task.clone())
        .await
        .expect("Agent failed to process task");

    // 6. Verify Artifact and Agent State
    assert!(artifact.content.contains("pub fn hello"));
    assert_eq!(artifact.task_id, task.id);
    
    // Check Agent State (should be AwaitingReview)
    match agent.state() {
        zed42_agents::blue::AgentState::AwaitingReview(ref a) => {
            assert_eq!(a.id, artifact.id);
        },
        s => panic!("Unexpected agent state: {:?}", s),
    }

    // 7. Simulate Tool Execution (Write Artifact to Disk)
    let write_tool = WriteFile::new(&sandbox_root);
    let tool_result = write_tool.execute(json!({
        "path": "src/hello.rs",
        "content": artifact.content,
        "create_dirs": true
    })).await.expect("Tool execution failed");

    assert!(tool_result["success"].as_bool().unwrap());

    // 8. Verify File Output
    let file_content = std::fs::read_to_string(sandbox_root.join("src/hello.rs"))
        .expect("Failed to read output file");
    assert!(file_content.contains("println!(\"Hello\")"));

    // 9. Approve Artifact
    let final_artifact = agent.approve().expect("Approval failed");
    assert_eq!(final_artifact.id, artifact.id);
    
    assert!(matches!(agent.state(), zed42_agents::blue::AgentState::Done(_)));
}
