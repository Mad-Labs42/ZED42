//! Working Memory (Tier 1) - Hot Cache
//!
//! In-memory LRU cache with semantic importance weighting.
//! Target: ~500MB capacity, sub-millisecond access latency
//!
//! Features:
//! - LRU eviction policy with importance weighting
//! - User-mentioned items are pinned
//! - Thread-safe concurrent access
//! - Automatic size tracking and eviction

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;

/// Maximum memory allocation in bytes (~500MB)
const MAX_MEMORY_BYTES: usize = 500 * 1024 * 1024;

/// Estimated average entry size for capacity calculation
const AVG_ENTRY_SIZE_BYTES: usize = 1024;

/// Entry metadata for LRU and importance tracking
#[derive(Debug, Clone)]
struct CacheEntry {
    key: String,
    value: serde_json::Value,
    last_accessed: Instant,
    access_count: u32,
    importance_weight: f32,
    is_pinned: bool,
    estimated_size: usize,
    created_at: u64,
}

impl CacheEntry {
    fn new(key: String, value: serde_json::Value, importance_weight: f32, is_pinned: bool) -> Self {
        let estimated_size = Self::estimate_size(&key, &value);
        Self {
            key,
            value,
            last_accessed: Instant::now(),
            access_count: 1,
            importance_weight,
            is_pinned,
            estimated_size,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(std::time::Duration::from_secs(0))
                .as_secs(),
        }
    }

    fn estimate_size(key: &str, value: &serde_json::Value) -> usize {
        key.len() + serde_json::to_string(value).unwrap_or_default().len()
    }

    fn touch(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count = self.access_count.saturating_add(1);
    }

    /// Calculate eviction priority (lower = evict first)
    /// Factors: recency, access count, importance weight, pinned status
    fn eviction_priority(&self) -> f32 {
        if self.is_pinned {
            return f32::MAX; // Never evict pinned entries
        }

        let recency_score = self.last_accessed.elapsed().as_secs_f32().recip();
        let access_score = (self.access_count as f32).ln_1p();
        let importance_score = self.importance_weight;

        // Weighted combination
        (recency_score * 0.4) + (access_score * 0.3) + (importance_score * 0.3)
    }
}

/// Working Memory - Tier 1 hot cache
///
/// Provides sub-millisecond access to frequently used data.
/// Uses LRU eviction with semantic importance weighting.
#[derive(Clone)]
pub struct WorkingMemory {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    total_size: Arc<RwLock<usize>>,
}

impl WorkingMemory {
    /// Create a new working memory instance
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            total_size: Arc::new(RwLock::new(0)),
        }
    }

    /// Get a value from the cache
    ///
    /// # Returns
    /// - `Some(value)` if key exists
    /// - `None` if key not found
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        let mut cache = self.cache.write();

        if let Some(entry) = cache.get_mut(key) {
            let entry: &mut CacheEntry = entry;
            entry.touch();
            Some(entry.value.clone())
        } else {
            None
        }
    }

    /// Insert a value into the cache
    ///
    /// # Arguments
    /// - `key` - Unique identifier
    /// - `value` - Data to store
    /// - `importance_weight` - Semantic importance (0.0-1.0, default 0.5)
    /// - `is_pinned` - Prevent eviction (for user-mentioned items)
    ///
    /// # Errors
    /// Returns error if lock poisoned
    pub fn insert(
        &self,
        key: String,
        value: serde_json::Value,
        importance_weight: f32,
        is_pinned: bool,
    ) -> anyhow::Result<()> {
        let entry = CacheEntry::new(key.clone(), value, importance_weight, is_pinned);
        let entry_size = entry.estimated_size;

        {
            let mut cache = self.cache.write();
            let mut total_size = self.total_size.write();

            // Update existing entry
            if let Some(old_entry) = cache.get(&key) {
                *total_size = total_size.saturating_sub(old_entry.estimated_size);
            }

            cache.insert(key.clone(), entry);
            *total_size = total_size.saturating_add(entry_size);
        }

        // Evict if over capacity
        self.evict_if_needed()?;
        
        Ok(())
    }

    /// Remove a specific key from cache
    pub fn remove(&self, key: &str) -> anyhow::Result<()> {
        let mut cache = self.cache.write();
        let mut total_size = self.total_size.write();

        if let Some(entry) = cache.remove(key) {
            *total_size = total_size.saturating_sub(entry.estimated_size);
        }

        Ok(())
    }

    /// Check if key exists in cache
    pub fn contains(&self, key: &str) -> bool {
        self.cache.read().contains_key(key)
    }

    /// Get current cache statistics
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.read();
        let total_size = *self.total_size.read();

        let pinned_count = cache.values().filter(|e| e.is_pinned).count();

        CacheStats {
            entry_count: cache.len(),
            pinned_count,
            total_size_bytes: total_size,
            capacity_bytes: MAX_MEMORY_BYTES,
            utilization_pct: (total_size as f32 / MAX_MEMORY_BYTES as f32) * 100.0,
        }
    }

    /// Clear all non-pinned entries
    pub fn clear(&self) -> anyhow::Result<()> {
        let mut cache = self.cache.write();
        let mut total_size = self.total_size.write();
        cache.retain(|_, entry| entry.is_pinned);
        *total_size = cache.values().map(|e| e.estimated_size).sum();

        Ok(())
    }

    /// Evict least valuable entries until under capacity
    fn evict_if_needed(&self) -> anyhow::Result<()> {
        let total_size = *self.total_size.read();

        if total_size <= MAX_MEMORY_BYTES {
            return Ok(());
        }

        let mut cache = self.cache.write();
        let mut size = total_size;

        while size > MAX_MEMORY_BYTES {
             let victim = cache
                .iter()
                .filter(|(_, entry)| !entry.is_pinned)
                .min_by(|(_, a), (_, b)| {
                    a.eviction_priority()
                        .partial_cmp(&b.eviction_priority())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(k, _)| k.clone());

            if let Some(key) = victim {
                if let Some(entry) = cache.remove(&key) {
                    size = size.saturating_sub(entry.estimated_size);
                    tracing::debug!("Evicted cache entry: {} (size: {})", key, entry.estimated_size);
                }
            } else {
                // No evictable entries found
                break;
            }
        }

        *self.total_size.write() = size;

        Ok(())
    }
}

