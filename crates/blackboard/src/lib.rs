//! Blackboard system for agent coordination and state management

mod database;
mod graph;
mod state;
mod types;
mod mom;
mod aura;
mod resolver;

#[cfg(test)]
mod tests;

pub use database::BlackboardDb;
pub use graph::{DecisionGraph, EdgeType};
pub use mom::MOMWatcher;
pub use aura::AuraSentinel as Aura;

// Re-export core types used by blackboard via root re-exports
pub use zed42_core::{Result, Error};
pub use zed42_core::{
    AgentId, Priority, Team, ThreadId, MessageId,
    Message, MessageType
};
pub use zed42_core::types::{
    SessionId, Confidence, ArtifactId, DecisionId, TaskId
};
pub use zed42_core::AgentBehavior;

// Local types
pub use types::{
    BlackboardStats, DecisionNode, StateEntry, StateKey, 
    MessageFilter, VoxMessage, AuraPulse
};
pub use zed42_core::AgentStatus;
pub use state::{BlackboardState};
pub use resolver::{StateResolver, ConsensusState};

// Backwards compatibility
pub use zed42_core::messages::MessageTarget;

