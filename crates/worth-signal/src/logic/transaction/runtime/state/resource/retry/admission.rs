use super::super::timeout::plan::ScheduledResourceTimeoutAdmission;
use super::super::ResourceRuntimeState;
use crate::data::resource::*;
use crate::data::telemetry::ResourceTelemetry;
use crate::data::temporal::{ClockTick, ReadyTemporalWake};
use crate::state::SignalBranchId;

#[derive(Debug)]
pub(in crate::logic::transaction::runtime::state) struct PreparedScheduledResourceRetry {
    pub(in crate::logic::transaction::runtime::state::resource) scheduled: ScheduledResourceRetry,
    pub(in crate::logic::transaction::runtime::state::resource) previous: InFlightResourceRequest,
}

impl PreparedScheduledResourceRetry {
    pub(in crate::logic::transaction::runtime::state) fn previous(
        &self,
    ) -> &InFlightResourceRequest {
        &self.previous
    }
}

struct RetryAdmissionInput {
    scheduled: ScheduledResourceRetry,
    previous: InFlightResourceRequest,
    ready_wake: ReadyTemporalWake,
    branch_id: SignalBranchId,
    generation_started_tick: ClockTick,
    resolved_timeout: Option<ScheduledResourceTimeoutAdmission>,
}

struct PreparedRetryAdmissionState {
    scheduled: ScheduledResourceRetry,
    previous: InFlightResourceRequest,
    admitted: AdmittedResourceRequest,
    ready_wake: ReadyTemporalWake,
    in_flight: InFlightResourceRequest,
    lifecycle: ResourceLifecycleSummary,
    transition: ResourceLifecycleTransition,
    scheduled_timeout_wake_footprint: u32,
}

struct InstalledRetryAdmission {
    scheduled: ScheduledResourceRetry,
    admitted: AdmittedResourceRequest,
    ready_wake: ReadyTemporalWake,
    lifecycle: ResourceLifecycleSummary,
    transition: ResourceLifecycleTransition,
    scheduled_timeout_wake_footprint: u32,
}

impl ResourceRuntimeState {
    pub(in crate::logic::transaction::runtime::state) fn admit_prepared_scheduled_resource_retry(
        &mut self,
        prepared: PreparedScheduledResourceRetry,
        ready_wake: ReadyTemporalWake,
        branch_id: SignalBranchId,
        generation_started_tick: ClockTick,
        resolved_timeout: Option<ScheduledResourceTimeoutAdmission>,
        mut telemetry: Option<&mut ResourceTelemetry>,
    ) -> ResourceRetryAdmissionReport {
        let input = RetryAdmissionInput {
            scheduled: prepared.scheduled,
            previous: prepared.previous,
            ready_wake,
            branch_id,
            generation_started_tick,
            resolved_timeout,
        };
        let prepared = self.prepare_retry_admission_state(input, telemetry.as_deref_mut());
        let installed = self.install_retry_admission_state(prepared, telemetry.as_deref_mut());
        if let Some(telemetry) = telemetry.as_deref_mut() {
            telemetry.resource_retry_temporal_wake_footprint = telemetry
                .resource_retry_temporal_wake_footprint
                .saturating_add(1);
        }
        let performance = Self::record_boundary_performance_optional(
            telemetry,
            ResourceBoundaryPerformanceEnvelope::retry_admission(
                1,
                0,
                1,
                1 + installed.scheduled_timeout_wake_footprint,
            )
            .with_output_continuity_classification_width(1),
        );
        ResourceRetryAdmissionReport::admitted(
            AdmittedResourceRetry::new(
                installed.scheduled,
                installed.admitted,
                installed.ready_wake,
            ),
            installed.lifecycle,
            installed.transition,
            performance,
        )
    }

