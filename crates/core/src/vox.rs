use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Strictly typed SAGA message schemas to eliminate "Payload-Agnostic" risks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VoxPayload {
    /// Assignment of a specific task to an agent
    TaskAssignment {
        task_id: String,
        description: String,
    },
    /// A technical proposal from an agent
    Proposal {
        content: String,
    },
    /// Update to a thread's consensus state
    ConsensusUpdate {
        thread_id: Uuid,
        state: String,
    },
    /// General observation of the environment or results
    Observation {
        content: String,
    },
    /// System-level alerts from AURA or MOM
    SystemAlert {
        action: String,
        agent_id: Option<Uuid>,
        reason: String,
    },
    /// Acknowledgment of a previous message or task
    Ack {
        result: String,
    },
}

/// VOX (Versatile Orchestration eXchange) Protocol Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxMessage {
    pub sender: surrealdb::sql::Thing,
    pub target_team: String,
    pub priority: u8,
    pub correlation_id: Uuid,
    pub payload: VoxPayload,
    pub created_at: DateTime<Utc>,
}
