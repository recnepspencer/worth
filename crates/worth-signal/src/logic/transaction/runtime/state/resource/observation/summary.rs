use super::super::ResourceRuntimeState;
use crate::data::resource::*;
use crate::data::telemetry::ResourceTelemetry;

impl ResourceRuntimeState {
    pub(in crate::logic::transaction::runtime::state::resource) fn current_lifecycle_summary(
        &self,
        node: ResourceNodeId,
    ) -> Option<ResourceLifecycleSummary> {
        self.lifecycle_by_node.get(&node).copied()
    }

    pub(in crate::logic::transaction::runtime) fn observed_resource_node_state(
        &self,
        node: ResourceNodeId,
    ) -> Option<ObservedResourceNodeState> {
        let descriptor_id = *self.descriptors_by_node.get(&node)?;
        let descriptor = self.descriptors.get(&descriptor_id)?;
        let summary = self.current_lifecycle_summary(node)?;
        let output_continuity = descriptor
            .observation_decision_plan()
            .includes_output_continuity()
            .then_some(summary.output_continuity());
        let denied_completion = descriptor
            .observation_decision_plan()
            .includes_denied_completion()
            .then(|| self.latest_denied_completion_for_node(node))
            .flatten();
        let scheduled_retry = descriptor
            .observation_decision_plan()
            .includes_retry_schedule()
            .then(|| self.scheduled_retry_for_node(node))
            .flatten();
        Some(ObservedResourceNodeState::new(
            node,
            summary.lifecycle(),
            summary.lifecycle_ordinal(),
            output_continuity,
            denied_completion,
            scheduled_retry,
            descriptor
                .observation_decision_plan()
                .decision_digest()
                .clone(),
        ))
    }

    pub(in crate::logic::transaction::runtime::state::resource) fn latest_denied_completion_for_node(
        &self,
        node: ResourceNodeId,
    ) -> Option<DeniedResourceCompletion> {
        self.latest_denied_completion_by_node.get(&node).copied()
    }

    pub(in crate::logic::transaction::runtime::state::resource) fn clear_latest_denied_completion_for_node(
        &mut self,
        node: ResourceNodeId,
    ) {
        self.latest_denied_completion_by_node.remove(&node);
    }

    pub(in crate::logic::transaction::runtime::state::resource) fn rebuild_latest_denied_completion_for_node(
        &mut self,
        node: ResourceNodeId,
    ) {
        let replacement = self
            .denied_completions
            .values()
            .filter(|denied| denied.node() == Some(node))
            .max_by_key(|denied| denied.denial_id().get())
            .copied();
        if let Some(denied) = replacement {
            self.latest_denied_completion_by_node.insert(node, denied);
        } else {
            self.latest_denied_completion_by_node.remove(&node);
        }
    }

    pub(in crate::logic::transaction::runtime::state::resource) fn scheduled_retry_for_node(
        &self,
        node: ResourceNodeId,
    ) -> Option<ScheduledResourceRetry> {
        self.pending_retry_by_node.get(&node).cloned()
    }

    pub(in crate::logic::transaction::runtime::state::resource) fn record_boundary_performance(
        telemetry: &mut ResourceTelemetry,
        envelope: ResourceBoundaryPerformanceEnvelope,
    ) -> ResourceBoundaryPerformanceEnvelope {
        telemetry.record_boundary_performance_envelope(envelope);
        envelope
    }

    pub(in crate::logic::transaction::runtime::state::resource) fn record_boundary_performance_optional(
        telemetry: Option<&mut ResourceTelemetry>,
        envelope: ResourceBoundaryPerformanceEnvelope,
    ) -> ResourceBoundaryPerformanceEnvelope {
        if let Some(telemetry) = telemetry {
            telemetry.record_boundary_performance_envelope(envelope);
        }
        envelope
    }

    pub(in crate::logic::transaction::runtime::state::resource) fn mark_terminal_in_flight(
        &mut self,
        request_id: ResourceRequestId,
    ) {
        if self.in_flight_by_request.contains_key(&request_id) {
            self.terminal_in_flight_by_request.insert(request_id);
        }
    }

    pub(in crate::logic::transaction::runtime::state::resource) fn terminal_in_flight_record(
        &self,
        request_id: ResourceRequestId,
    ) -> Option<InFlightResourceRequest> {
        self.in_flight_by_request
            .get(&request_id)
            .cloned()
            .filter(|in_flight| in_flight.lifecycle().is_terminal())
    }

    pub fn summary(&self) -> ResourceRuntimeSummary {
        let retained_history_unavailable_count = self
            .lifecycle_by_node
            .values()
            .filter(|summary| {
                summary.lifecycle() == ResourceLifecycleClass::RetainedHistoryUnavailable
            })
            .count()
            .saturating_add(self.pruned_in_flight_history_by_request.len())
            .saturating_add(
                usize::try_from(self.expired_lifecycle_availability.count()).unwrap_or(usize::MAX),
            );
        ResourceRuntimeSummary::new(
            self.descriptors.len(),
            self.descriptors_by_node.len(),
            self.in_flight_by_request.len(),
            self.active_request_by_node.len(),
            self.retained_in_flight_history_by_request.len(),
            retained_history_unavailable_count,
            self.denied_completions.len(),
            self.retained_retry_lineage_by_ordinal.len(),
            self.denied_completions.len(),
            self.pruned_denied_completions_by_id.len().saturating_add(
                usize::try_from(self.expired_denied_availability.count()).unwrap_or(usize::MAX),
            ),
            self.pruned_retry_lineage_by_ordinal.len().saturating_add(
                usize::try_from(self.expired_retry_availability.count()).unwrap_or(usize::MAX),
            ),
            self.next_descriptor_id,
        )
    }

    pub fn in_flight_request_optional(
        &self,
        handle: ResourceRequestHandle,
        telemetry: Option<&mut ResourceTelemetry>,
    ) -> Option<&InFlightResourceRequest> {
        if let Some(telemetry) = telemetry {
            telemetry.resource_hot_in_flight_lookup_count += 1;
        }
        self.in_flight_by_request
            .get(&handle.request_id())
            .filter(|in_flight| in_flight.handle() == handle)
    }
}
