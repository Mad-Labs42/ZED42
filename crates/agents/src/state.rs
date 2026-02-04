//! Agent State Safety
//! 
//! Provides RAII wrappers to ensure Agents never get stuck in a "Working" state
//! if a panic or early return occurs during execution.

use super::Agent;
use zed42_core::types::AgentStatus;

/// RAII Guard for Agent state
///
/// Automatically sets the agent's status to `Failed` if the guard is dropped
/// while the agent is still in the `Working` state.
///
/// # Example
/// ```rust
/// {
///     let mut agent = Agent::new(...);
///     let _guard = StateGuard::new(&mut agent);
///     // ... do work ...
///     // panic!() -> Guard drops -> Status = Failed
/// } // Guard drops -> Status = Idle (if completed) or Failed
/// ```
pub struct StateGuard<'a> {
    agent: &'a mut Agent,
    completed: bool,
}

impl<'a> StateGuard<'a> {
    /// Create a new state guard for an agent
    pub fn new(agent: &'a mut Agent) -> Self {
        Self {
            agent,
            completed: false,
        }
    }

    /// Mark the work as successfully completed
    ///
    /// The guard will not override the status if marked completed.
    pub fn mark_completed(&mut self) {
        self.completed = true;
    }
}

impl<'a> Drop for StateGuard<'a> {
    fn drop(&mut self) {
        if !self.completed && self.agent.status == AgentStatus::Working {
            self.agent.status = AgentStatus::Failed("Operation aborted unexpectedly (panic or drop)".to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zed42_core::types::{AgentId, Team};
    use crate::{Agent, AgentType};

    #[test]
    fn test_guard_sets_failed_on_drop() {
        let mut agent = Agent::new(AgentType::FeatureImplementer, None);
        agent.status = AgentStatus::Working;

        {
            let _guard = StateGuard::new(&mut agent);
            // Scope ends, guard drops
        }

        // Should be Failed because mark_completed was not called
        match agent.status {
            AgentStatus::Failed(ref msg) => assert!(msg.contains("aborted")),
            _ => panic!("Agent should be in Failed state"),
        }
    }

    #[test]
    fn test_guard_respects_completion() {
        let mut agent = Agent::new(AgentType::FeatureImplementer, None);
        agent.status = AgentStatus::Working;

        {
            let mut guard = StateGuard::new(&mut agent);
            guard.mark_completed();
            // Scope ends
        }

        // Should remain Working (or whatever explicit state was set, guard does nothing)
        assert_eq!(agent.status, AgentStatus::Working);
    }
}
