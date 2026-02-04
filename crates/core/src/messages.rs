//! VOX (Versatile Orchestration eXchange) - Message Definitions
//!
//! Formalized, type-safe communication protocol for SAGA agents.
//! Governs intent, routing, and threading across the MOM substrate.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::{AgentId, MessageId, Priority, Team, ThreadId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageType {
    // Command Messages
    ExecuteTask { task_description: String },
    SpawnAgent { agent_type: String, toolbox: Vec<String> },
    DissolveAgent { agent_id: AgentId },

    // Query Messages
    RequestContext { query: String },
    QueryKnowledgeGraph { query: String },
    GetConstraints { scope: String },

    // Proposal Messages
    ProposeSolution { solution: String },
    SuggestRefactor { refactor_plan: String },
    IdentifyRisk { risk_description: String },

    // Verdict Messages
    ApproveChange { change_id: String, rationale: String },
    RejectProposal { proposal_id: String, reason: String },
    RequestRevision { target_id: String, requested_changes: String },

    // Notification Messages
    TaskComplete { task_id: String, result: String },
    ErrorOccurred { error: String, context: String },
    MilestoneReached { milestone: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageTarget {
    Team(Team),
    All,
    Agent(AgentId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub timestamp: DateTime<Utc>,
    pub from_agent: AgentId,
    pub to_team: MessageTarget,
    pub message_type: MessageType,
    pub thread_id: ThreadId,
    pub priority: Priority,
    pub requires_response: bool,
}

impl Message {
    pub fn new(
        from_agent: AgentId,
        to_team: MessageTarget,
        message_type: MessageType,
        priority: Priority,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            from_agent,
            to_team,
            message_type,
            thread_id: uuid::Uuid::new_v4(),
            priority,
            requires_response: false,
        }
    }

    pub fn with_thread(mut self, thread_id: ThreadId) -> Self {
        self.thread_id = thread_id;
        self
    }

    pub fn requires_response(mut self) -> Self {
        self.requires_response = true;
        self
    }
}
