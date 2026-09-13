use super::retry::budget::ResourceRetryBudgetLedger;
use crate::data::resource::{
    AsyncDenialId, DeniedResourceCompletion, FrozenResourcePolicyRegistry, InFlightResourceRequest,
    LoweredResourceDescriptor, ResourceBranchRestoreReport, ResourceCancellationOrdinal,
    ResourceCompletionOrdinal, ResourceDescriptorId, ResourceGeneration, ResourceLifecycleOrdinal,
    ResourceLifecycleSummary, ResourceNodeId, ResourceRejectionOrdinal, ResourceRequestId,
    ResourceRetainedDeniedCompletionAvailability, ResourceRetainedHistoryAvailability,
    ResourceRetainedRetryLineageAvailability, ResourceRetryOrdinal,
    ResourceSafePointObservationOrdinal, ResourceSupersessionOrdinal, ResourceTimeoutOrdinal,
    RetainedResourceRetryLineage, ScheduledResourceRetry,
};
use crate::data::temporal::TemporalWakeId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::logic::transaction::runtime) struct ResourceRuntimeState {
    pub(super) next_descriptor_id: ResourceDescriptorId,
    pub(super) next_request_id: ResourceRequestId,
    pub(super) next_generation: ResourceGeneration,
    pub(super) next_lifecycle_ordinal: ResourceLifecycleOrdinal,
    pub(super) next_denial_id: AsyncDenialId,
    pub(super) next_completion_ordinal: ResourceCompletionOrdinal,
    pub(super) next_cancellation_ordinal: ResourceCancellationOrdinal,
    pub(super) next_timeout_ordinal: ResourceTimeoutOrdinal,
    pub(super) next_rejection_ordinal: ResourceRejectionOrdinal,
    pub(super) next_supersession_ordinal: ResourceSupersessionOrdinal,
    pub(super) next_retry_ordinal: ResourceRetryOrdinal,
    pub(super) next_safe_point_observation_ordinal: ResourceSafePointObservationOrdinal,
    pub(super) restore_epoch: u64,
    pub(super) policy_registry: FrozenResourcePolicyRegistry,
    pub(super) descriptors: crate::data::persistent_ord_map::PersistentOrdMap<
        ResourceDescriptorId,
        LoweredResourceDescriptor,
    >,
    pub(super) descriptors_by_node:
        crate::data::persistent_ord_map::PersistentOrdMap<ResourceNodeId, ResourceDescriptorId>,
    pub(super) lifecycle_by_node:
        crate::data::persistent_ord_map::PersistentOrdMap<ResourceNodeId, ResourceLifecycleSummary>,
    pub(super) in_flight_by_request: crate::data::persistent_ord_map::PersistentOrdMap<
        ResourceRequestId,
        InFlightResourceRequest,
    >,
    pub(super) retained_in_flight_history_by_request:
        crate::data::persistent_ord_map::PersistentOrdMap<
            ResourceRequestId,
            InFlightResourceRequest,
        >,
    pub(super) pruned_in_flight_history_by_request:
        crate::data::persistent_ord_map::PersistentOrdMap<
            ResourceRequestId,
            ResourceRetainedHistoryAvailability,
        >,
    pub(super) terminal_in_flight_by_request:
        crate::data::persistent_ord_set::PersistentOrdSet<ResourceRequestId>,
    pub(super) active_request_by_node:
        crate::data::persistent_ord_map::PersistentOrdMap<ResourceNodeId, ResourceRequestId>,
    pub(super) stale_after_wake_by_node:
        crate::data::persistent_ord_map::PersistentOrdMap<ResourceNodeId, TemporalWakeId>,
    pub(super) pending_retry_by_request: crate::data::persistent_ord_map::PersistentOrdMap<
        ResourceRequestId,
        ScheduledResourceRetry,
    >,
    pub(super) pending_retry_by_wake:
        crate::data::persistent_ord_map::PersistentOrdMap<TemporalWakeId, ResourceRequestId>,
    pub(super) pending_retry_by_node:
        crate::data::persistent_ord_map::PersistentOrdMap<ResourceNodeId, ScheduledResourceRetry>,
    pub(super) retained_retry_lineage_by_ordinal: crate::data::persistent_ord_map::PersistentOrdMap<
        ResourceRetryOrdinal,
        RetainedResourceRetryLineage,
    >,
    pub(super) pruned_retry_lineage_by_ordinal: crate::data::persistent_ord_map::PersistentOrdMap<
        ResourceRetryOrdinal,
        ResourceRetainedRetryLineageAvailability,
    >,
    pub(super) retry_budget_ledger: ResourceRetryBudgetLedger,
    pub(super) denied_completions:
        crate::data::persistent_ord_map::PersistentOrdMap<AsyncDenialId, DeniedResourceCompletion>,
    pub(super) pruned_denied_completions_by_id: crate::data::persistent_ord_map::PersistentOrdMap<
        AsyncDenialId,
        ResourceRetainedDeniedCompletionAvailability,
    >,
    pub(super) latest_denied_completion_by_node:
        crate::data::persistent_ord_map::PersistentOrdMap<ResourceNodeId, DeniedResourceCompletion>,
    pub(super) latest_branch_restore_report: Option<ResourceBranchRestoreReport>,
}

