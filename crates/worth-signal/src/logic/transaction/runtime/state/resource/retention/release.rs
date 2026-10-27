use super::super::ResourceRuntimeState;
use crate::data::resource::*;

impl ResourceRuntimeState {
    pub(in crate::logic::transaction::runtime) fn mark_process_local_request(
        &mut self,
        handle: ResourceRequestHandle,
    ) {
        let actual = self
            .in_flight_by_request
            .get_mut(&handle.request_id())
            .expect("admitted ProcessLocal occurrence remains under the same cell lock");
        assert_eq!(actual.handle(), handle);
        actual.mark_process_local();
    }

    /// Preparation and removal are used under the same branch-cell lock. A
    /// caller cannot select unrelated history by node or global tail limits.
    pub(in crate::logic::transaction::runtime) fn prepare_process_local_release(
        &self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
    ) -> Option<InFlightResourceRequest> {
        let descriptor = self.descriptor_for_node(node)?;
        if descriptor.retention_policy()
            != &ResourceRetentionPolicyDeclaration::TerminalSummariesOnly
            || self.active_request_by_node.get(&node) == Some(&handle.request_id())
            || self
                .pending_retry_by_request
                .contains_key(&handle.request_id())
        {
            return None;
        }
        let terminal = self.terminal_in_flight_record(handle.request_id())?;
        (terminal.handle() == handle
            && terminal.node() == node
            && terminal.is_process_local()
            && terminal
                .managed_queue()
                .is_none_or(ResourceManagedQueueState::is_empty))
        .then_some(terminal)
    }

    /// The prepared record and its exact native obligations were checked before
    /// mutation. This handoff has no fallible work after removal.
    pub(in crate::logic::transaction::runtime) fn release_process_local_record(
        &mut self,
        terminal: InFlightResourceRequest,
        current: Option<ResourceRequestHandle>,
    ) {
        let id = terminal.handle().request_id();
        let descriptor = self
            .descriptor_for_node(terminal.node())
            .expect("prepared release retains its installed descriptor")
            .clone();
        let generation = terminal.generation();
        let custodian_shares_generation = current
            .filter(|handle| *handle != terminal.handle())
            .and_then(|handle| {
                self.in_flight_by_request
                    .get(&handle.request_id())
                    .filter(|actual| actual.handle() == handle && actual.node() == terminal.node())
            })
            .is_some_and(|actual| actual.generation() == generation);
        let active_shares_generation = self
            .active_request_by_node
            .get(&terminal.node())
            .and_then(|id| self.in_flight_by_request.get(id))
            .is_some_and(|active| active.generation() == generation);
        let pending_shares_generation = self
            .pending_retry_by_node
            .get(&terminal.node())
            .is_some_and(|retry| retry.previous().generation() == generation);
        if !custodian_shares_generation && !active_shares_generation && !pending_shares_generation {
            self.retry_budget_ledger
                .clear_request_generation(generation);
        }
        self.take_terminal_record(id);
        self.retained_in_flight_history_by_request.remove(&id);
        self.pruned_in_flight_history_by_request.remove(&id);
        self.absorb_released_lifecycle(Self::retention_availability_from_in_flight(
            &descriptor,
            terminal,
            ResourceRetainedHistoryAvailabilityClass::TerminalSummaryOnly,
        ));
    }

    pub(super) fn take_terminal_record(
        &mut self,
        request: ResourceRequestId,
    ) -> Option<InFlightResourceRequest> {
        self.terminal_in_flight_by_request.remove(&request);
        self.in_flight_by_request
            .remove(&request)
            .filter(|in_flight| in_flight.lifecycle().is_terminal())
    }

    pub(in crate::logic::transaction::runtime) fn release_process_local_denial(
        &mut self,
        node: ResourceNodeId,
        denied: DeniedResourceCompletion,
    ) -> bool {
        let availability = Self::pruned_denied_completion_availability(denied);
        let stored = self.denied_completions.get(&denied.denial_id()) == Some(&denied)
            || self
                .pruned_denied_completions_by_id
                .get(&denied.denial_id())
                == Some(&availability);
        if denied.node() != Some(node)
            || self.descriptor_for_node(node).is_none_or(|descriptor| {
                descriptor.retention_policy()
                    != &ResourceRetentionPolicyDeclaration::TerminalSummariesOnly
            })
            || !stored
        {
            return false;
        }
        self.denied_completions.remove(&denied.denial_id());
        self.pruned_denied_completions_by_id
            .remove(&denied.denial_id());
        // Expiring this selected latest observation does not reconstruct an
        // older diagnostic by scanning another producer's retained records.
        if self.latest_denied_completion_by_node.get(&node) == Some(&denied) {
            self.latest_denied_completion_by_node.remove(&node);
        }
        self.absorb_released_denial(availability);
        true
    }

    pub(in crate::logic::transaction::runtime) fn release_process_local_retry_lineage(
        &mut self,
        node: ResourceNodeId,
        scheduled: &ScheduledResourceRetry,
    ) {
        let ordinal = scheduled.retry_ordinal();
        if let Some(retained) = self.retained_retry_lineage_by_ordinal.get(&ordinal) {
            assert_eq!(retained.node(), node);
            assert_eq!(retained.previous(), scheduled.previous());
        }
        if let Some(retained) = self.retained_retry_lineage_by_ordinal.remove(&ordinal) {
            self.pruned_retry_lineage_by_ordinal.remove(&ordinal);
            self.absorb_released_retry(ResourceRetainedRetryLineageAvailability::from_retained(
                retained,
                ResourceRetainedRetryLineageAvailabilityClass::PrunedByRetainedRetryLineageLimit,
            ));
        } else {
            self.expire_retry_availability(ordinal);
        }
    }

    pub(in crate::logic::transaction::runtime) fn process_local_retry_lineage_matches(
        &self,
        node: ResourceNodeId,
        scheduled: &ScheduledResourceRetry,
    ) -> bool {
        let ordinal = scheduled.retry_ordinal();
        self.retained_retry_lineage_by_ordinal
            .get(&ordinal)
            .is_none_or(|value| value.node() == node && value.previous() == scheduled.previous())
            && self
                .pruned_retry_lineage_by_ordinal
                .get(&ordinal)
                .is_none_or(|value| {
                    value.node() == node && value.previous() == scheduled.previous()
                })
            && !self
                .pending_retry_by_request
                .contains_key(&scheduled.previous().request_id())
            && !self
                .pending_retry_by_wake
                .contains_key(&scheduled.backoff_wake_id())
    }
}
