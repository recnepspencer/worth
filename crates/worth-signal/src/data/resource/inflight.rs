use serde::{Deserialize, Serialize};

use crate::data::temporal::ClockTick;
use crate::data::temporal::TemporalDuration;
use crate::data::temporal::TemporalWakeId;

use super::descriptor::ResourceDescriptorId;
use super::lifecycle::{ResourceLifecycleClass, ResourceLifecycleOrdinal};
use super::managed_queue::{ResourceManagedQueueState, ResourceQueuePressureObservation};
use super::policy::ResourceTimeoutOutcomeClass;
use super::policy_registry::ResourcePolicyDigest;
use super::request::{
    ResourceAttemptId, ResourceBranchEpoch, ResourceGeneration, ResourceNodeId,
    ResourceRequestHandle, ResourceRequestIntentDigest,
};
use super::revalidation::{
    ResourceRevalidationFreshnessClass, ResourceRevalidationFreshnessDecision,
};
use super::timeout::ResourceTimeoutDeadlineAuthority;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceInFlightStatus {
    Active,
    Fulfilled,
    Rejected,
    Superseded,
    Cancelled,
    TimedOut,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InFlightResourceRequest {
    handle: ResourceRequestHandle,
    node: ResourceNodeId,
    descriptor_id: ResourceDescriptorId,
    generation: ResourceGeneration,
    attempt: ResourceAttemptId,
    request_intent_digest: ResourceRequestIntentDigest,
    generation_started_tick: ClockTick,
    lifecycle: ResourceLifecycleClass,
    lifecycle_ordinal: ResourceLifecycleOrdinal,
    status: ResourceInFlightStatus,
    timeout_wake_id: Option<TemporalWakeId>,
    timeout_duration: Option<TemporalDuration>,
    timeout_due_tick: Option<ClockTick>,
    timeout_outcome_class: ResourceTimeoutOutcomeClass,
    timeout_deadline_authority: ResourceTimeoutDeadlineAuthority,
    timeout_decision_digest: ResourcePolicyDigest,
    revalidation_freshness_class: Option<ResourceRevalidationFreshnessClass>,
    revalidation_freshness_digest: Option<String>,
    revalidation_policy_decision_digest: Option<ResourcePolicyDigest>,
    superseded_by: Option<ResourceRequestHandle>,
    #[serde(default)]
    managed_queue: Option<ResourceManagedQueueState>,
    // Runtime custody only: diagnostic Serde values cannot be installed into
    // ResourceRuntimeState. Genuine state clones/rekeying preserve this bit.
    #[serde(skip)]
    process_local: bool,
}

impl InFlightResourceRequest {
    pub(crate) fn new(
        handle: ResourceRequestHandle,
        node: ResourceNodeId,
        descriptor_id: ResourceDescriptorId,
        generation: ResourceGeneration,
        attempt: ResourceAttemptId,
        request_intent_digest: ResourceRequestIntentDigest,
        generation_started_tick: ClockTick,
        lifecycle_ordinal: ResourceLifecycleOrdinal,
        timeout_duration: Option<TemporalDuration>,
        timeout_due_tick: Option<ClockTick>,
        timeout_outcome_class: ResourceTimeoutOutcomeClass,
        timeout_deadline_authority: ResourceTimeoutDeadlineAuthority,
        timeout_decision_digest: ResourcePolicyDigest,
    ) -> Self {
        Self {
            handle,
            node,
            descriptor_id,
            generation,
            attempt,
            request_intent_digest,
            generation_started_tick,
            lifecycle: ResourceLifecycleClass::Pending,
            lifecycle_ordinal,
            status: ResourceInFlightStatus::Active,
            timeout_wake_id: None,
            timeout_duration,
            timeout_due_tick,
            timeout_outcome_class,
            timeout_deadline_authority,
            timeout_decision_digest,
            revalidation_freshness_class: None,
            revalidation_freshness_digest: None,
            revalidation_policy_decision_digest: None,
            superseded_by: None,
            managed_queue: None,
            process_local: false,
        }
    }

    pub(crate) fn attach_timeout_wake(&mut self, wake_id: TemporalWakeId) {
        self.timeout_wake_id = Some(wake_id);
    }

    pub(crate) fn mark_process_local(&mut self) {
        self.process_local = true;
    }

    pub(crate) fn is_process_local(&self) -> bool {
        self.process_local
    }

    pub(crate) fn attach_revalidation_freshness(
        &mut self,
        freshness_decision: &ResourceRevalidationFreshnessDecision,
    ) {
        self.revalidation_freshness_class = Some(freshness_decision.class());
        self.revalidation_freshness_digest = Some(freshness_decision.freshness_digest().to_owned());
        self.revalidation_policy_decision_digest =
            Some(freshness_decision.policy_decision_digest().clone());
    }

    pub(crate) fn supersede(
        &mut self,
        lifecycle_ordinal: ResourceLifecycleOrdinal,
        replacing: ResourceRequestHandle,
    ) {
        self.lifecycle = ResourceLifecycleClass::Superseded;
        self.lifecycle_ordinal = lifecycle_ordinal;
        self.status = ResourceInFlightStatus::Superseded;
        self.superseded_by = Some(replacing);
    }

    pub(crate) fn cancel(&mut self, lifecycle_ordinal: ResourceLifecycleOrdinal) {
        self.lifecycle = ResourceLifecycleClass::Cancelled;
        self.lifecycle_ordinal = lifecycle_ordinal;
        self.status = ResourceInFlightStatus::Cancelled;
    }

    pub(crate) fn timeout(&mut self, lifecycle_ordinal: ResourceLifecycleOrdinal) {
        self.lifecycle = ResourceLifecycleClass::TimedOut;
        self.lifecycle_ordinal = lifecycle_ordinal;
        self.status = ResourceInFlightStatus::TimedOut;
    }

    pub(crate) fn reject(&mut self, lifecycle_ordinal: ResourceLifecycleOrdinal) {
        self.lifecycle = ResourceLifecycleClass::Rejected;
        self.lifecycle_ordinal = lifecycle_ordinal;
        self.status = ResourceInFlightStatus::Rejected;
    }

    pub(crate) fn fulfill(&mut self, lifecycle_ordinal: ResourceLifecycleOrdinal) {
        self.lifecycle = ResourceLifecycleClass::Fulfilled;
        self.lifecycle_ordinal = lifecycle_ordinal;
        self.status = ResourceInFlightStatus::Fulfilled;
    }

    pub(crate) fn refresh_branch_epoch(&mut self, branch_epoch: ResourceBranchEpoch) {
        self.handle = self.handle.with_branch_epoch(branch_epoch);
    }

    pub fn handle(&self) -> ResourceRequestHandle {
        self.handle
    }

    pub fn node(&self) -> ResourceNodeId {
        self.node
    }

    pub fn descriptor_id(&self) -> ResourceDescriptorId {
        self.descriptor_id
    }

    pub fn generation(&self) -> ResourceGeneration {
        self.generation
    }

    pub fn attempt(&self) -> ResourceAttemptId {
        self.attempt
    }

    pub fn request_intent_digest(&self) -> &ResourceRequestIntentDigest {
        &self.request_intent_digest
    }

    pub fn generation_started_tick(&self) -> ClockTick {
        self.generation_started_tick
    }

    pub fn lifecycle(&self) -> ResourceLifecycleClass {
        self.lifecycle
    }

    pub fn lifecycle_ordinal(&self) -> ResourceLifecycleOrdinal {
        self.lifecycle_ordinal
    }

    pub fn status(&self) -> ResourceInFlightStatus {
        self.status
    }

    pub fn timeout_wake_id(&self) -> Option<TemporalWakeId> {
        self.timeout_wake_id
    }

    pub fn timeout_duration(&self) -> Option<TemporalDuration> {
        self.timeout_duration
    }

    pub fn timeout_outcome_class(&self) -> ResourceTimeoutOutcomeClass {
        self.timeout_outcome_class
    }

    pub fn timeout_due_tick(&self) -> Option<ClockTick> {
        self.timeout_due_tick
    }

    pub fn timeout_deadline_authority(&self) -> ResourceTimeoutDeadlineAuthority {
        self.timeout_deadline_authority
    }

    pub fn timeout_decision_digest(&self) -> &ResourcePolicyDigest {
        &self.timeout_decision_digest
    }

    pub fn revalidation_freshness_decision(&self) -> Option<ResourceRevalidationFreshnessDecision> {
        Some(ResourceRevalidationFreshnessDecision::new(
            self.revalidation_freshness_class?,
            self.revalidation_freshness_digest.clone()?,
            self.revalidation_policy_decision_digest.clone()?,
        ))
    }

    pub fn superseded_by(&self) -> Option<ResourceRequestHandle> {
        self.superseded_by
    }

    pub const fn managed_queue_pressure(&self) -> Option<ResourceQueuePressureObservation> {
        match self.managed_queue {
            Some(queue) => Some(queue.pressure()),
            None => None,
        }
    }

    pub(crate) const fn managed_queue(&self) -> Option<ResourceManagedQueueState> {
        self.managed_queue
    }

    pub(crate) fn managed_queue_mut(&mut self) -> Option<&mut ResourceManagedQueueState> {
        self.managed_queue.as_mut()
    }

    pub(crate) fn bind_managed_queue(&mut self, state: ResourceManagedQueueState) {
        self.managed_queue = Some(state);
    }
}
