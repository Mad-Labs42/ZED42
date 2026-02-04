//! Titan Substrate - Shared Handle Registry
//!
//! Provides a centralized, thread-safe registry for all system components.
//! Uses `parking_lot` for zero-panic concurrency.

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::Arc;
use std::any::Any;

pub mod monitor;

/// The Titan Substrate Registry
///
/// Holds atomic references to core system components.
/// Designed for high-concurrency access without mutex poisoning.
pub struct TitanSubstrate {
    /// Generic storage for subsystems (Memory, Agents, etc.) to avoid circular deps in Core
    /// Key: Subsystem Name, Value: Arc<Any>
    subsystems: RwLock<std::collections::HashMap<String, Arc<dyn Any + Send + Sync>>>,
    /// Hardware Monitor
    pub sentry: Arc<monitor::SpaceSentry>,
}

impl Default for TitanSubstrate {
    fn default() -> Self {
        Self::new()
    }
}

impl TitanSubstrate {
    /// generic singleton instance pattern would be here, but we'll instantiate it in Cortex.
    pub fn new() -> Self {
        Self {
            subsystems: RwLock::new(std::collections::HashMap::new()),
            sentry: Arc::new(monitor::SpaceSentry::new()),
        }
    }

    /// Check system health via SpaceSentry
    pub fn list_vital_signs(&self) -> anyhow::Result<()> {
        self.sentry.refresh_and_verify()?; // Force update
        self.sentry.check_vital_signs()
    }

    /// Register a subsystem
    pub fn register<T: Any + Send + Sync>(&self, name: &str, system: Arc<T>) {
        let mut map = self.subsystems.write();
        map.insert(name.to_string(), system);
    }

    /// Retrieve a subsystem
    ///
    /// # Thread Safety
    /// This method is non-blocking and safe for concurrent use.
    /// It uses an `RwLock` read guard to access the registry.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use std::sync::Arc;
    /// # use zed42_core::titan::TitanSubstrate;
    /// # struct MySystem;
    /// let titan = TitanSubstrate::new();
    /// titan.register("sys", Arc::new(parking_lot::RwLock::new(MySystem)));
    /// 
    /// // Safe concurrent access
    /// if let Some(sys_lock) = titan.get::<MySystem>("sys") {
    ///     let sys = sys_lock.read();
    ///     // Use sys...
    /// }
    /// ```
    /// Retrieve a generic subsystem
    pub fn get<T: Any + Send + Sync>(&self, name: &str) -> Option<Arc<RwLock<T>>> {
        let map = self.subsystems.read();
        map.get(name).and_then(|any| {
             any.clone().downcast::<RwLock<T>>().ok()
        })
    }

    /// Specialized handle for Blackboard
    pub fn get_blackboard_handle(&self) -> anyhow::Result<Arc<RwLock<zed42_blackboard::BlackboardDb>>> {
        self.get::<zed42_blackboard::BlackboardDb>("Blackboard")
            .context("Titan Registry: Blackboard handle not found")
    }

    /// Specialized handle for Memory
    pub fn get_memory_handle(&self) -> anyhow::Result<Arc<RwLock<zed42_memory::MemorySubstrate>>> {
        self.get::<zed42_memory::MemorySubstrate>("Memory")
            .context("Titan Registry: Memory handle not found")
    }

    /// Specialized handle for LLM Client
    /// Note: Returns the concrete client stored under "LlmClient"
    pub fn get_llm_handle(&self) -> anyhow::Result<Arc<RwLock<Arc<dyn zed42_llm::LlmClient>>>> {
        // We store LlmClient as Arc<RwLock<Arc<dyn LlmClient>>> to handle the trait object
        self.get::<Arc<dyn zed42_llm::LlmClient>>("LlmClient")
            .context("Titan Registry: LLM handle not found")
    }
}
