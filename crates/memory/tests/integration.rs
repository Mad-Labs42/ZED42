//! Integration tests for ZED42 Memory Substrate
//!
//! Tests cross-tier functionality and real-world usage scenarios

use serde_json::json;
use tempfile::TempDir;
use uuid::Uuid;
use zed42_memory::*;

/// Test full 4-tier memory substrate initialization
#[tokio::test]
async fn test_full_memory_substrate_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let session_id = Uuid::new_v4();
    let project_name = "test_project";

    let substrate = MemorySubstrate::new(temp_dir.path(), session_id, project_name)
        .await
        .unwrap();

    // Verify all tiers are available
    assert!(substrate.session().is_some());
    assert!(substrate.knowledge_graph().is_some());
    assert!(substrate.archive().is_some());
}

/// Test cross-tier query integration
#[tokio::test]
async fn test_cross_tier_query() {
    let temp_dir = TempDir::new().unwrap();
    let session_id = Uuid::new_v4();
    let project_name = "test_cross_tier";

    let substrate = MemorySubstrate::new(temp_dir.path(), session_id, project_name)
        .await
        .unwrap();

    // Insert data into different tiers
    substrate.store_working(
        "test_key".to_string(),
        json!({"tier": "working", "data": "hot cache"}),
    );

    if let Some(session) = substrate.session() {
        session
            .insert(
                session::EntryType::Data,
                json!({"tier": "session", "data": "warm storage"}),
                None,
            )
            .unwrap();
    }

    // Query across all tiers
    let results = substrate.query("test", 10).await.unwrap();

    // Should get results from multiple tiers
    assert!(!results.is_empty());

    // Verify tier metadata
    let tiers: Vec<MemoryTier> = results.iter().map(|r| r.tier).collect();
    assert!(tiers.contains(&MemoryTier::Working) || tiers.contains(&MemoryTier::Session));
}

/// Test tier 1: Working memory performance
#[test]
fn test_working_memory_performance() {
    let working = working::WorkingMemory::new();

    // Insert 1000 entries
    for i in 0..1000 {
        working.insert(
            format!("key_{}", i),
            json!({"index": i}),
            1.0,
            false,
        );
    }

    // Verify retrieval is fast
    let start = std::time::Instant::now();
    for i in 0..100 {
        let _ = working.get(&format!("key_{}", i));
    }
    let duration = start.elapsed();

    // Should be sub-millisecond per access
    assert!(duration.as_millis() < 10, "Working memory too slow");
}

/// Test tier 2: Session memory with FTS5 search
#[test]
fn test_session_memory_fts5() {
    let temp_dir = TempDir::new().unwrap();
    let session_id = Uuid::new_v4();

    let session = session::SessionMemory::new(session_id, temp_dir.path()).unwrap();

    // Insert entries with searchable content
    session
        .insert(
            session::EntryType::UserMessage,
            json!({"text": "implement feature X with Rust"}),
            None,
        )
        .unwrap();

    session
        .insert(
            session::EntryType::UserMessage,
            json!({"text": "add database migration"}),
            None,
        )
        .unwrap();

    session
        .insert(
            session::EntryType::UserMessage,
            json!({"text": "Rust implementation details"}),
            None,
        )
        .unwrap();

    // Full-text search
    let results = session.search("Rust", 10).unwrap();
    assert_eq!(results.len(), 2, "FTS5 search failed");
}

