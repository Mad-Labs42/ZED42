//! Blackboard tests

use super::*;
use serde_json::json;
use tempfile::TempDir;

async fn create_test_blackboard() -> (BlackboardDb, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    // Use dummy WS address for local tests
    let blackboard = BlackboardDb::new(temp_dir.path(), "test_project", "ws://localhost:8000")
        .await
        .unwrap();
    (blackboard, temp_dir)
}

#[tokio::test]
async fn test_db_initialization() {
    let (_blackboard, _temp) = create_test_blackboard().await;
    // Schema initialization already verified by .unwrap() in create_test_blackboard
}

#[tokio::test]
async fn test_state_management() {
    let (blackboard, _temp) = create_test_blackboard().await;

    let entry = StateEntry {
        key: "current_phase".to_string(),
        value: json!("implementation"),
        owner_agent: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now().timestamp(),
        version: 1,
    };

    blackboard.set_state(entry.clone()).await.unwrap();

    let retrieved = blackboard
        .get_state(&"current_phase".to_string())
        .await
        .unwrap();

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().value, json!("implementation"));
}

#[tokio::test]
async fn test_stats() {
    let (blackboard, _temp) = create_test_blackboard().await;

    // Add some data
    let entry = StateEntry {
        key: "key1".to_string(),
        value: json!("value1"),
        owner_agent: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now().timestamp(),
        version: 1,
    };
    blackboard.set_state(entry).await.unwrap();

    let stats = blackboard.stats().await.unwrap();
    assert_eq!(stats.total_state_entries, 1);
}

#[tokio::test]
async fn test_vox_message_serialization() {
    let msg = VoxMessage {
        sender: surrealdb::sql::Thing::from(("agent", "cortex")),
        target_team: "blue".to_string(),
        priority: 2,
        correlation_id: uuid::Uuid::new_v4(),
        payload: zed42_core::vox::VoxPayload::TaskAssignment {
            task_id: "task-123".to_string(),
            description: "research".to_string(),
        },
        created_at: chrono::Utc::now(),
    };

    let serialized = serde_json::to_string(&msg).unwrap();
    let deserialized: VoxMessage = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.target_team, "blue");
    assert_eq!(deserialized.sender.to_string(), "agent:cortex");
}

#[tokio::test]
async fn test_agent_pulse() {
    let (blackboard, _temp) = create_test_blackboard().await;
    let agent_id = uuid::Uuid::new_v4();

    // Send successful pulse
    blackboard.send_pulse(agent_id, AgentStatus::Working).await.unwrap();

    // Verify pulse record exists via stats or direct query if needed
    // For now, verification that the query didn't error is the baseline.
}

