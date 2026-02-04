//! Core blackboard database operations

use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use surrealdb::engine::local::{Db, Mem};
use surrealdb::{Surreal, Response};
use tokio::sync::broadcast;

use crate::{
    DecisionNode, StateEntry, StateKey, 
    MessageFilter, MOMWatcher, VoxMessage
};
use crate::types::BlackboardStats;
use zed42_core::{Message, AgentId, Team};


/// Blackboard - Living communication substrate
///
/// Provides message bus, state management, and decision tracking.
pub struct BlackboardDb {
    db: Surreal<Db>,
    mom: Arc<MOMWatcher>,
    db_path: std::path::PathBuf,
}

impl BlackboardDb {
    /// Create or open a blackboard database
    ///
    /// # Arguments
    /// - `data_dir` - Directory for the database
    /// - `project_name` - Project identifier
    /// - `ws_addr` - WebSocket address for the reactive MOM substrate (e.g., "ws://localhost:8000")
    pub async fn new(data_dir: &Path, project_name: &str, ws_addr: &str) -> Result<Self> {
        std::fs::create_dir_all(data_dir)
            .context("Failed to create blackboard data directory")?;

        let db_path = data_dir.join(format!("blackboard_{}.db", project_name));

        // Use Mem for local state, but the watcher connects via WS to the authoritative DB
        let db: Surreal<Db> = Surreal::new::<Mem>(())
            .await
            .expect("Failed to start Mem DB");

        let _: () = db.use_ns("zed42")
            .use_db(project_name)
            .await
            .expect("Failed to select namespace/database");

        let mom = Arc::new(MOMWatcher::new(
            ws_addr.to_string(),
            "zed42".to_string(),
            project_name.to_string(),
        ));

        // Start the MOM reactive loop in a background task
        let mom_clone = mom.clone();
        tokio::spawn(async move {
            mom_clone.run_loop().await;
        });

        let blackboard = Self { db, mom, db_path };

        blackboard.initialize_schema().await?;

        Ok(blackboard)
    }

