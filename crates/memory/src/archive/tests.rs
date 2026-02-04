//! Archive memory tests

use super::*;
use serde_json::json;
use tempfile::TempDir;
use uuid::Uuid;

fn create_test_archive() -> (ArchiveMemory, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let archive = ArchiveMemory::new(temp_dir.path(), "test_project").unwrap();
    (archive, temp_dir)
}

fn create_test_entry(entry_type: &str, timestamp: i64) -> ArchiveEntry {
    ArchiveEntry {
        id: Uuid::new_v4().to_string(),
        source_tier: "session".to_string(),
        entry_type: entry_type.to_string(),
        content: json!({"text": format!("test content for {}", entry_type)}),
        timestamp,
        archived_at: chrono::Utc::now().timestamp(),
        metadata: None,
    }
}

#[test]
fn test_archive_and_get() {
    let (archive, _temp) = create_test_archive();

    let entry = create_test_entry("user_message", 1000);
    let entry_id = entry.id.clone();

    archive.archive(entry).unwrap();

    let retrieved = archive.get(&entry_id).unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().entry_type, "user_message");
}

#[test]
fn test_batch_archive() {
    let (archive, _temp) = create_test_archive();

    let entries: Vec<ArchiveEntry> = (0..10)
        .map(|i| create_test_entry("data", 1000 + i))
        .collect();

    let count = archive.archive_batch(entries).unwrap();
    assert_eq!(count, 10);

    let stats = archive.stats().unwrap();
    assert_eq!(stats.total_entries, 10);
}

#[test]
fn test_time_range_query() {
    let (archive, _temp) = create_test_archive();

    // Archive entries with different timestamps
    for i in 0..5 {
        let entry = create_test_entry("message", 1000 + (i * 100));
        archive.archive(entry).unwrap();
    }

    let result = archive
        .query(ArchiveQuery::TimeRange {
            start_timestamp: 1100,
            end_timestamp: 1300,
            entry_type: None,
        })
        .unwrap();

    assert_eq!(result.entries.len(), 3); // Entries at 1100, 1200, 1300
}

#[test]
fn test_search_query() {
    let (archive, _temp) = create_test_archive();

    let mut entry1 = create_test_entry("message", 1000);
    entry1.content = json!({"text": "find this content"});
    archive.archive(entry1).unwrap();

    let mut entry2 = create_test_entry("message", 1100);
    entry2.content = json!({"text": "other content"});
    archive.archive(entry2).unwrap();

    let result = archive
        .query(ArchiveQuery::Search {
            query_text: "find this".to_string(),
            start_timestamp: None,
            end_timestamp: None,
            limit: 10,
        })
        .unwrap();

    assert_eq!(result.entries.len(), 1);
}

#[test]
fn test_prune() {
    let (archive, _temp) = create_test_archive();

    // Archive entries
    for i in 0..10 {
        let entry = create_test_entry("data", 1000 + (i * 100));
        archive.archive(entry).unwrap();
    }

    let stats_before = archive.stats().unwrap();
    assert_eq!(stats_before.total_entries, 10);

    // Prune entries before timestamp 1500
    let deleted = archive.prune(1500).unwrap();
    assert_eq!(deleted, 5);

    let stats_after = archive.stats().unwrap();
    assert_eq!(stats_after.total_entries, 5);
}

#[test]
fn test_stats() {
    let (archive, _temp) = create_test_archive();

    let entry1 = create_test_entry("message", 1000);
    let entry2 = create_test_entry("message", 2000);

    archive.archive(entry1).unwrap();
    archive.archive(entry2).unwrap();

    let stats = archive.stats().unwrap();
    assert_eq!(stats.total_entries, 2);
    assert_eq!(stats.oldest_timestamp, Some(1000));
    assert_eq!(stats.newest_timestamp, Some(2000));
}

#[test]
fn test_parquet_export_import() {
    let (archive, temp_dir) = create_test_archive();

    // Archive some entries
    for i in 0..5 {
        let entry = create_test_entry("data", 1000 + i);
        archive.archive(entry).unwrap();
    }

    // Export to Parquet
    let parquet_path = temp_dir.path().join("export.parquet");
    archive.export_to_parquet(&parquet_path).unwrap();

    assert!(parquet_path.exists());

    // Create new archive and import
    let (archive2, _temp2) = create_test_archive();
    let imported = archive2.import_from_parquet(&parquet_path).unwrap();

    assert_eq!(imported, 5);

    let stats = archive2.stats().unwrap();
    assert_eq!(stats.total_entries, 5);
}
