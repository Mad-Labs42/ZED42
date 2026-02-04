//! SAGA Orchestration Loop - Cognitive Cycle (OODA)
//!
//! Implements the Observe-Orient-Decide-Act cycle across the MOM Reactive Substrate.
//! Mandates participation in the AURA Vitality Substrate via background heartbeat pulses.

use crate::orchestrator::mailbox::PriorityMailbox;
use zed42_blackboard::{BlackboardDb, VoxMessage, Team, StateResolver, ConsensusState};
use zed42_core::{AgentId, Result, AgentStatus};
use zed42_memory::MemorySubstrate;
use zed42_llm::LlmClient;
use std::sync::Arc;
use tracing::{info, error, instrument, debug, warn};
use serde_json::Value;
use uuid::Uuid;

use zed42_core::titan::TitanSubstrate;

/// The Cortex - The unified heartbeat of a SAGA agent
/// 
/// Manages the OODA loop via the Titan Substrate handles.
pub struct Cortex {
    agent_id: AgentId,
    team: Team,
    substrate: Arc<TitanSubstrate>,
    mailbox: PriorityMailbox,
}

impl Cortex {
    /// Create a new Cortex for an agent
    pub fn new(
        agent_id: AgentId, 
        team: Team, 
        substrate: Arc<TitanSubstrate>,
    ) -> Self {
        Self {
            agent_id,
            team,
            substrate,
            mailbox: PriorityMailbox::new(1024),
        }
    }

    /// Primary execution loop
    pub async fn run(&mut self) -> Result<()> {
        info!(agent_id = %self.agent_id, team = ?self.team, "Starting SAGA Cortex Loop");
        
        let blackboard_lock = self.substrate.get_blackboard_handle()?;
        
        // Critical: Check Hardware Health before starting
        self.substrate.list_vital_signs()
            .context("Titan Substrate: Hardware Health Check Failed - SpaceSentry Alert")?;

        let blackboard = blackboard_lock.read();
        let mut mom_rx = blackboard.subscribe(self.team);
        
        // AURA Vitality pulse interval
        let mut pulse_tick = tokio::time::interval(tokio::time::Duration::from_secs(30));
        
        loop {
            // Periodic Health Audit
            if let Err(e) = self.substrate.list_vital_signs() {
                error!("SpaceSentry Alert: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                continue; // Backpressure: Pause loop until resolved
            }

            tokio::select! {
                // OBSERVE: Incoming real-time VOX updates via MOM
                Ok(msg) = mom_rx.recv() => {
                    debug!(sender = %msg.sender, "Observed VOX message via Titan-linked MOM substrate");
                    self.mailbox.push(msg);
                }

                // AURA PULSE
                _ = pulse_tick.tick() => {
                    let jitter = rand::random::<u64>() % 5 + 1;
                    tokio::time::sleep(tokio::time::Duration::from_secs(jitter)).await;
                    
                    let blackboard = blackboard_lock.read();
                    if let Err(e) = blackboard.send_pulse(self.id(), AgentStatus::Working).await {
                        error!("AURA Substrate: failed to send vitality pulse: {}", e);
                    }
                }

                // COGNITIVE STEP
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(10)) => {
                    if let Some(msg) = self.mailbox.pop() {
                        if let Err(e) = self.cycle_step(msg).await {
                            error!("Error in cognitive cycle: {}", e);
                        }
                    }
                }
            }
        }
    }

    /// Run a single simplified OODA loop cycle using VOX messages
    #[instrument(skip(self, msg), fields(correlation_id = %msg.correlation_id))]
    async fn cycle_step(&mut self, msg: VoxMessage) -> Result<()> {
        let blackboard_lock = self.substrate.get_blackboard_handle()?;
        let memory_lock = self.substrate.get_memory_handle()?;
        let llm_lock = self.substrate.get_llm_handle()?;

        // 1. ORIENT: Fetch consensus state and RAG context (Non-blocking acquisitions)
        let consensus = {
            let blackboard = blackboard_lock.read();
            StateResolver::resolve_thread(&blackboard, msg.correlation_id).await
                .context("Cortex Orientation: Failed to resolve thread state")?
        };
        debug!(thread_id = %consensus.thread_id, "Oriented: VOX context loaded");

        // RAG: Background memory retrieval
        let memory_context = {
            let memory = memory_lock.read();
            self.retrieve_memories(&msg, Arc::new(memory.clone())).await.unwrap_or_default()
        };

        // 2. DECIDE
        let action = self.decide(&msg, &consensus, &memory_context).await?;

        // 3. ACT
        if let Some(act_msg) = action {
            let blackboard = blackboard_lock.read();
            blackboard.post_message(act_msg).await
                .context("Cortex Action: Failed to post message to blackboard")?;
        }

        // 4. REFLECT
        if self.is_milestone(&msg) {
            let memory = memory_lock.read().clone();
            let llm = llm_lock.read().clone();
            let agent_id = self.agent_id;
            let consensus_clone = consensus.clone();
            let msg_id = msg.correlation_id;

            tokio::spawn(async move {
                // Optimization: Reflect is a background task
                if let Err(e) = Self::reflect(Arc::new(memory), llm, agent_id, consensus_clone, msg_id).await {
                    error!("Reflection loop failed: {}", e);
                }
            });
        }

        Ok(())
    }

