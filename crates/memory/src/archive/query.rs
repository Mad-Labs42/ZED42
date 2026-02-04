//! Archive query operations

use super::database::ArchiveMemory;
use super::types::{AggregateMetric, ArchiveEntry, ArchiveQuery, QueryResult};
use anyhow::{Context, Result};
use duckdb::params;
use std::time::Instant;

impl ArchiveMemory {
    /// Query the archive
    ///
    /// # Arguments
    /// - `query` - Query specification
    ///
    /// # Returns
    /// Query results with timing information
    pub fn query(&self, query: ArchiveQuery) -> Result<QueryResult> {
        let start = Instant::now();

        let entries = match query {
            ArchiveQuery::TimeRange {
                start_timestamp,
                end_timestamp,
                entry_type,
            } => self.query_time_range(start_timestamp, end_timestamp, entry_type.as_deref())?,

            ArchiveQuery::Aggregate {
                metric,
                group_by,
                start_timestamp,
                end_timestamp,
            } => {
                // For aggregate queries, return empty entries with metadata in the count
                self.query_aggregate(metric, group_by, start_timestamp, end_timestamp)?;
                Vec::new()
            }

            ArchiveQuery::Search {
                query_text,
                start_timestamp,
                end_timestamp,
                limit,
            } => self.query_search(&query_text, start_timestamp, end_timestamp, limit)?,
        };

        let query_time_ms = start.elapsed().as_millis() as u64;

        Ok(QueryResult {
            total_count: entries.len(),
            entries,
            query_time_ms,
        })
    }

    /// Query entries by time range
    fn query_time_range(
        &self,
        start_timestamp: i64,
        end_timestamp: i64,
        entry_type: Option<&str>,
    ) -> Result<Vec<ArchiveEntry>> {
        let conn = self.conn.lock().unwrap();

        let (query, params_vec): (&str, Vec<Box<dyn duckdb::ToSql>>) = if let Some(et) = entry_type {
            (
                "SELECT id, source_tier, entry_type, content, timestamp, archived_at, metadata
                 FROM archive_entries
                 WHERE timestamp BETWEEN ? AND ? AND entry_type = ?
                 ORDER BY timestamp DESC",
                vec![
                    Box::new(start_timestamp),
                    Box::new(end_timestamp),
                    Box::new(et.to_string()),
                ],
            )
        } else {
            (
                "SELECT id, source_tier, entry_type, content, timestamp, archived_at, metadata
                 FROM archive_entries
                 WHERE timestamp BETWEEN ? AND ?
                 ORDER BY timestamp DESC",
                vec![Box::new(start_timestamp), Box::new(end_timestamp)],
            )
        };

        let mut stmt = conn.prepare(query)?;

        let param_refs: Vec<&dyn duckdb::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();

        let entries = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(ArchiveEntry {
                    id: row.get(0)?,
                    source_tier: row.get(1)?,
                    entry_type: row.get(2)?,
                    content: serde_json::from_str(&row.get::<_, String>(3)?).unwrap(),
                    timestamp: row.get(4)?,
                    archived_at: row.get(5)?,
                    metadata: row
                        .get::<_, Option<String>>(6)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries)
    }

    /// Execute aggregate query
    fn query_aggregate(
        &self,
        metric: AggregateMetric,
        _group_by: Option<String>,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> Result<serde_json::Value> {
        let conn = self.conn.lock().unwrap();

        let query = match metric {
            AggregateMetric::Count => {
                "SELECT COUNT(*) as count FROM archive_entries
                 WHERE timestamp BETWEEN ? AND ?"
            }
            AggregateMetric::CountByType => {
                "SELECT entry_type, COUNT(*) as count FROM archive_entries
                 WHERE timestamp BETWEEN ? AND ?
                 GROUP BY entry_type"
            }
            AggregateMetric::CountByDay => {
                "SELECT DATE_TRUNC('day', to_timestamp(timestamp)) as day, COUNT(*) as count
                 FROM archive_entries
                 WHERE timestamp BETWEEN ? AND ?
                 GROUP BY day
                 ORDER BY day"
            }
            AggregateMetric::CountByHour => {
                "SELECT DATE_TRUNC('hour', to_timestamp(timestamp)) as hour, COUNT(*) as count
                 FROM archive_entries
                 WHERE timestamp BETWEEN ? AND ?
                 GROUP BY hour
                 ORDER BY hour"
            }
        };

        let mut stmt = conn.prepare(query)?;

        // Simple implementation - just return first row for Count metric
        let result = stmt
            .query_row(params![start_timestamp, end_timestamp], |row| {
                row.get::<_, i64>(0)
            })
            .context("Failed to execute aggregate query")?;

        Ok(serde_json::json!({ "count": result }))
    }

    /// Search archived content
    fn query_search(
        &self,
        query_text: &str,
        start_timestamp: Option<i64>,
        end_timestamp: Option<i64>,
        limit: usize,
    ) -> Result<Vec<ArchiveEntry>> {
        let conn = self.conn.lock().unwrap();

        let (query, params_vec): (&str, Vec<Box<dyn duckdb::ToSql>>) = match (start_timestamp, end_timestamp) {
            (Some(start), Some(end)) => (
                "SELECT id, source_tier, entry_type, content, timestamp, archived_at, metadata
                 FROM archive_entries
                 WHERE content::VARCHAR LIKE ? AND timestamp BETWEEN ? AND ?
                 ORDER BY timestamp DESC
                 LIMIT ?",
                vec![
                    Box::new(format!("%{}%", query_text)),
                    Box::new(start),
                    Box::new(end),
                    Box::new(limit as i64),
                ],
            ),
            _ => (
                "SELECT id, source_tier, entry_type, content, timestamp, archived_at, metadata
                 FROM archive_entries
                 WHERE content::VARCHAR LIKE ?
                 ORDER BY timestamp DESC
                 LIMIT ?",
                vec![
                    Box::new(format!("%{}%", query_text)),
                    Box::new(limit as i64),
                ],
            ),
        };

        let mut stmt = conn.prepare(query)?;

        let param_refs: Vec<&dyn duckdb::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();

        let entries = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(ArchiveEntry {
                    id: row.get(0)?,
                    source_tier: row.get(1)?,
                    entry_type: row.get(2)?,
                    content: serde_json::from_str(&row.get::<_, String>(3)?).unwrap(),
                    timestamp: row.get(4)?,
                    archived_at: row.get(5)?,
                    metadata: row
                        .get::<_, Option<String>>(6)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries)
    }
}
