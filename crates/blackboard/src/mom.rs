//! MOM (Message-Oriented Middleware) Watcher - Native Reactive Substrate
//!
//! Maintains a resilient WebSocket connection to SurrealDB and
//! routes LIVE SELECT notifications to agents via MOM broadcast channels.

use crate::types::VoxMessage;
use surrealdb::Action;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::Surreal;
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use tracing::{warn, info, error};
use zed42_core::Team;
use futures_util::StreamExt;
use anyhow::{Result, Context};
use dashmap::DashMap;
use std::sync::Arc;

/// Authoritative source for real-time coordination via the MOM Reactive Substrate
pub struct MOMWatcher {
    /// Team-based broadcast channels
    senders: Arc<DashMap<String, broadcast::Sender<VoxMessage>>>,
    /// Connection address
    addr: String,
    /// Namespace
    ns: String,
    /// Database name
    db: String,
}

impl MOMWatcher {
    /// Create a new MOMWatcher
    pub fn new(addr: String, ns: String, db: String) -> Self {
        Self {
            senders: Arc::new(DashMap::new()),
            addr,
            ns,
            db,
        }
    }

    /// Subscribe to VOX messages for a specific team via MOM
    pub fn subscribe(&self, team: Team) -> broadcast::Receiver<VoxMessage> {
        let team_key = format!("{:?}", team).to_lowercase();
        self.subscribe_raw(team_key)
    }

    /// Internal subscription by team string (supports "all")
    fn subscribe_raw(&self, team_key: String) -> broadcast::Receiver<VoxMessage> {
        let tx = self.senders.entry(team_key).or_insert_with(|| {
            let (tx, _) = broadcast::channel(1024);
            tx
        });
        tx.subscribe()
    }

    /// Broadcast a system message immediately via MOM (e.g., AURA pulse alerts)
    pub async fn broadcast_system_message(&self, msg: VoxMessage) {
        let team_key = msg.target_team.to_lowercase();
        
        if team_key == "all" {
            // Send to everyone
            for entry in self.senders.iter() {
                let _ = entry.value().send(msg.clone());
            }
        } else if let Some(tx) = self.senders.get(&team_key) {
            let _ = tx.send(msg);
        }
    }

    /// Primary execution loop with exponential backoff
    pub async fn run_loop(&self) {
        let mut backoff = Duration::from_secs(1);
        loop {
            match self.run().await {
                Ok(_) => {
                    info!("MOM Substrate: connection closed normally, reconnecting...");
                    backoff = Duration::from_secs(1);
                }
                Err(e) => {
                    error!("MOM Substrate error: {}. Reconnecting in {:?}...", e, backoff);
                    sleep(backoff).await;
                    backoff = (backoff * 2).min(Duration::from_secs(60));
                }
            }
        }
    }

    /// Establish connection and process MOM LIVE SELECT stream
    async fn run(&self) -> Result<()> {
        let db = Surreal::new::<Ws>(&self.addr).await
            .context("Failed to connect to SurrealDB MOM substrate via Ws")?;
        
        db.use_ns(&self.ns).use_db(&self.db).await
            .context("Failed to select MOM namespace/database")?;

        info!("MOM Substrate: connected to {}/{}, starting LIVE SELECT...", self.ns, self.db);

        // Native MOM Reactive Substrate LIVE SELECT
        let mut stream = db.select("blackboard").live().await
            .context("Failed to start MOM LIVE SELECT on blackboard")?;
        
        // Ensure stability across thread boundaries
        tokio::pin!(stream);

        while let Some(result) = stream.next().await {
            match result {
                Ok(notification) => {
                    match notification.action {
                        Action::Create | Action::Update => {
                            let msg: VoxMessage = match serde_json::from_value(notification.data) {
                                Ok(m) => m,
                                Err(e) => {
                                    warn!("MOM Substrate: failed to deserialize VOX notification data: {}", e);
                                    continue;
                                }
                            };

                            let team_key = msg.target_team.to_lowercase();
                            if let Some(tx) = self.senders.get(&team_key) {
                                // If an agent lags, drop the message and warn
                                if let Err(e) = tx.send(msg) {
                                    warn!("MOM Substrate: broadcast drop/error for team {}: {}", team_key, e);
                                }
                            }
                        }
                        _ => {
                            // Ignored actions (Delete, etc.)
                            continue;
                        }
                    }
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("MOM Substrate: stream error: {}", e));
                }
            }
        }
        
        Ok(())
    }
}