    /// Initialize database schema using SurrealQL 2.0
    async fn initialize_schema(&self) -> Result<()> {
        // Define Immutable, High-Integrity Message Bus
        self.db
            .query(
                "DEFINE TABLE IF NOT EXISTS blackboard SCHEMAFULL;
                 DEFINE FIELD IF NOT EXISTS sender ON blackboard TYPE record<agent>;
                 DEFINE FIELD IF NOT EXISTS target_team ON blackboard TYPE string;
                 DEFINE FIELD IF NOT EXISTS priority ON blackboard TYPE int DEFAULT 1;
                 DEFINE FIELD IF NOT EXISTS correlation_id ON blackboard TYPE uuid;
                 DEFINE FIELD IF NOT EXISTS payload ON blackboard TYPE object;
                 DEFINE FIELD IF NOT EXISTS created_at ON blackboard TYPE datetime VALUE $before OR time::now();
                 DEFINE INDEX IF NOT EXISTS target_team_idx ON blackboard FIELDS target_team;",
            )
            .await
            .context("Failed to create blackboard schema")?;

        // Define AURA Vitality Substrate (Pulses)
        self.db
            .query(
                "DEFINE TABLE IF NOT EXISTS aura_pulses SCHEMAFULL;
                 DEFINE FIELD IF NOT EXISTS last_pulse ON aura_pulses TYPE datetime VALUE time::now();
                 DEFINE FIELD IF NOT EXISTS status ON aura_pulses TYPE string;
                 DEFINE EVENT IF NOT EXISTS monitor_ghosts ON aura_pulses WHEN $event = 'UPDATE' THEN {
                    IF (time::now() - last_pulse) > 1m {
                        UPDATE $after SET status = 'ghost';
                    };
                 };
                 DEFINE EVENT IF NOT EXISTS cleanup_old_pulses ON aura_pulses WHEN $event = 'UPDATE' THEN {
                    DELETE aura_pulses WHERE last_pulse < (time::now() - 24h);
                 };",
            )
            .await
            .context("Failed to create AURA pulse schema")?;

        // Maintain legacy tables for compatibility during transition
        self.db
            .query(
                "DEFINE TABLE IF NOT EXISTS messages SCHEMAFULL;
                 DEFINE FIELD IF NOT EXISTS id ON messages TYPE string;
                 DEFINE FIELD IF NOT EXISTS message_type ON messages TYPE string;
                 DEFINE FIELD IF NOT EXISTS from_agent ON messages TYPE string;
                 DEFINE FIELD IF NOT EXISTS to_agent ON messages TYPE option<string>;
                 DEFINE FIELD IF NOT EXISTS content ON messages TYPE object;
                 DEFINE FIELD IF NOT EXISTS metadata ON messages TYPE object;
                 DEFINE FIELD IF NOT EXISTS timestamp ON messages TYPE int;
                 DEFINE FIELD IF NOT EXISTS reply_to ON messages TYPE option<string>;
                 DEFINE INDEX IF NOT EXISTS message_type_idx ON messages FIELDS message_type;
                 DEFINE INDEX IF NOT EXISTS from_agent_idx ON messages FIELDS from_agent;
                 DEFINE INDEX IF NOT EXISTS timestamp_idx ON messages FIELDS timestamp;",
            )
            .await
            .context("Failed to create messages schema")?;

        self.db
            .query(
                "DEFINE TABLE IF NOT EXISTS state SCHEMAFULL;
                 DEFINE FIELD IF NOT EXISTS key ON state TYPE string;
                 DEFINE FIELD IF NOT EXISTS value ON state TYPE any;
                 DEFINE FIELD IF NOT EXISTS owner_agent ON state TYPE any;
                 DEFINE FIELD IF NOT EXISTS timestamp ON state TYPE int;
                 DEFINE FIELD IF NOT EXISTS version ON state TYPE int;
                 DEFINE INDEX IF NOT EXISTS key_idx ON state FIELDS key UNIQUE;",
            )
            .await
            .context("Failed to create state schema")?;

        self.db
            .query(
                "DEFINE TABLE IF NOT EXISTS decisions SCHEMAFULL;
                 DEFINE FIELD IF NOT EXISTS id ON decisions TYPE string;
                 DEFINE FIELD IF NOT EXISTS decision_type ON decisions TYPE string;
                 DEFINE FIELD IF NOT EXISTS description ON decisions TYPE string;
                 DEFINE FIELD IF NOT EXISTS made_by ON decisions TYPE string;
                 DEFINE FIELD IF NOT EXISTS rationale ON decisions TYPE object;
                 DEFINE FIELD IF NOT EXISTS alternatives_considered ON decisions TYPE array;
                 DEFINE FIELD IF NOT EXISTS timestamp ON decisions TYPE int;
                 DEFINE FIELD IF NOT EXISTS parent_decision ON decisions TYPE option<string>;
                 DEFINE INDEX IF NOT EXISTS decision_type_idx ON decisions FIELDS decision_type;
                 DEFINE INDEX IF NOT EXISTS timestamp_idx ON decisions FIELDS timestamp;",
            )
            .await
            .context("Failed to create decisions schema")?;

        Ok(())
    }

    /// Subscribe to real-time VOX messages for a specific team via MOM
    pub fn subscribe(&self, team: Team) -> broadcast::Receiver<VoxMessage> {
        self.mom.subscribe(team)
    }

    /// Update AURA vitality pulse
    /// 
    /// Uses SurrealDB time::now() to prevent clock skew issues.
    pub async fn send_pulse(&self, agent_id: AgentId, status: zed42_core::AgentStatus) -> Result<()> {
        let query = "UPSERT type::thing('aura_pulses', $agent_id) SET 
            last_pulse = time::now(),
            status = $status";
        
        self.db.query(query)
            .bind(("agent_id", agent_id.to_string()))
            .bind(("status", format!("{:?}", status).to_lowercase()))
            .await
            .context("Failed to send AURA vitality pulse")?;

        Ok(())
    }