    fn prepare_retry_admission_state(
        &mut self,
        input: RetryAdmissionInput,
        mut telemetry: Option<&mut ResourceTelemetry>,
    ) -> PreparedRetryAdmissionState {
        let retry_request_id = self.issue_request_id();
        let admitted = AdmittedResourceRequest::new(
            retry_request_id,
            input.previous.generation(),
            ResourceBranchEpoch::new(input.branch_id, self.restore_epoch),
            input.scheduled.next_attempt(),
        );
        let ordinal = self.issue_lifecycle_ordinal();
        let output_continuity = self.pending_output_continuity_for_node_optional(
            input.previous.node(),
            input.previous.descriptor_id(),
            telemetry.as_deref_mut(),
        );
        let lifecycle = ResourceLifecycleSummary::new(
            input.previous.node(),
            ResourceLifecycleClass::Pending,
            output_continuity,
            ordinal,
        );
        let transition = ResourceLifecycleTransition::new(
            input.previous.node(),
            ResourceLifecycleClass::TimedOut,
            ResourceLifecycleClass::Pending,
            ResourceLifecycleTransitionKind::RequestAdmitted,
            ordinal,
            output_continuity,
        );
        let (
            timeout_duration,
            timeout_due_tick,
            timeout_outcome_class,
            timeout_deadline_authority,
            timeout_decision_digest,
            timeout_wake_id,
        ) = match input.resolved_timeout {
            Some(timeout) => (
                Some(timeout.timeout_duration),
                Some(timeout.due_tick),
                timeout.outcome_class,
                timeout.deadline_authority,
                timeout.decision_digest,
                Some(timeout.wake_id),
            ),
            None => (
                input.previous.timeout_duration(),
                input.previous.timeout_due_tick(),
                input.previous.timeout_outcome_class(),
                input.previous.timeout_deadline_authority(),
                input.previous.timeout_decision_digest().clone(),
                input.previous.timeout_wake_id(),
            ),
        };
        let mut in_flight = InFlightResourceRequest::new(
            admitted.handle(),
            input.previous.node(),
            input.previous.descriptor_id(),
            input.previous.generation(),
            input.scheduled.next_attempt(),
            input.previous.request_intent_digest().clone(),
            input.generation_started_tick,
            ordinal,
            timeout_duration,
            timeout_due_tick,
            timeout_outcome_class,
            timeout_deadline_authority,
            timeout_decision_digest,
        );
        if input.previous.is_process_local() {
            in_flight.mark_process_local();
        }
        if let Some(wake_id) = timeout_wake_id {
            in_flight.attach_timeout_wake(wake_id);
            if let Some(telemetry) = telemetry {
                telemetry.resource_timeout_temporal_wake_footprint = telemetry
                    .resource_timeout_temporal_wake_footprint
                    .saturating_add(1);
            }
        }
        PreparedRetryAdmissionState {
            scheduled: input.scheduled,
            previous: input.previous,
            admitted,
            ready_wake: input.ready_wake,
            in_flight,
            lifecycle,
            transition,
            scheduled_timeout_wake_footprint: u32::from(timeout_wake_id.is_some()),
        }
    }

    fn install_retry_admission_state(
        &mut self,
        prepared: PreparedRetryAdmissionState,
        telemetry: Option<&mut ResourceTelemetry>,
    ) -> InstalledRetryAdmission {
        let PreparedRetryAdmissionState {
            scheduled,
            previous,
            admitted,
            ready_wake,
            in_flight,
            lifecycle,
            transition,
            scheduled_timeout_wake_footprint,
        } = prepared;
        let request_id = previous.handle().request_id();
        self.pending_retry_by_request.remove(&request_id);
        self.pending_retry_by_wake.remove(&ready_wake.id());
        self.pending_retry_by_node.remove(&previous.node());
        self.retain_retry_lineage(previous.node(), scheduled.clone());
        self.in_flight_by_request
            .insert(admitted.handle().request_id(), in_flight);
        self.active_request_by_node
            .insert(previous.node(), admitted.handle().request_id());
        self.lifecycle_by_node.insert(previous.node(), lifecycle);
        self.clear_latest_denied_completion_for_node(previous.node());
        if let Some(telemetry) = telemetry {
            telemetry.resource_retry_admission_count += 1;
            telemetry.resource_in_flight_request_count = self.in_flight_by_request.len() as u64;
            telemetry.resource_in_flight_frontier_width = telemetry
                .resource_in_flight_frontier_width
                .max(self.active_request_by_node.len() as u64);
        }
        InstalledRetryAdmission {
            scheduled,
            admitted,
            ready_wake,
            lifecycle,
            transition,
            scheduled_timeout_wake_footprint,
        }
    }
}
