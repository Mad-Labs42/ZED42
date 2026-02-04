use crate::{BlackboardDb, VoxMessage};
use zed42_ledger::IntelligenceLedger;
use anyhow::{Result, Context};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error, instrument};
use serde::Deserialize;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
struct AuraPulseRow {
    id: surrealdb::sql::Thing,
    last_pulse: DateTime<Utc>,
    status: String,
}

/// AURA Sentinel - Background monitor for agent vitality
pub struct AuraSentinel {
    blackboard: Arc<BlackboardDb>,
    ledger: Arc<IntelligenceLedger>,
    check_interval: Duration,
    laggard_threshold: Duration,
    ghost_threshold: Duration,
}

impl AuraSentinel {
    /// Create a new AuraSentinel
    pub fn new(blackboard: Arc<BlackboardDb>, ledger: Arc<IntelligenceLedger>) -> Self {
        Self {
            blackboard,
            ledger,
            check_interval: Duration::from_secs(30),
            laggard_threshold: Duration::from_secs(60),
            ghost_threshold: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Start the AURA monitor loop
    pub async fn run(&self) {
        info!("AURA Substrate: starting vitality monitor loop...");
        loop {
            if let Err(e) = self.monitor_pulse_health().await {
                error!("AURA Substrate: error during pulse health check: {}", e);
            }
            sleep(self.check_interval).await;
        }
    }

    /// Primary monitoring logic
    #[instrument(skip(self))]
    async fn monitor_pulse_health(&self) -> Result<()> {
        let query = "SELECT *, (time::now() - last_pulse) AS age FROM aura_pulses";
        let mut response = self.blackboard.db().query(query).await?;
        
        let rows: Vec<AuraPulseRow> = response.take(0)?;
        
        for row in rows {
            let age_secs = (Utc::now() - row.last_pulse).num_seconds();
            
            if age_secs > self.ghost_threshold.as_secs() as i64 {
                if row.status != "ghost" {
                    self.dissolve_ghost(row.id.id.to_string(), row.last_pulse).await?;
                }
            } else if age_secs > self.laggard_threshold.as_secs() as i64 {
                if row.status != "laggard" {
                    self.mark_laggard(row.id.id.to_string(), row.last_pulse).await?;
                }
            }
        }
        
        Ok(())
    }

    #[instrument(skip(self))]
    async fn mark_laggard(&self, agent_id_str: String, last_pulse: DateTime<Utc>) -> Result<()> {
        warn!(agent_id = %agent_id_str, %last_pulse, "Agent flagged as LAGGARD");
        
        let query = "UPDATE type::thing('aura_pulses', $id) SET status = 'laggard'";
        self.blackboard.db().query(query)
            .bind(("id", agent_id_str))
            .await?;
            
        Ok(())
    }

    #[instrument(skip(self))]
    async fn dissolve_ghost(&self, agent_id_str: String, last_pulse: DateTime<Utc>) -> Result<()> {
        error!(agent_id = %agent_id_str, %last_pulse, "Agent flagged as GHOST - Dissolving...");
        
        // 1. Update status in DB
        let query = "UPDATE type::thing('aura_pulses', $id) SET status = 'ghost'";
        self.blackboard.db().query(query)
            .bind(("id", agent_id_str.clone()))
            .await?;

        // 2. Extract Uuid from id string
        let agent_id = uuid::Uuid::parse_str(&agent_id_str)
            .context("Failed to parse agent_id from pulse row")?;

        // 2b. Freeze Budget in Intelligence Ledger (Fiscal Immunity)
        if let Err(e) = self.ledger.freeze_budget(&agent_id_str, "AURA Dissolution: Agent became Ghost").await {
            error!(agent_id = %agent_id_str, "AURA Substrate: Failed to freeze budget for ghost: {}", e);
        } else {
            info!(agent_id = %agent_id_str, "AURA Substrate: Budget FROZEN for ghost");
        }

        // 3. Broadcast DissolveAgent message via MOM for immediate alert (Typed VOX)
        let dissolve_msg = VoxMessage {
            sender: surrealdb::sql::Thing::from(("system", "aura")),
            target_team: "all".to_string(),
            priority: 255, // Critical system message
            correlation_id: uuid::Uuid::new_v4(),
            payload: zed42_core::vox::VoxPayload::SystemAlert {
                action: "dissolve_ghost".to_string(),
                agent_id: Some(agent_id),
                reason: "ghost_threshold_exceeded".to_string(),
            },
            created_at: Utc::now(),
        };

        // Notify via MOM Reactive Substrate (bypass DB for latency)
        self.blackboard.mom().broadcast_system_message(dissolve_msg).await;
            
        Ok(())
    }
}