    /// Post a message to the blackboard
    pub async fn post_message(&self, message: Message) -> Result<()> {
        let _: Message = self
            .db
            .create("messages")
            .content(message)
            .await
            .context("Failed to post message")?
            .context("Failed to create message record")?;

        Ok(())
    }

    /// Get messages matching filter
    pub async fn get_messages(&self, filter: MessageFilter) -> Result<Vec<Message>> {
        let mut conditions = Vec::new();

        if let Some(msg_type) = filter.message_type {
            conditions.push(format!("message_type = '{:?}'", msg_type));
        }

        if let Some(from) = filter.from_agent {
            conditions.push(format!("from_agent = '{}'", from));
        }

        if let Some(to) = filter.to_agent {
            conditions.push(format!("to_agent = '{}'", to));
        }

        if let Some(since) = filter.since_timestamp {
            conditions.push(format!("timestamp >= {}", since));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let limit_clause = if let Some(limit) = filter.limit {
            format!("LIMIT {}", limit)
        } else {
            String::new()
        };

        let query = format!(
            "SELECT * FROM messages {} ORDER BY timestamp DESC {}",
            where_clause, limit_clause
        );

        let mut response: Response = self.db.query(query).await?;
        let messages: Vec<Message> = response.take(0)?;

        Ok(messages)
    }

    /// Set state entry
    pub async fn set_state(&self, entry: StateEntry) -> Result<()> {
        // Update or insert
        let _: StateEntry = self
            .db
            .create("state")
            .content(entry)
            .await
            .context("Failed to set state")?
            .context("Failed to create state record")?;

        Ok(())
    }

    /// Get state entry
    pub async fn get_state(&self, key: &StateKey) -> Result<Option<StateEntry>> {
        let query = format!("SELECT * FROM state WHERE key = '{}'", key);

        let mut response: Response = self.db.query(query).await?;
        let entries: Vec<StateEntry> = response.take(0)?;

        Ok(entries.into_iter().next())
    }

    /// Record a decision
    pub async fn record_decision(&self, decision: DecisionNode) -> Result<()> {
        let _: DecisionNode = self
            .db
            .create("decisions")
            .content(decision)
            .await
            .context("Failed to record decision")?
            .context("Failed to create decision record")?;

        Ok(())
    }

    /// Get decision history for an agent
    pub async fn get_decisions(&self, agent_id: Option<AgentId>) -> Result<Vec<DecisionNode>> {
        let query = if let Some(agent) = agent_id {
            format!(
                "SELECT * FROM decisions WHERE made_by = '{}' ORDER BY timestamp DESC",
                agent
            )
        } else {
            "SELECT * FROM decisions ORDER BY timestamp DESC".to_string()
        };

        let mut response: Response = self.db.query(query).await?;
        let decisions: Vec<DecisionNode> = response.take(0)?;

        Ok(decisions)
    }

    /// Get blackboard statistics
    pub async fn stats(&self) -> Result<BlackboardStats> {
        let mut msg_count_response: Response = self
            .db
            .query("SELECT count() FROM messages GROUP ALL")
            .await?;
        let total_messages: Option<i64> = msg_count_response.take("count")?;

        let mut state_count_response: Response = self
            .db
            .query("SELECT count() FROM state GROUP ALL")
            .await?;
        let total_state_entries: Option<i64> = state_count_response.take("count")?;

        let mut decision_count_response: Response = self
            .db
            .query("SELECT count() FROM decisions GROUP ALL")
            .await?;
        let total_decisions: Option<i64> = decision_count_response.take("count")?;

        Ok(BlackboardStats {
            total_messages: total_messages.unwrap_or(0) as usize,
            total_state_entries: total_state_entries.unwrap_or(0) as usize,
            total_decisions: total_decisions.unwrap_or(0) as usize,
            db_path: self.db_path.clone(),
        })
    }

    /// Internal access to the SurrealDB instance
    pub(crate) fn db(&self) -> &surrealdb::Surreal<surrealdb::engine::local::Db> {
        &self.db
    }

    /// Internal access to the MOM Watcher
    pub(crate) fn mom(&self) -> &crate::MOMWatcher {
        &self.mom
    }
}

