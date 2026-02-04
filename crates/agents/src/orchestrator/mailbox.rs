//! Prioritized Mailbox for SAGA Agents
//!
//! Handles backpressure by prioritizing critical messages (Commands/Dissolve)
//! over informational ones.

use zed42_blackboard::VoxMessage;
use std::collections::BinaryHeap;
use std::cmp::Ordering;

/// A message in the priority mailbox
#[derive(Debug, Clone)]
pub struct PrioritizedMessage {
    pub priority: u8,
    pub message: VoxMessage,
}

impl PartialEq for PrioritizedMessage {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for PrioritizedMessage {}

impl PartialOrd for PrioritizedMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher number = Higher Priority
        self.priority.cmp(&other.priority)
    }
}

/// Priority mailbox using a BinaryHeap for O(log n) push/pop
pub struct PriorityMailbox {
    heap: BinaryHeap<PrioritizedMessage>,
    capacity: usize,
}

impl PriorityMailbox {
    /// Create a new priority mailbox with a maximum capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            heap: BinaryHeap::with_capacity(capacity),
            capacity,
        }
    }

    /// Push a VOX message into the mailbox. Returns false if at capacity.
    pub fn push(&mut self, message: VoxMessage) -> bool {
        if self.heap.len() >= self.capacity {
            // Check if this message is higher priority than the lowest (which isn't easy with BinaryHeap)
            // Actually, BinaryHeap gives the MAX. So we can easily see the highest priority.
            // For now, let's just enforce a hard limit to prevent OOM.
            return false;
        }
        
        self.heap.push(PrioritizedMessage {
            priority: message.priority,
            message,
        });
        true
    }

    /// Pop the highest priority VOX message from the mailbox
    pub fn pop(&mut self) -> Option<VoxMessage> {
        self.heap.pop().map(|pm| pm.message)
    }

    /// Check if the mailbox is empty
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Get current size
    pub fn len(&self) -> usize {
        self.heap.len()
    }
}