/// Test tier 3: Knowledge graph with multiple node types
#[tokio::test]
async fn test_knowledge_graph_multi_type() {
    let temp_dir = TempDir::new().unwrap();

    let kg = knowledge_graph::KnowledgeGraphMemory::new(temp_dir.path(), "test_kg", None)
        .await
        .unwrap();

    // Insert different node types
    let function_node = knowledge_graph::KnowledgeNode {
        id: Uuid::new_v4().to_string(),
        node_type: knowledge_graph::NodeType::Function,
        name: "main_function".to_string(),
        content: json!({"signature": "fn main()"}).to_string(),
        embedding: None,
        metadata: json!({}).to_string(),
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    };

    let file_node = knowledge_graph::KnowledgeNode {
        id: Uuid::new_v4().to_string(),
        node_type: knowledge_graph::NodeType::File,
        name: "main.rs".to_string(),
        content: json!({"path": "/src/main.rs"}).to_string(),
        embedding: None,
        metadata: json!({}).to_string(),
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    };

    kg.insert_node(function_node.clone()).await.unwrap();
    kg.insert_node(file_node.clone()).await.unwrap();

    // Create edge relationship
    let edge = knowledge_graph::KnowledgeEdge {
        id: Uuid::new_v4().to_string(),
        edge_type: knowledge_graph::EdgeType::Contains,
        from_id: file_node.id.clone(),
        to_id: function_node.id.clone(),
        metadata: None,
        created_at: chrono::Utc::now().timestamp(),
    };

    kg.insert_edge(edge).await.unwrap();

    // Verify retrieval
    let retrieved = kg.get_node(&function_node.id).await.unwrap();
    assert!(retrieved.is_some());
}

/// Test tier 4: Archive with time range queries
#[test]
fn test_archive_time_range_queries() {
    let temp_dir = TempDir::new().unwrap();

    let archive = archive::ArchiveMemory::new(temp_dir.path(), "test_archive").unwrap();

    let base_time = chrono::Utc::now().timestamp();

    // Insert entries across time range
    for i in 0..10 {
        let entry = archive::ArchiveEntry {
            id: Uuid::new_v4().to_string(),
            source_tier: "session".to_string(),
            entry_type: "data".to_string(),
            content: json!({"index": i}),
            timestamp: base_time + (i * 3600), // 1 hour apart
            archived_at: chrono::Utc::now().timestamp(),
            metadata: None,
        };

        archive.archive(entry).unwrap();
    }

    // Query specific time range
    let results = archive
        .query(archive::ArchiveQuery::TimeRange {
            start_timestamp: base_time + 3600,
            end_timestamp: base_time + (5 * 3600),
            entry_type: None,
        })
        .unwrap();

    assert_eq!(results.entries.len(), 5, "Time range query failed");
}

/// Test archive Parquet export/import
#[test]
fn test_archive_parquet_roundtrip() {
    let temp_dir = TempDir::new().unwrap();

    let archive = archive::ArchiveMemory::new(temp_dir.path(), "test_parquet").unwrap();

    // Insert test data
    for i in 0..5 {
        let entry = archive::ArchiveEntry {
            id: format!("entry_{}", i),
            source_tier: "session".to_string(),
            entry_type: "test".to_string(),
            content: json!({"data": i}),
            timestamp: 1000 + i,
            archived_at: chrono::Utc::now().timestamp(),
            metadata: None,
        };

        archive.archive(entry).unwrap();
    }

    // Export to Parquet
    let parquet_path = temp_dir.path().join("export.parquet");
    archive.export_to_parquet(&parquet_path).unwrap();

    assert!(parquet_path.exists(), "Parquet file not created");

    // Import into new archive
    let archive2 = archive::ArchiveMemory::new(temp_dir.path(), "test_import").unwrap();
    let imported_count = archive2.import_from_parquet(&parquet_path).unwrap();

    assert_eq!(imported_count, 5, "Parquet import count mismatch");

    let stats = archive2.stats().unwrap();
    assert_eq!(stats.total_entries, 5, "Import verification failed");
}

