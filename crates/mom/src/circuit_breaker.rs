use dashmap::DashMap;
use std::time::{Duration, Instant};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq)]
enum State {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Clone, Debug)]
struct CircuitState {
    state: State,
    failures: u32,
    last_failure: Instant,
    open_until: Option<Instant>,
    canary_in_flight: bool,
    canary_sent_at: Option<Instant>,
}

/// Status of a specific model's circuit
#[derive(Debug, Serialize)]
pub struct CircuitStatus {
    pub model: String,
    pub is_open: bool,
    pub state: String,
    pub failures: u32,
}

/// Hybrid circuit breaker for LLM providers
pub struct CircuitBreaker {
    /// Map of model_id -> State
    states: DashMap<String, CircuitState>,
    /// Number of failures before opening
    failure_threshold: u32,
    /// How long the circuit remains open
    reset_timeout: Duration,
    /// Sliding window for failure count
    window_duration: Duration,
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            states: DashMap::new(),
            failure_threshold: 3,
            reset_timeout: Duration::from_secs(300), // 5 minutes
            window_duration: Duration::from_secs(30), // 30 seconds
        }
    }

    /// Set custom thresholds (mostly for testing)
    pub fn with_thresholds(mut self, failures: u32, reset: Duration, window: Duration) -> Self {
        self.failure_threshold = failures;
        self.reset_timeout = reset;
        self.window_duration = window;
        self
    }

    /// Check if circuit is open for a model
    pub fn is_open(&self, model: &str) -> bool {
        let now = Instant::now();
        
        // First check with read lock
        if let Some(state_entry) = self.states.get(model) {
            match state_entry.state {
                State::Closed => return false,
                State::Open => {
                    if let Some(open_until) = state_entry.open_until {
                        if now < open_until {
                            return true;
                        }
                    }
                }
                State::HalfOpen => {
                    if state_entry.canary_in_flight {
                        // Check for stuck canary (60s timeout)
                        if let Some(sent_at) = state_entry.canary_sent_at {
                            if now.duration_since(sent_at) < Duration::from_secs(60) {
                                return true;
                            }
                        }
                    }
                    // If we reach here, either no canary in flight OR it timed out
                }
            }
        } else {
            return false;
        }

        // Transition logic (needs write lock)
        if let Some(mut state_guard) = self.states.get_mut(model) {
            match state_guard.state {
                State::Open => {
                    if let Some(open_until) = state_guard.open_until {
                        if now >= open_until {
                            state_guard.state = State::HalfOpen;
                            state_guard.canary_in_flight = true;
                            state_guard.canary_sent_at = Some(now);
                            tracing::info!(model = %model, "Circuit HALF-OPEN: Sending canary");
                            return false; 
                        }
                    }
                    true
                }
                State::HalfOpen => {
                    // Re-check flight status with timeout
                    let stuck = state_guard.canary_sent_at
                        .map(|t| now.duration_since(t) >= Duration::from_secs(60))
                        .unwrap_or(true);

                    if !state_guard.canary_in_flight || stuck {
                        state_guard.canary_in_flight = true;
                        state_guard.canary_sent_at = Some(now);
                        if stuck {
                            tracing::warn!(model = %model, "Canary timed out, sending replacement");
                        } else {
                            tracing::info!(model = %model, "Sending secondary canary");
                        }
                        return false;
                    }
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    /// Report a failure
    pub fn report_failure(&self, model: &str) {
        let mut state_guard = self.states.entry(model.to_string()).or_insert(CircuitState {
            state: State::Closed,
            failures: 0,
            last_failure: Instant::now(),
            open_until: None,
            canary_in_flight: false,
            canary_sent_at: None,
        });

        match state_guard.state {
            State::HalfOpen => {
                state_guard.state = State::Open;
                state_guard.open_until = Some(Instant::now() + self.reset_timeout);
                state_guard.failures = self.failure_threshold; 
                state_guard.canary_in_flight = false;
                state_guard.canary_sent_at = None;
                tracing::warn!(model = %model, "Canary failed! Circuit OPENED for 5m");
            }
            State::Open => {
                state_guard.open_until = Some(Instant::now() + self.reset_timeout);
            }
            State::Closed => {
                if state_guard.last_failure.elapsed() > self.window_duration {
                    state_guard.failures = 0;
                }

                state_guard.failures += 1;
                state_guard.last_failure = Instant::now();

                if state_guard.failures >= self.failure_threshold {
                    state_guard.state = State::Open;
                    state_guard.open_until = Some(Instant::now() + self.reset_timeout);
                    tracing::warn!(model = %model, "Circuit breaker OPENED for 5m");
                }
            }
        }
    }

    /// Report a success (reset failures)
    pub fn report_success(&self, model: &str) {
        if let Some(mut state_guard) = self.states.get_mut(model) {
            state_guard.state = State::Closed;
            state_guard.failures = 0;
            state_guard.open_until = None;
            state_guard.canary_in_flight = false;
            state_guard.canary_sent_at = None;
            tracing::info!(model = %model, "Circuit CLOSED (Success)");
        }
    }

    /// Get transparency on all circuit states
    pub fn get_status(&self) -> Vec<CircuitStatus> {
        self.states.iter().map(|kv| {
            let (model, state) = kv.pair();
            CircuitStatus {
                model: model.clone(),
                is_open: state.state != State::Closed,
                state: format!("{:?}", state.state),
                failures: state.failures,
            }
        }).collect()
    }

    /// Get count of open/half-open circuits
    pub fn count_open(&self) -> usize {
        self.states.iter()
            .filter(|kv| kv.state != State::Closed)
            .count()
    }

    /// Get total number of active models monitored
    pub fn total_models(&self) -> usize {
        self.states.len()
    }
}
