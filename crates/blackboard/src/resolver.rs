//! State Resolver
//!
//! Collapses the event stream of the Blackboard into a localized state view.
//! Allows agents to query the 'Current Consensus' for a specific ThreadId.

use crate::types::VoxMessage;
use crate::BlackboardDb;
use uuid::Uuid;
use std::collections::HashMap;
use serde_json::Value;
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};

/// Collapsed state representing the 'Current Consensus'
#[derive(Debug, Clone)]
pub struct ConsensusState {
    pub thread_id: Uuid,
    pub values: HashMap<String, Value>,
    pub last_updated: DateTime<Utc>,
}

pub struct StateResolver;

impl StateResolver {
    /// Collapses a stream of VOX messages into a single consensus state
    pub fn collapse(thread_id: Uuid, messages: &[VoxMessage]) -> ConsensusState {
        let mut values = HashMap::new();
        let mut last_updated = DateTime::<Utc>::MIN_UTC;

        for msg in messages {
            // Extract consensus-relevant data from typed payloads
            match &msg.payload {
                zed42_core::vox::VoxPayload::ConsensusUpdate { state, .. } => {
                    // Parse state as JSON if possible, otherwise store as string
                    if let Ok(state_value) = serde_json::from_str::<Value>(state) {
                        if let Some(obj) = state_value.as_object() {
                            for (k, v) in obj {
                                values.insert(k.clone(), v.clone());
                            }
                        }
                    } else {
                        values.insert("state".to_string(), Value::String(state.clone()));
                    }
                }
                zed42_core::vox::VoxPayload::TaskAssignment { task_id, description } => {
                    values.insert("task_id".to_string(), Value::String(task_id.clone()));
                    values.insert("description".to_string(), Value::String(description.clone()));
                }
                zed42_core::vox::VoxPayload::Proposal { content } => {
                    values.insert("proposal".to_string(), Value::String(content.clone()));
                }
                zed42_core::vox::VoxPayload::Observation { content } => {
                    values.insert("observation".to_string(), Value::String(content.clone()));
                }
                zed42_core::vox::VoxPayload::SystemAlert { action, agent_id, reason } => {
                    values.insert("alert_action".to_string(), Value::String(action.clone()));
                    values.insert("alert_reason".to_string(), Value::String(reason.clone()));
                    if let Some(id) = agent_id {
                        values.insert("alert_agent_id".to_string(), Value::String(id.to_string()));
                    }
                }
                zed42_core::vox::VoxPayload::Ack { result } => {
                    values.insert("ack_result".to_string(), Value::String(result.clone()));
                }
            }
            if msg.created_at > last_updated {
                last_updated = msg.created_at;
            }
        }

        ConsensusState {
            thread_id,
            values,
            last_updated,
        }
    }

    /// Fetch and collapse all messages for a specific ThreadId from the DB
    pub async fn resolve_thread(db: &BlackboardDb, thread_id: Uuid) -> Result<ConsensusState> {
        let query = "SELECT * FROM blackboard WHERE correlation_id = $thread_id ORDER BY created_at ASC";
        let mut response = db.db().query(query)
            .bind(("thread_id", thread_id))
            .await?;
        
        let messages: Vec<VoxMessage> = response.take(0)?;
        
        Ok(Self::collapse(thread_id, &messages))
    }
}
