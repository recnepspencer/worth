mod retained_charge;

use serde::{Deserialize, Serialize};

use super::super::super::state::{
    ObservationHandleId, ObservationPolicy, ObservedNodeSet, ObserverId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservationBoundaryOutcome {
    Delivered,
    RollbackSuppressed,
    BranchLocalSuppressed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommittedObservationEventSummary {
    pub observer_id: ObserverId,
    pub handle_id: ObservationHandleId,
    pub policy: ObservationPolicy,
    pub observed_nodes: ObservedNodeSet,
    pub matched_nodes: ObservedNodeSet,
    pub touched: bool,
    pub recomputed: bool,
    pub meaningful_change: bool,
    pub trigger_matched: bool,
    pub outcome: ObservationBoundaryOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ObservationBoundarySummary {
    pub classified_event_count: u32,
    pub trigger_matched_event_count: u32,
    pub delivered_event_count: u32,
    pub rollback_suppressed_event_count: u32,
    pub branch_local_suppressed_event_count: u32,
    pub boundary_events: Vec<CommittedObservationEventSummary>,
}
