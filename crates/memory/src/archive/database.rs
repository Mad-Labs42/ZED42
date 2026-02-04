//! Core archive database operations

use super::types::{ArchiveEntry, ArchiveStats};
use anyhow::{Context, Result};
use duckdb::{params, Connection, OptionalExt};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Archive Memory - Tier 4 cold storage
///
/// Provides long-term storage with analytical query capabilities.
pub struct ArchiveMemory {
    pub(crate) conn: Arc<Mutex<Connection>>,
    pub(crate) db_path: PathBuf,
}

impl ArchiveMemory {
    /// Create or open an archive memory database
    ///
    /// # Arguments
    /// - `data_dir` - Directory for archive database
    /// - `project_name` - Project identifier
    ///
    /// # Errors
    /// Returns error if database cannot be created/opened
    pub fn new(data_dir: &Path, project_name: &str) -> Result<Self> {
        std::fs::create_dir_all(data_dir)
            .context("Failed to create archive data directory")?;

        let db_path = data_dir.join(format!("archive_{}.db", project_name));
        let conn = Connection::open(&db_path)
            .context("Failed to open archive database")?;

        let memory = Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        };

        memory.initialize_schema()?;

        Ok(memory)
    }

    /// Initialize database schema
    pub(crate) fn initialize_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Create main archive table
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS archive_entries (
                id VARCHAR PRIMARY KEY,
                source_tier VARCHAR NOT NULL,
                entry_type VARCHAR NOT NULL,
                content JSON NOT NULL,
                timestamp BIGINT NOT NULL,
                archived_at BIGINT NOT NULL,
                metadata JSON
            );

            CREATE INDEX IF NOT EXISTS idx_timestamp ON archive_entries(timestamp);
            CREATE INDEX IF NOT EXISTS idx_archived_at ON archive_entries(archived_at);
            CREATE INDEX IF NOT EXISTS idx_entry_type ON archive_entries(entry_type);
            CREATE INDEX IF NOT EXISTS idx_source_tier ON archive_entries(source_tier);",
        ).context("Failed to create archive schema")?;

        Ok(())
    }

    /// Archive an entry
    ///
    /// # Arguments
    /// - `entry` - Entry to archive
    pub fn archive(&self, entry: ArchiveEntry) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO archive_entries
             (id, source_tier, entry_type, content, timestamp, archived_at, metadata)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                &entry.id,
                &entry.source_tier,
                &entry.entry_type,
                serde_json::to_string(&entry.content)?,
                entry.timestamp,
                entry.archived_at,
                entry.metadata.as_ref().map(|m| serde_json::to_string(m).ok()).flatten(),
            ],
        ).context("Failed to archive entry")?;

        Ok(())
    }

    /// Archive multiple entries in a batch
    pub fn archive_batch(&self, entries: Vec<ArchiveEntry>) -> Result<usize> {
        let conn = self.conn.lock().unwrap();

        let tx = conn.unchecked_transaction()?;

        for entry in &entries {
            tx.execute(
                "INSERT INTO archive_entries
                 (id, source_tier, entry_type, content, timestamp, archived_at, metadata)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
                params![
                    &entry.id,
                    &entry.source_tier,
                    &entry.entry_type,
                    serde_json::to_string(&entry.content)?,
                    entry.timestamp,
                    entry.archived_at,
                    entry.metadata.as_ref().map(|m| serde_json::to_string(m).ok()).flatten(),
                ],
            )?;
        }

        tx.commit()?;

        Ok(entries.len())
    }

    /// Get an entry by ID
    pub fn get(&self, id: &str) -> Result<Option<ArchiveEntry>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, source_tier, entry_type, content, timestamp, archived_at, metadata
             FROM archive_entries WHERE id = ?",
        )?;

        let entry = stmt
            .query_row(params![id], |row| {
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
            })
            .optional()
            .context("Failed to get archived entry")?;

        Ok(entry)
    }

    /// Delete entries older than specified timestamp
    pub fn prune(&self, before_timestamp: i64) -> Result<usize> {
        let conn = self.conn.lock().unwrap();

        let deleted = conn.execute(
            "DELETE FROM archive_entries WHERE timestamp < ?",
            params![before_timestamp],
        )?;

        Ok(deleted)
    }

    /// Get archive statistics
    pub fn stats(&self) -> Result<ArchiveStats> {
        let conn = self.conn.lock().unwrap();

        let total_entries: usize = conn.query_row(
            "SELECT COUNT(*) FROM archive_entries",
            [],
            |row| row.get(0),
        )?;

        let (oldest, newest): (Option<i64>, Option<i64>) = conn.query_row(
            "SELECT MIN(timestamp), MAX(timestamp) FROM archive_entries",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        // Estimate size (rough approximation)
        let db_metadata = std::fs::metadata(&self.db_path)
            .context("Failed to get database file metadata")?;
        let total_size_bytes = db_metadata.len();

        Ok(ArchiveStats {
            total_entries,
            oldest_timestamp: oldest,
            newest_timestamp: newest,
            total_size_bytes,
            db_path: self.db_path.clone(),
        })
    }

    /// Export to Parquet file
    ///
    /// # Arguments
    /// - `output_path` - Path for Parquet export
    pub fn export_to_parquet(&self, output_path: &Path) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let query = format!(
            "COPY archive_entries TO '{}' (FORMAT PARQUET)",
            output_path.display()
        );

        conn.execute(&query, [])
            .context("Failed to export to Parquet")?;

        Ok(())
    }

    /// Import from Parquet file
    ///
    /// # Arguments
    /// - `input_path` - Path to Parquet file
    pub fn import_from_parquet(&self, input_path: &Path) -> Result<usize> {
        let conn = self.conn.lock().unwrap();

        let query = format!(
            "INSERT INTO archive_entries SELECT * FROM read_parquet('{}')",
            input_path.display()
        );

        let imported = conn.execute(&query, [])
            .context("Failed to import from Parquet")?;

        Ok(imported)
    }
}
