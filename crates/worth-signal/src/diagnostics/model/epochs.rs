mod retained_charge;

use serde::{Deserialize, Serialize};

use crate::data::checkpoint::CheckpointBarrier;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventSubscriberOutcomeKind {
    Committed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventSubscriberOutcome {
    pub subscriber_name: String,
    pub outcome: EventSubscriberOutcomeKind,
    #[serde(default)]
    pub requires_data_ids: Vec<String>,
    #[serde(default)]
    pub provides_data_ids: Vec<String>,
    #[serde(default)]
    pub staged_data_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventEpochOutcome {
    Committed,
    RolledBack,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventEpochSummary {
    pub ordinal: u32,
    pub barrier: CheckpointBarrier,
    pub emitted_event_count: u32,
    pub subscriber_count: u32,
    pub committed_subscriber_count: u32,
    pub failed_subscriber_position: Option<u32>,
    #[serde(default)]
    pub subscriber_outcomes: Vec<EventSubscriberOutcome>,
    pub outcome: EventEpochOutcome,
    pub failure_subscriber: Option<String>,
    pub message: Option<String>,
}