    /// Decision logic - Typed matching via VOX Schema Registry
    async fn decide(
        &self, 
        msg: &VoxMessage, 
        _consensus: &ConsensusState,
        _memory_context: &[zed42_memory::MemoryResult]
    ) -> Result<Option<zed42_core::Message>> {
        match &msg.payload {
            zed42_core::vox::VoxPayload::TaskAssignment { task_id, .. } => {
                info!(%task_id, "Decided: Acknowledging VOX task assignment");
                let mut ack = zed42_core::Message::new(
                    self.agent_id,
                    zed42_core::messages::MessageTarget::Agent(uuid::Uuid::parse_str(&msg.sender.id.to_string()).unwrap_or_default()),
                    zed42_core::MessageType::TaskComplete { 
                        task_id: task_id.clone(), 
                        result: "Acknowledged via VOX".to_string() 
                    },
                    2 // High priority for ACK
                );
                ack.thread_id = msg.correlation_id;
                Ok(Some(ack))
            }
            _ => Ok(None),
        }
    }

    /// Fetch memories via RAG - Extracting text from VOX Grammar
    async fn retrieve_memories(&self, msg: &VoxMessage, memory: Arc<MemorySubstrate>) -> Result<Vec<zed42_memory::MemoryResult>> {
        let query_text = match &msg.payload {
            zed42_core::vox::VoxPayload::TaskAssignment { description, .. } => description.as_str(),
            zed42_core::vox::VoxPayload::Proposal { content } => content.as_str(),
            zed42_core::vox::VoxPayload::Observation { content } => content.as_str(),
            _ => "general coordination",
        };

        // Query memory tiers in parallel
        memory.query(query_text, 5).await.map_err(Into::into)
    }

    /// Determine if the message indicates a milestone reach
    fn is_milestone(&self, msg: &VoxMessage) -> bool {
        match &msg.payload {
            zed42_core::vox::VoxPayload::Ack { .. } => true,
            _ => false,
        }
    }

    /// Asynchronous Reflection - Summarizes and stores lessons learned
    async fn reflect(
        memory: Arc<MemorySubstrate>, 
        llm: Arc<dyn LlmClient>, 
        agent_id: AgentId,
        consensus: ConsensusState,
        source_id: Uuid,
    ) -> Result<()> {
        info!(thread_id = %consensus.thread_id, "Starting background reflection...");

        // 1. Generate Lesson Summary via LLM
        let prompt = format!(
            "Analyze the following thread context and extract 'Lessons Learned'. \
             Focus on architectural decisions, pitfalls avoided, or successful patterns used.\n\
             Context: {:#?}", 
            consensus.values
        );

        let system = "You are a senior SAGA agent performing self-reflection. \
                      Output a concise summary of technical lessons learned. \
                      Format: '[Pattern/Pitfall]: Description'.";

        let req = zed42_llm::LlmRequest::new(prompt)
            .system(system.to_string())
            .agent(agent_id.to_string());

        let resp = llm.complete(req).await.map_err(|e| zed42_core::Error::Llm(e.to_string()))?;
        let lesson = resp.content;

        // 2. Generate Embedding for the lesson
        let embed_req = zed42_llm::EmbeddingRequest::new(lesson.clone());
        let embed_resp = llm.embed(embed_req).await.map_err(|e| zed42_core::Error::Llm(e.to_string()))?;

        // 3. Store in Knowledge Graph (Tier 3)
        if let Some(kg) = memory.knowledge_graph() {
            let node = zed42_memory::knowledge_graph::KnowledgeNode {
                id: Uuid::new_v4().to_string(),
                node_type: "documentation".to_string(),
                name: format!("Lesson: {}", consensus.thread_id),
                content: serde_json::json!({
                    "lesson": lesson,
                    "thread_id": consensus.thread_id,
                    "source_message": source_id,
                }).to_string(),
                embedding: Some(embed_resp.embedding),
                metadata: serde_json::json!({
                    "traceability": {
                        "agent_id": agent_id,
                        "source_id": source_id,
                        "type": "reflection"
                    }
                }).to_string(),
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
            };

            kg.insert_node(node).await.map_err(zed42_core::Error::from)?;
            info!(thread_id = %consensus.thread_id, "Reflection stored in Memory Fabric");
        }

        Ok(())
    }

    fn id(&self) -> AgentId {
        self.agent_id
    }
}
