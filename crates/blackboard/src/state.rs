//! Blackboard state management

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use zed42_core::types::{SessionId, Confidence};


/// Current blackboard state for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackboardState {
    pub session_id: SessionId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub goals: Vec<String>,
    pub constraints: HashMap<String, Constraint>,
    pub confidence_threshold: Confidence,
}

/// Constraint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub name: String,
    pub description: String,
    pub rule: String,
    pub applies_to: Vec<String>,
    pub severity: ConstraintSeverity,
}

/// Constraint severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConstraintSeverity {
    Error,   // Must be satisfied
    Warning, // Should be satisfied
    Info,    // Nice to have
}

impl BlackboardState {
    pub fn new(session_id: SessionId) -> Self {
        Self {
            session_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            goals: Vec::new(),
            constraints: HashMap::new(),
            confidence_threshold: 0.7,
        }
    }

    pub fn add_goal(&mut self, goal: String) {
        self.goals.push(goal);
        self.updated_at = Utc::now();
    }

    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.insert(constraint.name.clone(), constraint);
        self.updated_at = Utc::now();
    }
}
