//! Atomic File State Guard
//!
//! Implements the "Shadow Write" pattern:
//! 1. Writes to a .tmp file
//! 2. Atomically renames to target on success
//! 3. Cleans up garbage on failure/drop
//!
//! # Safety
//! Enforces path sandboxing via PathSanitizer.

use crate::file_manipulation::PathSanitizer;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use anyhow::{Context, Result};

/// Guard for atomic file/directory operations
pub struct FileStateGuard {
    target_path: PathBuf,
    temp_path: PathBuf,
    is_dir: bool,
    completed: AtomicBool,
}

impl FileStateGuard {
    /// Create a new guard for writing to `path`
    pub fn new(sanitizer: &PathSanitizer, path: &str) -> Result<Self> {
        Self::create_guard(sanitizer, path, false)
    }

    /// Create a new guard for directory operations
    pub fn new_dir(sanitizer: &PathSanitizer, path: &str) -> Result<Self> {
        Self::create_guard(sanitizer, path, true)
    }

    fn create_guard(sanitizer: &PathSanitizer, path: &str, is_dir: bool) -> Result<Self> {
        // 0. Hook: Check disk space before any write operation
        Self::check_disk_health()?;

        // 1. Sanitize the target path
        let target_path = sanitizer.sanitize(path).context("Invalid target path")?;
        
        // 2. Create temp path (sibling)
        let file_name = target_path.file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file/dir name"))?
            .to_string_lossy();
        let temp_name = if is_dir {
            format!(".tmp_dir_{}", file_name)
        } else {
            format!(".{}.tmp", file_name)
        };
        
        let target_parent = target_path.parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid parent dir"))?;
        let temp_path = target_parent.join(temp_name);

        Ok(Self {
            target_path,
            temp_path,
            is_dir,
            completed: AtomicBool::new(false),
        })
    }

    /// SpaceSentry Hook
    fn check_disk_health() -> Result<()> {
        use sysinfo::Disks;
        let disks = Disks::new_with_refreshed_list();
        for disk in disks.list() {
            if disk.mount_point().to_string_lossy().contains("C:") || disk.mount_point() == std::path::Path::new("/") {
                // 500MB Threshold
                if disk.available_space() < 500 * 1024 * 1024 {
                    return Err(anyhow::anyhow!("SpaceSentry: Disk space critical (<500MB). FS Writes Halted."));
                }
            }
        }
        Ok(())
    }

    /// Get the path to write to (the temp file)
    pub fn path(&self) -> &PathBuf {
        &self.temp_path
    }

    /// Mark operation as complete and commit changes (atomic rename)
    pub fn commit(&self) -> Result<()> {
        if self.completed.load(Ordering::Acquire) {
            return Ok(());
        }

        // Rename temp -> target (Atomic on POSIX, mostly atomic on Windows)
        std::fs::rename(&self.temp_path, &self.target_path)
            .context(format!("Failed to commit file: atomic rename failed from {:?} to {:?}", self.temp_path, self.target_path))?;

        self.completed.store(true, Ordering::Release);
        Ok(())
    }
}

impl Drop for FileStateGuard {
    fn drop(&mut self) {
        if !self.completed.load(Ordering::Acquire) {
            // Aborted or failed. Clean up temp path.
            if self.temp_path.exists() {
                if self.is_dir {
                    let _ = std::fs::remove_dir_all(&self.temp_path);
                } else {
                    let _ = std::fs::remove_file(&self.temp_path);
                }
            }
        }
    }
}
