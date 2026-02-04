//! Live SELECT subscription system using SurrealDB

use crate::{Message, MessageType, AgentId};

use anyhow::Result;
use tokio::sync::mpsc;

/// Live subscription handle
pub struct LiveSubscription {
    receiver: mpsc::Receiver<Message>,
    subscription_id: String,
}

impl LiveSubscription {
    /// Create a new subscription
    pub(crate) fn new(receiver: mpsc::Receiver<Message>, subscription_id: String) -> Self {
        Self {
            receiver,
            subscription_id,
        }
    }

    /// Get the subscription ID
    pub fn id(&self) -> &str {
        &self.subscription_id
    }

    /// Receive next message (blocking)
    pub async fn recv(&mut self) -> Option<Message> {
        self.receiver.recv().await
    }

    /// Try to receive message (non-blocking)
    pub fn try_recv(&mut self) -> Result<Option<Message>, mpsc::error::TryRecvError> {
        match self.receiver.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

/// Subscription filter for LIVE SELECT
#[derive(Debug, Clone)]
pub struct SubscriptionFilter {
    pub message_type: Option<MessageType>,
    pub from_agent: Option<AgentId>,
    pub to_agent: Option<AgentId>,
}

impl SubscriptionFilter {
    /// Subscribe to all messages
    pub fn all() -> Self {
        Self {
            message_type: None,
            from_agent: None,
            to_agent: None,
        }
    }

    /// Subscribe to specific message type
    pub fn message_type(message_type: MessageType) -> Self {
        Self {
            message_type: Some(message_type),
            from_agent: None,
            to_agent: None,
        }
    }

    /// Subscribe to messages from specific agent
    pub fn from_agent(agent_id: AgentId) -> Self {
        Self {
            message_type: None,
            from_agent: Some(agent_id),
            to_agent: None,
        }
    }

    /// Subscribe to messages to specific agent
    pub fn to_agent(agent_id: AgentId) -> Self {
        Self {
            message_type: None,
            from_agent: None,
            to_agent: Some(agent_id),
        }
    }

    /// Build SurrealDB LIVE SELECT query
    pub(crate) fn build_query(&self) -> String {
        let mut conditions = Vec::new();

        if let Some(msg_type) = &self.message_type {
            conditions.push(format!("message_type = '{:?}'", msg_type));
        }

        if let Some(from) = &self.from_agent {
            conditions.push(format!("from_agent = '{}'", from));
        }

        if let Some(to) = &self.to_agent {
            conditions.push(format!("to_agent = '{}'", to));
        }

        if conditions.is_empty() {
            "LIVE SELECT * FROM messages".to_string()
        } else {
            format!(
                "LIVE SELECT * FROM messages WHERE {}",
                conditions.join(" AND ")
            )
        }
    }
}

/// Subscription manager
pub struct SubscriptionManager {
    subscriptions: Vec<(String, mpsc::Sender<Message>)>,
}

impl SubscriptionManager {
    /// Create a new subscription manager
    pub fn new() -> Self {
        Self {
            subscriptions: Vec::new(),
        }
    }

    /// Subscribe to messages
    pub fn subscribe(&mut self, _filter: SubscriptionFilter) -> LiveSubscription {
        let (tx, rx) = mpsc::channel(100);
        let subscription_id = uuid::Uuid::new_v4().to_string();

        self.subscriptions.push((subscription_id.clone(), tx));

        LiveSubscription::new(rx, subscription_id)
    }

    /// Notify all subscriptions of a new message
    pub async fn notify(&mut self, message: Message) {
        // Remove closed subscriptions
        self.subscriptions.retain(|(_, tx)| !tx.is_closed());

        // Send to all active subscriptions
        for (_, tx) in &self.subscriptions {
            let _ = tx.send(message.clone()).await;
        }
    }

    /// Get active subscription count
    pub fn active_count(&self) -> usize {
        self.subscriptions
            .iter()
            .filter(|(_, tx)| !tx.is_closed())
            .count()
    }
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}
