//! Archive Memory (Tier 4) - Cold Storage
//!
//! DuckDB-based analytical storage with Parquet columnar format.
//! Target: 100ms-1s access latency, unlimited scale
//!
//! Features:
//! - Parquet columnar storage for compression
//! - DuckDB for analytical queries
//! - Automatic archival of old data (>90 days)
//! - Time-series analytics
//! - Aggregation queries
//! - Long-term conversation history

mod database;
mod query;
mod types;

#[cfg(test)]
mod tests;

// Re-export public API
pub use database::ArchiveMemory;
pub use types::{ArchiveEntry, ArchiveQuery, ArchiveStats, QueryResult};
