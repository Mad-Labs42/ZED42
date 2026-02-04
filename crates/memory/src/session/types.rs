//! Session memory type definitions

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;
use zed42_core::types::SessionId;


/// Session memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEntry {
    pub id: String,
    pub session_id: SessionId,
    pub entry_type: EntryType,
    pub content: serde_json::Value,
    pub timestamp: i64,
    pub metadata: Option<serde_json::Value>,
}

/// Entry type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryType {
    /// User message or intent
    UserMessage,
    /// Agent response
    AgentResponse,
    /// System decision
    Decision,
    /// Agent state snapshot
    AgentState,
    /// Undo/redo action
    Action,
    /// Arbitrary data
    Data,
}

impl std::fmt::Display for EntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryType::UserMessage => write!(f, "user_message"),
            EntryType::AgentResponse => write!(f, "agent_response"),
            EntryType::Decision => write!(f, "decision"),
            EntryType::AgentState => write!(f, "agent_state"),
            EntryType::Action => write!(f, "action"),
            EntryType::Data => write!(f, "data"),
        }
    }
}

impl EntryType {
    /// Parse entry type from string
    pub(crate) fn parse(s: &str) -> Self {
        match s {
            "user_message" => EntryType::UserMessage,
            "agent_response" => EntryType::AgentResponse,
            "decision" => EntryType::Decision,
            "agent_state" => EntryType::AgentState,
            "action" => EntryType::Action,
            _ => EntryType::Data,
        }
    }
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub session_id: SessionId,
    pub total_entries: usize,
    pub type_counts: std::collections::HashMap<String, usize>,
    pub created_at: i64,
    pub updated_at: i64,
    pub db_path: PathBuf,
}
