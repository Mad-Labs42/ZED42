//! Integration tests for the Reactive Blackboard
//!
//! Verified basic initialization and schema integrity.

use zed42_blackboard::*;
use tempfile::TempDir;
use serde_json::json;

async fn setup() -> (BlackboardDb, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let blackboard = BlackboardDb::new(
        temp_dir.path(), 
        "test_project", 
        "ws://localhost:8000"
    ).await.unwrap();
    (blackboard, temp_dir)
}

#[tokio::test]
async fn test_blackboard_initialization() {
    let (blackboard, _temp) = setup().await;
    let stats = blackboard.stats().await.unwrap();
    assert_eq!(stats.total_messages, 0);
}

#[tokio::test]
async fn test_state_persistence() {
    let (blackboard, _temp) = setup().await;
    
    let entry = StateEntry {
        key: "test_key".to_string(),
        value: json!({"status": "active"}),
        owner_agent: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now().timestamp(),
        version: 1,
    };
    
    blackboard.set_state(entry).await.unwrap();
    
    let retrieved = blackboard.get_state(&"test_key".to_string()).await.unwrap().unwrap();
    assert_eq!(retrieved.value["status"], "active");
}