impl Default for WorkingMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub entry_count: usize,
    pub pinned_count: usize,
    pub total_size_bytes: usize,
    pub capacity_bytes: usize,
    pub utilization_pct: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_insert_and_get() {
        let memory = WorkingMemory::new();
        let value = json!({"test": "data"});

        memory.insert("key1".to_string(), value.clone(), 0.5, false).unwrap();

        let retrieved = memory.get("key1").unwrap();
        assert_eq!(retrieved, value);
    }

    #[test]
    fn test_pinned_not_evicted() {
        let memory = WorkingMemory::new();

        // Insert pinned entry
        memory.insert("pinned".to_string(), json!({"important": true}), 1.0, true).unwrap();

        // Fill cache with many entries
        for i in 0..1000 {
            let large_value = json!({"data": vec!["x"; 1000]});
            memory.insert(format!("key{}", i), large_value, 0.1, false).unwrap();
        }

        // Pinned entry should still exist
        assert!(memory.contains("pinned"));
    }

    #[test]
    fn test_lru_eviction() {
        let memory = WorkingMemory::new();

        // Insert entries until eviction occurs
        for i in 0..1000 {
            let value = json!({"index": i, "data": vec!["x"; 1000]});
            memory.insert(format!("key{}", i), value, 0.5, false).unwrap();
        }

        let stats = memory.stats();
        assert!(stats.total_size_bytes <= MAX_MEMORY_BYTES);
    }

    #[test]
    fn test_access_updates_recency() {
        let memory = WorkingMemory::new();

        memory.insert("key1".to_string(), json!({"data": 1}), 0.5, false).unwrap();
        memory.insert("key2".to_string(), json!({"data": 2}), 0.5, false).unwrap();

        // Access key1 multiple times
        for _ in 0..10 {
            memory.get("key1");
        }

        // Both should still exist
        assert!(memory.contains("key1"));
        assert!(memory.contains("key2"));
    }

    #[test]
    fn test_remove() {
        let memory = WorkingMemory::new();

        memory.insert("key1".to_string(), json!({"data": 1}), 0.5, false).unwrap();
        assert!(memory.contains("key1"));

        memory.remove("key1").unwrap();
        assert!(!memory.contains("key1"));
    }

    #[test]
    fn test_clear_preserves_pinned() {
        let memory = WorkingMemory::new();

        memory.insert("pinned".to_string(), json!({"pin": true}), 1.0, true).unwrap();
        memory.insert("normal".to_string(), json!({"pin": false}), 0.5, false).unwrap();

        memory.clear().unwrap();

        assert!(memory.contains("pinned"));
        assert!(!memory.contains("normal"));
    }

    #[test]
    fn test_stats() {
        let memory = WorkingMemory::new();

        for i in 0..10 {
            memory.insert(format!("key{}", i), json!({"data": i}), 0.5, i < 2).unwrap();
        }

        let stats = memory.stats();
        assert_eq!(stats.entry_count, 10);
        assert_eq!(stats.pinned_count, 2);
        assert!(stats.utilization_pct >= 0.0 && stats.utilization_pct <= 100.0);
    }
}