impl Default for ResourceRuntimeState {
    fn default() -> Self {
        Self {
            next_descriptor_id: ResourceDescriptorId::new(0),
            next_request_id: ResourceRequestId::new(0),
            next_generation: ResourceGeneration::new(0),
            next_lifecycle_ordinal: ResourceLifecycleOrdinal::ZERO,
            next_denial_id: AsyncDenialId::new(0),
            next_completion_ordinal: ResourceCompletionOrdinal::ZERO,
            next_cancellation_ordinal: ResourceCancellationOrdinal::ZERO,
            next_timeout_ordinal: ResourceTimeoutOrdinal::ZERO,
            next_rejection_ordinal: ResourceRejectionOrdinal::ZERO,
            next_supersession_ordinal: ResourceSupersessionOrdinal::ZERO,
            next_retry_ordinal: ResourceRetryOrdinal::ZERO,
            next_safe_point_observation_ordinal: ResourceSafePointObservationOrdinal::ZERO,
            restore_epoch: 0,
            policy_registry: FrozenResourcePolicyRegistry::built_in(),
            descriptors: Default::default(),
            descriptors_by_node: Default::default(),
            lifecycle_by_node: Default::default(),
            in_flight_by_request: Default::default(),
            retained_in_flight_history_by_request: Default::default(),
            pruned_in_flight_history_by_request: Default::default(),
            terminal_in_flight_by_request: crate::data::persistent_ord_set::PersistentOrdSet::new(),
            active_request_by_node: Default::default(),
            stale_after_wake_by_node: Default::default(),
            pending_retry_by_request: Default::default(),
            pending_retry_by_wake: Default::default(),
            pending_retry_by_node: Default::default(),
            retained_retry_lineage_by_ordinal: Default::default(),
            pruned_retry_lineage_by_ordinal: Default::default(),
            retry_budget_ledger: ResourceRetryBudgetLedger::default(),
            denied_completions: Default::default(),
            pruned_denied_completions_by_id: Default::default(),
            latest_denied_completion_by_node: Default::default(),
            latest_branch_restore_report: None,
        }
    }
}

impl ResourceRuntimeState {
    pub(in crate::logic::transaction::runtime) fn fork_persistent(&mut self) -> Self {
        Self {
            next_descriptor_id: self.next_descriptor_id,
            next_request_id: self.next_request_id,
            next_generation: self.next_generation,
            next_lifecycle_ordinal: self.next_lifecycle_ordinal,
            next_denial_id: self.next_denial_id,
            next_completion_ordinal: self.next_completion_ordinal,
            next_cancellation_ordinal: self.next_cancellation_ordinal,
            next_timeout_ordinal: self.next_timeout_ordinal,
            next_rejection_ordinal: self.next_rejection_ordinal,
            next_supersession_ordinal: self.next_supersession_ordinal,
            next_retry_ordinal: self.next_retry_ordinal,
            next_safe_point_observation_ordinal: self.next_safe_point_observation_ordinal,
            restore_epoch: self.restore_epoch,
            policy_registry: self.policy_registry.clone(),
            descriptors: self.descriptors.fork_persistent(),
            descriptors_by_node: self.descriptors_by_node.fork_persistent(),
            lifecycle_by_node: self.lifecycle_by_node.fork_persistent(),
            in_flight_by_request: self.in_flight_by_request.fork_persistent(),
            retained_in_flight_history_by_request: self
                .retained_in_flight_history_by_request
                .fork_persistent(),
            pruned_in_flight_history_by_request: self
                .pruned_in_flight_history_by_request
                .fork_persistent(),
            terminal_in_flight_by_request: self.terminal_in_flight_by_request.fork_persistent(),
            active_request_by_node: self.active_request_by_node.fork_persistent(),
            stale_after_wake_by_node: self.stale_after_wake_by_node.fork_persistent(),
            pending_retry_by_request: self.pending_retry_by_request.fork_persistent(),
            pending_retry_by_wake: self.pending_retry_by_wake.fork_persistent(),
            pending_retry_by_node: self.pending_retry_by_node.fork_persistent(),
            retained_retry_lineage_by_ordinal: self
                .retained_retry_lineage_by_ordinal
                .fork_persistent(),
            pruned_retry_lineage_by_ordinal: self.pruned_retry_lineage_by_ordinal.fork_persistent(),
            retry_budget_ledger: self.retry_budget_ledger.fork_persistent(),
            denied_completions: self.denied_completions.fork_persistent(),
            pruned_denied_completions_by_id: self.pruned_denied_completions_by_id.fork_persistent(),
            latest_denied_completion_by_node: self
                .latest_denied_completion_by_node
                .fork_persistent(),
            latest_branch_restore_report: self.latest_branch_restore_report,
        }
    }

