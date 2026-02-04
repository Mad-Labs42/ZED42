//! SpaceSentry - Hardware Resiliency Monitor
//!
//! Prevents catastrophic OS Error 112 (Disk Full) by enforcing backpressure
//! when disk space is critical.

use sysinfo::{System, Disks};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use parking_lot::RwLock;
use anyhow::Result;
use std::time::Duration;

/// Critical disk space threshold (500MB)
const CRITICAL_THRESHOLD_BYTES: u64 = 500 * 1024 * 1024; 

pub struct SpaceSentry {
    /// Global system state
    system: RwLock<System>,
    /// Disks state
    disks: RwLock<Disks>,
    /// Alert flag
    critical_state: AtomicBool,
}

impl SpaceSentry {
    pub fn new() -> Self {
        Self {
            system: RwLock::new(System::new_all()),
            disks: RwLock::new(Disks::new_with_refreshed_list()),
            critical_state: AtomicBool::new(false),
        }
    }

    /// Check system health and update status
    /// Returns Error::Backpressure if critical
    pub fn check_vital_signs(&self) -> Result<()> {
        // Lightweight check: Atomic flag first
        if self.critical_state.load(Ordering::Relaxed) {
             // Re-verify before erroring (in case of race/cleanup)
             self.refresh_and_verify()?;
             if self.critical_state.load(Ordering::Relaxed) {
                 return Err(anyhow::anyhow!("CRITICAL: Disk Space Low (<500MB). Titans Halted. Action: Clean Disk."));
             }
        }
        
        // Periodic refresh could be here, but we'll assume external trigger or lazy refresh
        // For now, let's refresh disjointly or via a background ticker.
        // To be safe/simple for Phase 3, we verify on demand but optimizing for read.
        
        Ok(())
    }

    /// Force refresh and status update
    pub fn refresh_and_verify(&self) -> Result<()> {
         let mut disks = self.disks.write();
         disks.refresh_list();
         
         // Check C: or Root
         for disk in disks.list() {
             // Heuristic: Check root or primary drive. Windows: "C:\\"
             if disk.mount_point().to_string_lossy().contains("C:") || disk.mount_point() == std::path::Path::new("/") {
                 if disk.available_space() < CRITICAL_THRESHOLD_BYTES {
                     self.critical_state.store(true, Ordering::SeqCst);
                     return Ok(());
                 }
             }
         }
         
         self.critical_state.store(false, Ordering::SeqCst);
         Ok(())
    }
}
