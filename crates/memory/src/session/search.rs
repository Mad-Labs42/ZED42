//! Full-text search operations

use super::database::SessionMemory;
use super::types::{EntryType, SessionEntry};
use anyhow::Result;
use rusqlite::params;
use uuid::Uuid;

impl SessionMemory {
    /// Full-text search across session entries
    ///
    /// # Arguments
    /// - `query` - FTS5 query string
    /// - `limit` - Maximum number of results
    ///
    /// # Returns
    /// Vector of matching entries ordered by relevance
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SessionEntry>> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT e.id, e.session_id, e.entry_type, e.content, e.timestamp, e.metadata
             FROM entries_fts fts
             JOIN entries e ON e.rowid = fts.rowid
             WHERE entries_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )?;

        let entries = stmt
            .query_map(params![query, limit as i64], |row| {
                Ok(SessionEntry {
                    id: row.get(0)?,
                    session_id: row.get::<_, String>(1)?.parse().unwrap_or_default(), // Fallback for invalid UUID
                    entry_type: EntryType::parse(&row.get::<_, String>(2)?),
                    content: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or(serde_json::Value::Null), // Fallback
                    timestamp: row.get(4)?,
                    metadata: row
                        .get::<_, Option<String>>(5)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                })
            })?
            .filter_map(|r: std::result::Result<SessionEntry, _>| r.ok())
            .collect();

        Ok(entries)
    }

    /// Get recent entries of a specific type
    ///
    /// # Arguments
    /// - `entry_type` - Filter by entry type (None for all)
    /// - `limit` - Maximum number of results
    pub fn get_recent(
        &self,
        entry_type: Option<EntryType>,
        limit: usize,
    ) -> Result<Vec<SessionEntry>> {
        let conn = self.conn.lock();

        let (query, params): (&str, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(et) = entry_type {
            (
                "SELECT id, session_id, entry_type, content, timestamp, metadata
                 FROM entries
                 WHERE entry_type = ?1
                 ORDER BY timestamp DESC
                 LIMIT ?2",
                vec![Box::new(et.to_string()), Box::new(limit as i64)],
            )
        } else {
            (
                "SELECT id, session_id, entry_type, content, timestamp, metadata
                 FROM entries
                 ORDER BY timestamp DESC
                 LIMIT ?1",
                vec![Box::new(limit as i64)],
            )
        };

        let mut stmt = conn.prepare(query)?;

        let param_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|p| p.as_ref() as &dyn rusqlite::ToSql).collect();

        let entries = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(SessionEntry {
                    id: row.get(0)?,
                    session_id: row.get::<_, String>(1)?.parse().unwrap_or_default(),
                    entry_type: EntryType::parse(&row.get::<_, String>(2)?),
                    content: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or(serde_json::Value::Null),
                    timestamp: row.get(4)?,
                    metadata: row
                        .get::<_, Option<String>>(5)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                })
            })?
            .filter_map(|r: std::result::Result<SessionEntry, _>| r.ok())
            .collect();

        Ok(entries)
    }
}