    #[cfg(test)]
    pub(in crate::logic::transaction::runtime) fn fork_storage_identity(&self) -> Self {
        Self {
            next_descriptor_id: self.next_descriptor_id,
            next_request_id: self.next_request_id,
            next_generation: self.next_generation,
            next_lifecycle_ordinal: self.next_lifecycle_ordinal,
            next_denial_id: self.next_denial_id,
            next_completion_ordinal: self.next_completion_ordinal,
            next_cancellation_ordinal: self.next_cancellation_ordinal,
            next_timeout_ordinal: self.next_timeout_ordinal,
            next_rejection_ordinal: self.next_rejection_ordinal,
            next_supersession_ordinal: self.next_supersession_ordinal,
            next_retry_ordinal: self.next_retry_ordinal,
            next_safe_point_observation_ordinal: self.next_safe_point_observation_ordinal,
            restore_epoch: self.restore_epoch,
            policy_registry: self.policy_registry.clone(),
            descriptors: self.descriptors.fork_storage_identity(),
            descriptors_by_node: self.descriptors_by_node.fork_storage_identity(),
            lifecycle_by_node: self.lifecycle_by_node.fork_storage_identity(),
            in_flight_by_request: self.in_flight_by_request.fork_storage_identity(),
            retained_in_flight_history_by_request: self
                .retained_in_flight_history_by_request
                .fork_storage_identity(),
            pruned_in_flight_history_by_request: self
                .pruned_in_flight_history_by_request
                .fork_storage_identity(),
            terminal_in_flight_by_request: self
                .terminal_in_flight_by_request
                .fork_storage_identity(),
            active_request_by_node: self.active_request_by_node.fork_storage_identity(),
            stale_after_wake_by_node: self.stale_after_wake_by_node.fork_storage_identity(),
            pending_retry_by_request: self.pending_retry_by_request.fork_storage_identity(),
            pending_retry_by_wake: self.pending_retry_by_wake.fork_storage_identity(),
            pending_retry_by_node: self.pending_retry_by_node.fork_storage_identity(),
            retained_retry_lineage_by_ordinal: self
                .retained_retry_lineage_by_ordinal
                .fork_storage_identity(),
            pruned_retry_lineage_by_ordinal: self
                .pruned_retry_lineage_by_ordinal
                .fork_storage_identity(),
            retry_budget_ledger: self.retry_budget_ledger.fork_storage_identity(),
            denied_completions: self.denied_completions.fork_storage_identity(),
            pruned_denied_completions_by_id: self
                .pruned_denied_completions_by_id
                .fork_storage_identity(),
            latest_denied_completion_by_node: self
                .latest_denied_completion_by_node
                .fork_storage_identity(),
            latest_branch_restore_report: self.latest_branch_restore_report,
        }
    }
}

#[cfg(test)]
impl ResourceRuntimeState {
    pub(in crate::logic::transaction::runtime) fn shares_storage_with(&self, other: &Self) -> bool {
        self.policy_registry
            .shares_storage_with(&other.policy_registry)
            && self.descriptors.ptr_eq(&other.descriptors)
            && self.descriptors_by_node.ptr_eq(&other.descriptors_by_node)
            && self.lifecycle_by_node.ptr_eq(&other.lifecycle_by_node)
            && self
                .in_flight_by_request
                .ptr_eq(&other.in_flight_by_request)
            && self
                .retained_in_flight_history_by_request
                .ptr_eq(&other.retained_in_flight_history_by_request)
            && self
                .pruned_in_flight_history_by_request
                .ptr_eq(&other.pruned_in_flight_history_by_request)
            && self
                .terminal_in_flight_by_request
                .ptr_eq(&other.terminal_in_flight_by_request)
            && self
                .active_request_by_node
                .ptr_eq(&other.active_request_by_node)
            && self
                .stale_after_wake_by_node
                .ptr_eq(&other.stale_after_wake_by_node)
            && self
                .pending_retry_by_request
                .ptr_eq(&other.pending_retry_by_request)
            && self
                .pending_retry_by_wake
                .ptr_eq(&other.pending_retry_by_wake)
            && self
                .pending_retry_by_node
                .ptr_eq(&other.pending_retry_by_node)
            && self
                .retained_retry_lineage_by_ordinal
                .ptr_eq(&other.retained_retry_lineage_by_ordinal)
            && self
                .pruned_retry_lineage_by_ordinal
                .ptr_eq(&other.pruned_retry_lineage_by_ordinal)
            && self
                .retry_budget_ledger
                .shares_storage_with(&other.retry_budget_ledger)
            && self.denied_completions.ptr_eq(&other.denied_completions)
            && self
                .pruned_denied_completions_by_id
                .ptr_eq(&other.pruned_denied_completions_by_id)
            && self
                .latest_denied_completion_by_node
                .ptr_eq(&other.latest_denied_completion_by_node)
    }
}
