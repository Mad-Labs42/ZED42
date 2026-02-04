//! Core session database operations

use super::types::{EntryType, SessionEntry, SessionStats};
use zed42_core::types::SessionId;
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use parking_lot::Mutex;
use std::sync::Arc;
use uuid::Uuid;

/// Session Memory - Tier 2 warm storage
///
/// Provides persistent session storage with full-text search capabilities.
pub struct SessionMemory {
    pub(crate) conn: Arc<Mutex<Connection>>,
    pub(crate) session_id: SessionId,
    pub(crate) db_path: PathBuf,
}

impl SessionMemory {
    /// Create or open a session memory database
    ///
    /// # Arguments
    /// - `session_id` - Unique session identifier
    /// - `data_dir` - Directory for session databases
    ///
    /// # Errors
    /// Returns error if database cannot be created/opened
    pub fn new(session_id: SessionId, data_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(data_dir)
            .context("Failed to create session data directory")?;

        let db_path = data_dir.join(format!("session_{}.db", session_id));
        let conn = Connection::open(&db_path)
            .context("Failed to open session database")?;

        // Enable WAL mode for better concurrency
        // Enable WAL mode for better concurrency and safety
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA busy_timeout = 5000;
             PRAGMA cache_size = -64000;
             PRAGMA temp_store = MEMORY;"
        ).map_err(|e| {
            // Check for OS Error 112 (Disk Full)
            if e.to_string().contains("Os { code: 112") {
                anyhow::anyhow!("CRITICAL: Disk Full (Error 112). Entering Read-Only Mode. {}", e)
            } else {
                anyhow::anyhow!("Failed to set SQLite pragmas: {}", e)
            }
        })?;

        let memory = Self {
            conn: Arc::new(Mutex::new(conn)),
            session_id,
            db_path,
        };

        memory.initialize_schema()?;

        Ok(memory)
    }

    /// Initialize database schema
    pub(crate) fn initialize_schema(&self) -> Result<()> {
        let conn = self.conn.lock();

        // Main entries table
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS entries (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                entry_type TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                metadata TEXT,
                FOREIGN KEY(session_id) REFERENCES sessions(id)
            );
            CREATE INDEX IF NOT EXISTS idx_entries_timestamp ON entries(timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_entries_type ON entries(entry_type);

            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                metadata TEXT
            );

            INSERT OR IGNORE INTO sessions (id, created_at, updated_at, metadata)
            VALUES (?1, ?2, ?2, NULL);",
        ).context("Failed to create base schema")?;

        // FTS5 virtual table for full-text search
        conn.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
                id UNINDEXED,
                entry_type UNINDEXED,
                content,
                content=entries,
                content_rowid=rowid
            );

            CREATE TRIGGER IF NOT EXISTS entries_ai AFTER INSERT ON entries BEGIN
                INSERT INTO entries_fts(rowid, id, entry_type, content)
                VALUES (new.rowid, new.id, new.entry_type, new.content);
            END;

            CREATE TRIGGER IF NOT EXISTS entries_ad AFTER DELETE ON entries BEGIN
                DELETE FROM entries_fts WHERE rowid = old.rowid;
            END;

            CREATE TRIGGER IF NOT EXISTS entries_au AFTER UPDATE ON entries BEGIN
                DELETE FROM entries_fts WHERE rowid = old.rowid;
                INSERT INTO entries_fts(rowid, id, entry_type, content)
                VALUES (new.rowid, new.id, new.entry_type, new.content);
            END;",
        ).context("Failed to create FTS5 schema")?;

        // Initialize session record
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT OR IGNORE INTO sessions (id, created_at, updated_at)
             VALUES (?1, ?2, ?2)",
            params![self.session_id.to_string(), now],
        ).context("Failed to initialize session")?;

        Ok(())
    }

    /// Insert an entry into session memory
    ///
    /// # Arguments
    /// - `entry_type` - Classification of the entry
    /// - `content` - Entry data
    /// - `metadata` - Optional metadata
    pub fn insert(
        &self,
        entry_type: EntryType,
        content: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().timestamp();

        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO entries (id, session_id, entry_type, content, timestamp, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &id,
                self.session_id.to_string(),
                entry_type.to_string(),
                serde_json::to_string(&content)?,
                timestamp,
                metadata.as_ref().map(|m| serde_json::to_string(m).ok()).flatten(),
            ],
        ).context("Failed to insert entry")?;

        // Update session timestamp
        conn.execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
            params![timestamp, self.session_id.to_string()],
        )?;

        Ok(id)
    }

    /// Get an entry by ID
    pub fn get(&self, id: &str) -> Result<Option<SessionEntry>> {
        let conn = self.conn.lock();

        let entry = conn
            .query_row(
                "SELECT id, session_id, entry_type, content, timestamp, metadata
                 FROM entries WHERE id = ?1",
                params![id],
                |row| {
                    Ok(SessionEntry {
                        id: row.get(0)?,
                        session_id: row.get::<_, String>(1)?
                            .parse()
                            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(e)))?,
                        entry_type: EntryType::parse(&row.get::<_, String>(2)?),
                        content: serde_json::from_str(&row.get::<_, String>(3)?)
                            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e)))?,
                        timestamp: row.get(4)?,
                        metadata: row
                            .get::<_, Option<String>>(5)?
                            .and_then(|s| serde_json::from_str(&s).ok()),
                    })
                },
            )
            .optional()
            .context("Failed to get entry")?;

        Ok(entry)
    }

    /// Delete entries older than specified timestamp
    pub fn prune_old_entries(&self, before_timestamp: i64) -> Result<usize> {
        let conn = self.conn.lock();

        let deleted = conn.execute(
            "DELETE FROM entries WHERE timestamp < ?1",
            params![before_timestamp],
        )?;

        Ok(deleted)
    }

    /// Get session statistics
    pub fn stats(&self) -> Result<SessionStats> {
        let conn = self.conn.lock();

        let total_entries: i64 = conn.query_row(
            "SELECT COUNT(*) FROM entries",
            [],
            |row| row.get(0),
        )?;

        let (created_at, updated_at): (i64, i64) = conn.query_row(
            "SELECT created_at, updated_at FROM sessions WHERE id = ?1",
            params![self.session_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        // Count by type
        let mut type_counts = std::collections::HashMap::new();
        let mut stmt = conn.prepare("SELECT entry_type, COUNT(*) FROM entries GROUP BY entry_type")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        for row in rows.flatten() {
            type_counts.insert(row.0, row.1 as usize);
        }

        Ok(SessionStats {
            session_id: self.session_id,
            total_entries: total_entries as usize,
            type_counts,
            created_at,
            updated_at,
            db_path: self.db_path.clone(),
        })
    }

    /// Manually checkpoint WAL file
    pub fn checkpoint(&self) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        Ok(())
    }
}

impl Drop for SessionMemory {
    fn drop(&mut self) {
        // Final checkpoint on drop
        let conn = self.conn.lock();
        let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
    }
}
