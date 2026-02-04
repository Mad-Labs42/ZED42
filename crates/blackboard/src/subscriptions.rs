//! Event subscription system

use serde::{Deserialize, Serialize};

use crate::{MessageType, AgentId};


/// Subscription handle
#[derive(Debug, Clone)]
pub struct SubscriptionHandle {
    pub id: String,
    pub query: String,
}

/// Subscription configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub agent_id: AgentId,
    pub message_type: Option<MessageType>,
    pub from_agent: Option<AgentId>,
    pub to_agent: Option<AgentId>,
}

impl Subscription {
    /// Create a subscription for all messages
    pub fn all(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            message_type: None,
            from_agent: None,
            to_agent: None,
        }
    }

    /// Subscribe to specific message type
    pub fn message_type(agent_id: AgentId, message_type: MessageType) -> Self {
        Self {
            agent_id,
            message_type: Some(message_type),
            from_agent: None,
            to_agent: None,
        }
    }

    /// Subscribe to messages from specific agent
    pub fn from_agent(agent_id: AgentId, from_agent: AgentId) -> Self {
        Self {
            agent_id,
            message_type: None,
            from_agent: Some(from_agent),
            to_agent: None,
        }
    }

    /// Subscribe to messages to specific agent
    pub fn to_agent(agent_id: AgentId, to_agent: AgentId) -> Self {
        Self {
            agent_id,
            message_type: None,
            from_agent: None,
            to_agent: Some(to_agent),
        }
    }
}
