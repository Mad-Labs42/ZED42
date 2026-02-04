//! Archive memory type definitions

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Archive entry - generic storage for old data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub id: String,
    pub source_tier: String, // "session" or "knowledge_graph"
    pub entry_type: String,
    pub content: serde_json::Value,
    pub timestamp: i64,
    pub archived_at: i64,
    pub metadata: Option<serde_json::Value>,
}

/// Query specification for archive
#[derive(Debug, Clone)]
pub enum ArchiveQuery {
    /// Time range query
    TimeRange {
        start_timestamp: i64,
        end_timestamp: i64,
        entry_type: Option<String>,
    },
    /// Aggregation query
    Aggregate {
        metric: AggregateMetric,
        group_by: Option<String>,
        start_timestamp: i64,
        end_timestamp: i64,
    },
    /// Full-text search in archived content
    Search {
        query_text: String,
        start_timestamp: Option<i64>,
        end_timestamp: Option<i64>,
        limit: usize,
    },
}

/// Aggregate metrics
#[derive(Debug, Clone)]
pub enum AggregateMetric {
    Count,
    CountByType,
    CountByDay,
    CountByHour,
}

/// Query result from archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub entries: Vec<ArchiveEntry>,
    pub total_count: usize,
    pub query_time_ms: u64,
}

/// Archive statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveStats {
    pub total_entries: usize,
    pub oldest_timestamp: Option<i64>,
    pub newest_timestamp: Option<i64>,
    pub total_size_bytes: u64,
    pub db_path: PathBuf,
}