/// Test data migration across tiers
#[tokio::test]
async fn test_tier_data_migration() {
    let temp_dir = TempDir::new().unwrap();
    let session_id = Uuid::new_v4();

    let session = session::SessionMemory::new(session_id, temp_dir.path()).unwrap();
    let archive = archive::ArchiveMemory::new(temp_dir.path(), "migration_test").unwrap();

    // Insert data in session
    let entry_id = session
        .insert(
            session::EntryType::Data,
            json!({"message": "old data to archive"}),
            None,
        )
        .unwrap();

    // Retrieve from session
    let session_entry = session.get(&entry_id).unwrap().unwrap();

    // Migrate to archive
    let archive_entry = archive::ArchiveEntry {
        id: session_entry.id.clone(),
        source_tier: "session".to_string(),
        entry_type: "data".to_string(),
        content: session_entry.content,
        timestamp: session_entry.timestamp,
        archived_at: chrono::Utc::now().timestamp(),
        metadata: session_entry.metadata,
    };

    archive.archive(archive_entry).unwrap();

    // Verify in archive
    let retrieved = archive.get(&entry_id).unwrap();
    assert!(retrieved.is_some(), "Migration failed");
}

/// Test concurrent access to working memory
#[tokio::test]
async fn test_concurrent_working_memory_access() {
    use std::sync::Arc;

    let working = Arc::new(working::WorkingMemory::new());

    let mut handles = vec![];

    // Spawn 10 concurrent tasks
    for i in 0..10 {
        let working_clone = working.clone();
        let handle = tokio::spawn(async move {
            for j in 0..100 {
                working_clone.insert(
                    format!("key_{}_{}", i, j),
                    json!({"task": i, "item": j}),
                    1.0,
                    false,
                );
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify data integrity
    let stats = working.stats();
    assert_eq!(stats.total_entries, 1000, "Concurrent access failed");
}

/// Test memory substrate with minimal tiers
#[test]
fn test_working_only_substrate() {
    let substrate = MemorySubstrate::working_only();

    // Should work with only working memory
    substrate.store_working("test".to_string(), json!({"data": "test"}));

    let value = substrate.working().get("test");
    assert!(value.is_some());

    // Other tiers should be None
    assert!(substrate.session().is_none());
    assert!(substrate.knowledge_graph().is_none());
    assert!(substrate.archive().is_none());
}

/// Test error handling across tiers
#[tokio::test]
async fn test_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let session_id = Uuid::new_v4();

    let session = session::SessionMemory::new(session_id, temp_dir.path()).unwrap();

    // Try to get non-existent entry
    let result = session.get("non_existent_id").unwrap();
    assert!(result.is_none());

    // Prune with future timestamp should delete nothing
    let future_time = chrono::Utc::now().timestamp() + 10000;
    let deleted = session.prune_old_entries(future_time).unwrap();
    assert_eq!(deleted, 0);
}

/// Test knowledge graph search modes
#[tokio::test]
async fn test_knowledge_graph_search_modes() {
    let temp_dir = TempDir::new().unwrap();

    let kg = knowledge_graph::KnowledgeGraphMemory::new(temp_dir.path(), "search_test", None)
        .await
        .unwrap();

    // Insert test nodes
    for i in 0..5 {
        let node = knowledge_graph::KnowledgeNode {
            id: format!("node_{}", i),
            node_type: knowledge_graph::NodeType::Function,
            name: format!("test_function_{}", i),
            content: json!({"code": format!("fn test_{}() {{}}", i)}).to_string(),
            embedding: None,
            metadata: json!({}).to_string(),
            created_at: 1000 + i,
            updated_at: 1000 + i,
        };

        kg.insert_node(node).await.unwrap();
    }

    // Test semantic search
    let semantic_results = kg
        .search(knowledge_graph::SearchQuery::Semantic {
            query_text: "test_function".to_string(),
            top_k: 3,
            node_types: None,
        })
        .await
        .unwrap();

    assert!(!semantic_results.is_empty(), "Semantic search failed");

    // Test temporal search
    let temporal_results = kg
        .search(knowledge_graph::SearchQuery::Temporal {
            target_timestamp: 1003,
            node_types: None,
        })
        .await
        .unwrap();

    assert!(!temporal_results.is_empty(), "Temporal search failed");
}
