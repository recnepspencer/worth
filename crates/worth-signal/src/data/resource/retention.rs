use serde::Serialize;

use super::denial::AsyncDenialId;
use super::denial::CompletionDenialClass;
use super::lifecycle::ResourceLifecycleClass;
use super::policy::ResourceRetentionDecisionClass;
use super::policy_registry::{ResourcePolicyDescriptorId, ResourcePolicyDigest};
use super::request::{
    ResourceAttemptId, ResourceBranchEpoch, ResourceNodeId, ResourceRequestHandle,
    ResourceRequestId, ResourceRetryOrdinal,
};
use super::retry::{ResourceRetryReason, ScheduledResourceRetry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ResourceRetentionCompactionBudget {
    retained_lifecycle_history_limit: Option<u32>,
    retained_denied_completion_limit: Option<u32>,
    retained_retry_lineage_limit: Option<u32>,
    pruned_lifecycle_availability_limit: Option<u32>,
    pruned_denied_availability_limit: Option<u32>,
    pruned_retry_availability_limit: Option<u32>,
}

impl ResourceRetentionCompactionBudget {
    pub fn unbounded() -> Self {
        Self {
            retained_lifecycle_history_limit: None,
            retained_denied_completion_limit: None,
            retained_retry_lineage_limit: None,
            pruned_lifecycle_availability_limit: None,
            pruned_denied_availability_limit: None,
            pruned_retry_availability_limit: None,
        }
    }

    pub fn retained_history_limit_only(retained_lifecycle_history_limit: u32) -> Self {
        Self {
            retained_lifecycle_history_limit: Some(retained_lifecycle_history_limit),
            retained_denied_completion_limit: None,
            retained_retry_lineage_limit: None,
            pruned_lifecycle_availability_limit: None,
            pruned_denied_availability_limit: None,
            pruned_retry_availability_limit: None,
        }
    }

    pub fn with_retained_lifecycle_history_limit(
        mut self,
        retained_lifecycle_history_limit: u32,
    ) -> Self {
        self.retained_lifecycle_history_limit = Some(retained_lifecycle_history_limit);
        self
    }

    pub fn with_retained_denied_completion_limit(
        mut self,
        retained_denied_completion_limit: u32,
    ) -> Self {
        self.retained_denied_completion_limit = Some(retained_denied_completion_limit);
        self
    }

    pub fn with_retained_retry_lineage_limit(mut self, retained_retry_lineage_limit: u32) -> Self {
        self.retained_retry_lineage_limit = Some(retained_retry_lineage_limit);
        self
    }

    /// Opt-in diagnostic tails; absence outside the tail is never exact history.
    pub fn with_pruned_availability_limits(
        mut self,
        lifecycle: u32,
        denied: u32,
        retry: u32,
    ) -> Self {
        self.pruned_lifecycle_availability_limit = Some(lifecycle);
        self.pruned_denied_availability_limit = Some(denied);
        self.pruned_retry_availability_limit = Some(retry);
        self
    }

    pub fn retained_lifecycle_history_limit(self) -> Option<u32> {
        self.retained_lifecycle_history_limit
    }

    pub fn retained_denied_completion_limit(self) -> Option<u32> {
        self.retained_denied_completion_limit
    }

    pub fn retained_retry_lineage_limit(self) -> Option<u32> {
        self.retained_retry_lineage_limit
    }

    pub fn pruned_lifecycle_availability_limit(self) -> Option<u32> {
        self.pruned_lifecycle_availability_limit
    }

    pub fn pruned_denied_availability_limit(self) -> Option<u32> {
        self.pruned_denied_availability_limit
    }

    pub fn pruned_retry_availability_limit(self) -> Option<u32> {
        self.pruned_retry_availability_limit
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ResourceRetainedHistoryAvailabilityClass {
    TerminalSummaryOnly,
    CompactSuperseded,
    CompactCancelled,
    CompactTimedOut,
    PrunedByRetainedHistoryLimit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResourceRetainedHistoryAvailability {
    handle: ResourceRequestHandle,
    attempt: ResourceAttemptId,
    node: ResourceNodeId,
    lifecycle: ResourceLifecycleClass,
    class: ResourceRetainedHistoryAvailabilityClass,
    retention_descriptor_id: ResourcePolicyDescriptorId,
    retention_decision_class: ResourceRetentionDecisionClass,
    retention_decision_digest: ResourcePolicyDigest,
}

impl ResourceRetainedHistoryAvailability {
    pub(crate) fn new(
        handle: ResourceRequestHandle,
        attempt: ResourceAttemptId,
        node: ResourceNodeId,
        lifecycle: ResourceLifecycleClass,
        class: ResourceRetainedHistoryAvailabilityClass,
        retention_descriptor_id: ResourcePolicyDescriptorId,
        retention_decision_class: ResourceRetentionDecisionClass,
        retention_decision_digest: ResourcePolicyDigest,
    ) -> Self {
        Self {
            handle,
            attempt,
            node,
            lifecycle,
            class,
            retention_descriptor_id,
            retention_decision_class,
            retention_decision_digest,
        }
    }

    pub fn handle(&self) -> ResourceRequestHandle {
        self.handle
    }

    pub fn attempt(&self) -> ResourceAttemptId {
        self.attempt
    }

    pub fn node(&self) -> ResourceNodeId {
        self.node
    }

    pub fn lifecycle(&self) -> ResourceLifecycleClass {
        self.lifecycle
    }

    pub fn class(&self) -> ResourceRetainedHistoryAvailabilityClass {
        self.class
    }

    pub fn retention_descriptor_id(&self) -> ResourcePolicyDescriptorId {
        self.retention_descriptor_id
    }

    pub fn retention_decision_class(&self) -> ResourceRetentionDecisionClass {
        self.retention_decision_class
    }

    pub fn retention_decision_digest(&self) -> &ResourcePolicyDigest {
        &self.retention_decision_digest
    }

    pub(crate) fn with_branch_epoch(
        mut self,
        branch_epoch: super::request::ResourceBranchEpoch,
    ) -> Self {
        self.handle = self.handle.with_branch_epoch(branch_epoch);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ResourceRetainedDeniedCompletionAvailabilityClass {
    PrunedByRetainedDeniedCompletionLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ResourceRetainedDeniedCompletionAvailability {
    denial_id: AsyncDenialId,
    request_id: ResourceRequestId,
    node: Option<ResourceNodeId>,
    denial_class: CompletionDenialClass,
    class: ResourceRetainedDeniedCompletionAvailabilityClass,
}

impl ResourceRetainedDeniedCompletionAvailability {
    pub(crate) fn new(
        denial_id: AsyncDenialId,
        request_id: ResourceRequestId,
        node: Option<ResourceNodeId>,
        denial_class: CompletionDenialClass,
        class: ResourceRetainedDeniedCompletionAvailabilityClass,
    ) -> Self {
        Self {
            denial_id,
            request_id,
            node,
            denial_class,
            class,
        }
    }

    pub fn denial_id(self) -> AsyncDenialId {
        self.denial_id
    }

    pub fn request_id(self) -> ResourceRequestId {
        self.request_id
    }

    pub fn node(self) -> Option<ResourceNodeId> {
        self.node
    }

    pub fn denial_class(self) -> CompletionDenialClass {
        self.denial_class
    }

    pub fn class(self) -> ResourceRetainedDeniedCompletionAvailabilityClass {
        self.class
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RetainedResourceRetryLineage {
    previous: ResourceRequestHandle,
    retry_ordinal: ResourceRetryOrdinal,
    node: ResourceNodeId,
    reason: ResourceRetryReason,
    next_attempt: ResourceAttemptId,
    scheduled_delay: crate::data::temporal::TemporalDuration,
    policy_decision_digest: ResourcePolicyDigest,
}

impl RetainedResourceRetryLineage {
    pub(crate) fn from_scheduled(node: ResourceNodeId, scheduled: ScheduledResourceRetry) -> Self {
        Self {
            previous: scheduled.previous(),
            retry_ordinal: scheduled.retry_ordinal(),
            node,
            reason: scheduled.reason(),
            next_attempt: scheduled.next_attempt(),
            scheduled_delay: scheduled.scheduled_delay(),
            policy_decision_digest: scheduled.policy_decision_digest().clone(),
        }
    }

    pub fn previous(&self) -> ResourceRequestHandle {
        self.previous
    }

    pub fn retry_ordinal(&self) -> ResourceRetryOrdinal {
        self.retry_ordinal
    }

    pub fn node(&self) -> ResourceNodeId {
        self.node
    }

    pub fn reason(&self) -> ResourceRetryReason {
        self.reason
    }

    pub fn next_attempt(&self) -> ResourceAttemptId {
        self.next_attempt
    }

    pub fn scheduled_delay(&self) -> crate::data::temporal::TemporalDuration {
        self.scheduled_delay
    }

    pub fn policy_decision_digest(&self) -> &ResourcePolicyDigest {
        &self.policy_decision_digest
    }

    pub(crate) fn with_branch_epoch(mut self, branch_epoch: ResourceBranchEpoch) -> Self {
        self.previous = self.previous.with_branch_epoch(branch_epoch);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ResourceRetainedRetryLineageAvailabilityClass {
    PrunedByRetainedRetryLineageLimit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResourceRetainedRetryLineageAvailability {
    previous: ResourceRequestHandle,
    retry_ordinal: ResourceRetryOrdinal,
    node: ResourceNodeId,
    reason: ResourceRetryReason,
    next_attempt: ResourceAttemptId,
    scheduled_delay: crate::data::temporal::TemporalDuration,
    class: ResourceRetainedRetryLineageAvailabilityClass,
    policy_decision_digest: ResourcePolicyDigest,
}

impl ResourceRetainedRetryLineageAvailability {
    pub(crate) fn from_retained(
        retained: RetainedResourceRetryLineage,
        class: ResourceRetainedRetryLineageAvailabilityClass,
    ) -> Self {
        Self {
            previous: retained.previous,
            retry_ordinal: retained.retry_ordinal,
            node: retained.node,
            reason: retained.reason,
            next_attempt: retained.next_attempt,
            scheduled_delay: retained.scheduled_delay,
            class,
            policy_decision_digest: retained.policy_decision_digest,
        }
    }

    pub fn previous(&self) -> ResourceRequestHandle {
        self.previous
    }

    pub fn retry_ordinal(&self) -> ResourceRetryOrdinal {
        self.retry_ordinal
    }

    pub fn node(&self) -> ResourceNodeId {
        self.node
    }

    pub fn reason(&self) -> ResourceRetryReason {
        self.reason
    }

    pub fn next_attempt(&self) -> ResourceAttemptId {
        self.next_attempt
    }

    pub fn scheduled_delay(&self) -> crate::data::temporal::TemporalDuration {
        self.scheduled_delay
    }

    pub fn class(&self) -> ResourceRetainedRetryLineageAvailabilityClass {
        self.class
    }

    pub fn policy_decision_digest(&self) -> &ResourcePolicyDigest {
        &self.policy_decision_digest
    }

    pub(crate) fn with_branch_epoch(mut self, branch_epoch: ResourceBranchEpoch) -> Self {
        self.previous = self.previous.with_branch_epoch(branch_epoch);
        self
    }
}
