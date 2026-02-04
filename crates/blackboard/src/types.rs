//! Blackboard type definitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

pub use zed42_core::AgentId;
pub use zed42_core::MessageType;

// Re-export VOX Protocol from Core
pub use zed42_core::vox::VoxMessage;

/// AURA Vitality pulse for agent health monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuraPulse {
    pub last_pulse: DateTime<Utc>,
    pub status: String,
}

/// Blackboard statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackboardStats {
    pub total_messages: usize,
    pub total_state_entries: usize,
    pub total_decisions: usize,
    pub db_path: PathBuf,
}

/// Key for state storage
pub type StateKey = String;

/// Entry in the blackboard state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEntry {
    pub key: StateKey,
    pub value: serde_json::Value,
    pub owner_agent: AgentId,
    pub timestamp: i64,
    pub version: i32,
}

/// Node representing a decision in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionNode {
    pub id: String,
    pub decision_type: String,
    pub description: String,
    pub made_by: AgentId,
    pub rationale: serde_json::Value,
    pub alternatives_considered: Vec<String>,
    pub timestamp: i64,
    pub parent_decision: Option<String>,
}

/// Filter for message queries
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageFilter {
    pub message_type: Option<MessageType>,
    pub from_agent: Option<AgentId>,
    pub to_agent: Option<AgentId>,
    pub since_timestamp: Option<i64>,
    pub limit: Option<usize>,
}
