use serde::{Deserialize, Serialize};

use crate::logic::transaction::{
    ObservationBoundaryOutcome, ObservationHandleId, ObservationPolicy, ObserverId,
};

use super::{
    DeniedResourceCompletion, ResourceBoundaryPerformanceEnvelope, ResourceLifecycleClass,
    ResourceLifecycleOrdinal, ResourceNodeId, ResourceOutputContinuity, ResourcePolicyDigest,
    ScheduledResourceRetry,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedResourceNodeState {
    node: ResourceNodeId,
    lifecycle: ResourceLifecycleClass,
    lifecycle_ordinal: ResourceLifecycleOrdinal,
    output_continuity: Option<ResourceOutputContinuity>,
    denied_completion: Option<DeniedResourceCompletion>,
    scheduled_retry: Option<ScheduledResourceRetry>,
    observation_decision_digest: ResourcePolicyDigest,
}

impl ObservedResourceNodeState {
    pub(crate) fn new(
        node: ResourceNodeId,
        lifecycle: ResourceLifecycleClass,
        lifecycle_ordinal: ResourceLifecycleOrdinal,
        output_continuity: Option<ResourceOutputContinuity>,
        denied_completion: Option<DeniedResourceCompletion>,
        scheduled_retry: Option<ScheduledResourceRetry>,
        observation_decision_digest: ResourcePolicyDigest,
    ) -> Self {
        Self {
            node,
            lifecycle,
            lifecycle_ordinal,
            output_continuity,
            denied_completion,
            scheduled_retry,
            observation_decision_digest,
        }
    }

    pub fn node(&self) -> ResourceNodeId {
        self.node
    }

    pub fn lifecycle(&self) -> ResourceLifecycleClass {
        self.lifecycle
    }

    pub fn lifecycle_ordinal(&self) -> ResourceLifecycleOrdinal {
        self.lifecycle_ordinal
    }

    pub fn output_continuity(&self) -> Option<ResourceOutputContinuity> {
        self.output_continuity
    }

    pub fn denied_completion(&self) -> Option<DeniedResourceCompletion> {
        self.denied_completion
    }

    pub fn scheduled_retry(&self) -> Option<&ScheduledResourceRetry> {
        self.scheduled_retry.as_ref()
    }

    pub fn observation_decision_digest(&self) -> &ResourcePolicyDigest {
        &self.observation_decision_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceObservationEvent {
    observer_id: ObserverId,
    handle_id: ObservationHandleId,
    policy: ObservationPolicy,
    outcome: ObservationBoundaryOutcome,
    #[serde(alias = "touched")]
    visited: bool,
    recomputed: bool,
    meaningful_change: bool,
    trigger_matched: bool,
    matched_resource_nodes: Vec<ObservedResourceNodeState>,
}

impl ResourceObservationEvent {
    pub(crate) fn new(
        observer_id: ObserverId,
        handle_id: ObservationHandleId,
        policy: ObservationPolicy,
        outcome: ObservationBoundaryOutcome,
        visited: bool,
        recomputed: bool,
        meaningful_change: bool,
        trigger_matched: bool,
        matched_resource_nodes: Vec<ObservedResourceNodeState>,
    ) -> Self {
        Self {
            observer_id,
            handle_id,
            policy,
            outcome,
            visited,
            recomputed,
            meaningful_change,
            trigger_matched,
            matched_resource_nodes,
        }
    }

    pub fn observer_id(&self) -> ObserverId {
        self.observer_id
    }

    pub fn handle_id(&self) -> ObservationHandleId {
        self.handle_id
    }

    pub fn policy(&self) -> ObservationPolicy {
        self.policy
    }

    pub fn outcome(&self) -> ObservationBoundaryOutcome {
        self.outcome
    }

    pub fn visited(&self) -> bool {
        self.visited
    }

    pub fn recomputed(&self) -> bool {
        self.recomputed
    }

    pub fn meaningful_change(&self) -> bool {
        self.meaningful_change
    }

    pub fn trigger_matched(&self) -> bool {
        self.trigger_matched
    }

    pub fn matched_resource_nodes(&self) -> &[ObservedResourceNodeState] {
        &self.matched_resource_nodes
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceObservationBatchReport {
    events: Vec<ResourceObservationEvent>,
    performance: ResourceBoundaryPerformanceEnvelope,
}

impl ResourceObservationBatchReport {
    pub(crate) fn new(
        events: Vec<ResourceObservationEvent>,
        performance: ResourceBoundaryPerformanceEnvelope,
    ) -> Self {
        Self {
            events,
            performance,
        }
    }

    pub fn events(&self) -> &[ResourceObservationEvent] {
        &self.events
    }

    pub fn performance(&self) -> ResourceBoundaryPerformanceEnvelope {
        self.performance
    }
}
