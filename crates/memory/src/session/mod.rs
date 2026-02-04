//! Session Memory (Tier 2) - Warm Storage
//!
//! SQLite-based session storage with full-text search (FTS5).
//! Target: 1-10ms access latency, per-session database files
//!
//! Features:
//! - Per-session SQLite database with WAL mode
//! - FTS5 full-text search for conversation history
//! - Undo/redo stack
//! - Recent decisions tracking
//! - Active agent states
//! - Auto-checkpoint every 5 minutes

mod database;
mod search;
mod types;

#[cfg(test)]
mod tests;

// Re-export public API
pub use database::SessionMemory;
pub use types::{EntryType, SessionEntry, SessionStats};
