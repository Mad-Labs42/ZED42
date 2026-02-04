//! Session memory tests

use super::*;
use serde_json::json;
use tempfile::TempDir;

fn create_test_session() -> (SessionMemory, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let session_id = zed42_core::types::SessionId::new_v4();
    let memory = SessionMemory::new(session_id, temp_dir.path()).unwrap();
    (memory, temp_dir)
}

#[test]
fn test_insert_and_get() {
    let (memory, _temp) = create_test_session();

    let content = json!({"message": "test"});
    let id = memory
        .insert(EntryType::UserMessage, content.clone(), None)
        .unwrap();

    let entry = memory.get(&id).unwrap().unwrap();
    assert_eq!(entry.content, content);
    assert_eq!(entry.entry_type, EntryType::UserMessage);
}

#[test]
fn test_full_text_search() {
    let (memory, _temp) = create_test_session();

    // Insert test entries
    memory
        .insert(
            EntryType::UserMessage,
            json!({"text": "implement authentication"}),
            None,
        )
        .unwrap();

    memory
        .insert(
            EntryType::UserMessage,
            json!({"text": "add database migration"}),
            None,
        )
        .unwrap();

    memory
        .insert(
            EntryType::UserMessage,
            json!({"text": "authentication bug fix"}),
            None,
        )
        .unwrap();

    // Search for "authentication"
    let results = memory.search("authentication", 10).unwrap();
    assert_eq!(results.len(), 2);
}

#[test]
fn test_get_recent() {
    let (memory, _temp) = create_test_session();

    // Insert multiple entries
    for i in 0..5 {
        memory
            .insert(
                EntryType::UserMessage,
                json!({"index": i}),
                None,
            )
            .unwrap();
    }

    for i in 0..3 {
        memory
            .insert(
                EntryType::Decision,
                json!({"decision": i}),
                None,
            )
            .unwrap();
    }

    let recent = memory.get_recent(None, 3).unwrap();
    assert_eq!(recent.len(), 3);

    let recent_messages = memory.get_recent(Some(EntryType::UserMessage), 10).unwrap();
    assert_eq!(recent_messages.len(), 5);
}

#[test]
fn test_prune_old_entries() {
    let (memory, _temp) = create_test_session();

    // Insert entries
    for i in 0..10 {
        memory
            .insert(EntryType::Data, json!({"index": i}), None)
            .unwrap();
    }

    let stats_before = memory.stats().unwrap();
    assert_eq!(stats_before.total_entries, 10);

    // Prune entries (use future timestamp to ensure deletion in test)
    let now = chrono::Utc::now().timestamp();
    let deleted = memory.prune_old_entries(now + 10).unwrap();
    assert!(deleted > 0);
}

#[test]
fn test_stats() {
    let (memory, _temp) = create_test_session();

    memory
        .insert(EntryType::UserMessage, json!({"msg": 1}), None)
        .unwrap();
    memory
        .insert(EntryType::Decision, json!({"dec": 1}), None)
        .unwrap();
    memory
        .insert(EntryType::Decision, json!({"dec": 2}), None)
        .unwrap();

    let stats = memory.stats().unwrap();
    assert_eq!(stats.total_entries, 3);
    assert_eq!(*stats.type_counts.get("user_message").unwrap_or(&0), 1);
    assert_eq!(*stats.type_counts.get("decision").unwrap_or(&0), 2);
}

#[test]
fn test_checkpoint() {
    let (memory, _temp) = create_test_session();

    memory
        .insert(EntryType::Data, json!({"test": 1}), None)
        .unwrap();

    memory.checkpoint().unwrap();
    // Should not panic
}
