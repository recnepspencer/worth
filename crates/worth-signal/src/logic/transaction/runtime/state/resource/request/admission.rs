use super::super::timeout::plan::ScheduledResourceTimeoutAdmission;
use super::super::ResourceRuntimeState;
use super::coalescing::RequestIntentCoalescingInput;
use crate::data::resource::*;
use crate::data::telemetry::ResourceTelemetry;
use crate::data::temporal::ClockTick;
use crate::state::SignalBranchId;

struct ResourceRequestAdmissionInput {
    node: ResourceNodeId,
    descriptor_id: ResourceDescriptorId,
    branch_id: SignalBranchId,
    generation_started_tick: ClockTick,
    request_intent_digest: ResourceRequestIntentDigest,
    resolved_timeout: Option<ScheduledResourceTimeoutAdmission>,
}

struct ResourceRequestTimeoutBinding {
    timeout_duration: Option<crate::data::temporal::TemporalDuration>,
    timeout_due_tick: Option<ClockTick>,
    timeout_outcome_class: ResourceTimeoutOutcomeClass,
    timeout_deadline_authority: ResourceTimeoutDeadlineAuthority,
    timeout_decision_digest: ResourcePolicyDigest,
    timeout_wake_id: Option<crate::data::temporal::TemporalWakeId>,
}

struct PreparedResourceRequestAdmission {
    admitted: AdmittedResourceRequest,
    in_flight: InFlightResourceRequest,
    lifecycle: ResourceLifecycleSummary,
    transition: ResourceLifecycleTransition,
    supersession: Option<ResourceSupersessionRecord>,
    timeout_wake_id: Option<crate::data::temporal::TemporalWakeId>,
}

impl ResourceRequestTimeoutBinding {
    fn from_scheduled(resolved_timeout: Option<ScheduledResourceTimeoutAdmission>) -> Self {
        match resolved_timeout {
            Some(timeout) => Self {
                timeout_duration: Some(timeout.timeout_duration),
                timeout_due_tick: Some(timeout.due_tick),
                timeout_outcome_class: timeout.outcome_class,
                timeout_deadline_authority: timeout.deadline_authority,
                timeout_decision_digest: timeout.decision_digest,
                timeout_wake_id: Some(timeout.wake_id),
            },
            None => Self {
                timeout_duration: None,
                timeout_due_tick: None,
                timeout_outcome_class: ResourceTimeoutOutcomeClass::Terminal,
                timeout_deadline_authority: ResourceTimeoutDeadlineAuthority::Descriptor,
                timeout_decision_digest: ResourcePolicyDigest::new(
                    "resource-timeout:disabled-admission-default",
                ),
                timeout_wake_id: None,
            },
        }
    }
}

impl ResourceRuntimeState {
    pub(in crate::logic::transaction::runtime::state) fn admit_resource_request(
        &mut self,
        intent: ResourceRequestIntent,
        branch_id: SignalBranchId,
        generation_started_tick: ClockTick,
        allow_intent_equivalence_coalescing: bool,
        resolved_timeout: Option<ScheduledResourceTimeoutAdmission>,
        mut telemetry: Option<&mut ResourceTelemetry>,
    ) -> Result<ResourceRequestAdmissionReport, crate::data::error::SignalError> {
        let node = intent.node();
        let descriptor_id = self
            .descriptors_by_node
            .get(&node)
            .copied()
            .ok_or_else(|| {
                if let Some(telemetry) = telemetry.as_deref_mut() {
                    telemetry.resource_undeclared_owner_denial_count += 1;
                }
                crate::data::error::SignalError::invalid_input(format!(
                    "cannot admit resource request for undeclared resource node {}",
                    node.node()
                ))
            })?;
        Ok(self.admit_resource_request_with_descriptor(
            intent,
            descriptor_id,
            branch_id,
            generation_started_tick,
            allow_intent_equivalence_coalescing,
            resolved_timeout,
            telemetry,
        ))
    }

    pub(in crate::logic::transaction::runtime::state) fn admit_resource_request_with_descriptor(
        &mut self,
        intent: ResourceRequestIntent,
        descriptor_id: ResourceDescriptorId,
        branch_id: SignalBranchId,
        generation_started_tick: ClockTick,
        allow_intent_equivalence_coalescing: bool,
        resolved_timeout: Option<ScheduledResourceTimeoutAdmission>,
        mut telemetry: Option<&mut ResourceTelemetry>,
    ) -> ResourceRequestAdmissionReport {
        let input = ResourceRequestAdmissionInput {
            node: intent.node(),
            descriptor_id,
            branch_id,
            generation_started_tick,
            request_intent_digest: intent.canonical_digest(),
            resolved_timeout,
        };
        if allow_intent_equivalence_coalescing {
            let coalescing_input = RequestIntentCoalescingInput::new(
                input.node,
                input.descriptor_id,
                input.request_intent_digest.clone(),
                input.branch_id,
                input.generation_started_tick,
            );
            if let Some(coalesced) = self
                .try_coalesce_equivalent_request_intent(coalescing_input, telemetry.as_deref_mut())
            {
                return coalesced;
            }
        }

        let admitted = self.issue_resource_request_admission_identity(&input);
        let supersession = self.supersede_active_request_for_node(
            input.node,
            admitted.handle(),
            input.descriptor_id,
            telemetry.as_deref_mut(),
        );
        let prepared = self.prepare_resource_request_admission(
            input,
            admitted,
            supersession,
            telemetry.as_deref_mut(),
        );
        self.install_resource_request_admission(prepared, telemetry)
    }

    fn issue_resource_request_admission_identity(
        &mut self,
        input: &ResourceRequestAdmissionInput,
    ) -> AdmittedResourceRequest {
        let request_id = self.issue_request_id();
        let generation = self.issue_generation();
        AdmittedResourceRequest::new(
            request_id,
            generation,
            ResourceBranchEpoch::new(input.branch_id, self.restore_epoch),
            ResourceAttemptId::ZERO,
        )
    }

    fn prepare_resource_request_admission(
        &mut self,
        input: ResourceRequestAdmissionInput,
        admitted: AdmittedResourceRequest,
        supersession: Option<ResourceSupersessionRecord>,
        mut telemetry: Option<&mut ResourceTelemetry>,
    ) -> PreparedResourceRequestAdmission {
        let from = self
            .lifecycle_by_node
            .get(&input.node)
            .copied()
            .map(ResourceLifecycleSummary::lifecycle)
            .unwrap_or(ResourceLifecycleClass::Unrequested);
        let ordinal = self.issue_lifecycle_ordinal();
        let output_continuity = self.pending_output_continuity_for_node_optional(
            input.node,
            input.descriptor_id,
            telemetry.as_deref_mut(),
        );
        let lifecycle = ResourceLifecycleSummary::new(
            input.node,
            ResourceLifecycleClass::Pending,
            output_continuity,
            ordinal,
        );
        let transition = ResourceLifecycleTransition::new(
            input.node,
            from,
            ResourceLifecycleClass::Pending,
            ResourceLifecycleTransitionKind::RequestAdmitted,
            ordinal,
            output_continuity,
        );
        let timeout = ResourceRequestTimeoutBinding::from_scheduled(input.resolved_timeout);
        let mut in_flight = InFlightResourceRequest::new(
            admitted.handle(),
            input.node,
            input.descriptor_id,
            admitted.handle().generation(),
            ResourceAttemptId::ZERO,
            input.request_intent_digest,
            input.generation_started_tick,
            ordinal,
            timeout.timeout_duration,
            timeout.timeout_due_tick,
            timeout.timeout_outcome_class,
            timeout.timeout_deadline_authority,
            timeout.timeout_decision_digest,
        );
        if let Some(wake_id) = timeout.timeout_wake_id {
            in_flight.attach_timeout_wake(wake_id);
            if let Some(telemetry) = telemetry {
                telemetry.resource_timeout_temporal_wake_footprint = telemetry
                    .resource_timeout_temporal_wake_footprint
                    .saturating_add(1);
            }
        }
        PreparedResourceRequestAdmission {
            admitted,
            in_flight,
            lifecycle,
            transition,
            supersession,
            timeout_wake_id: timeout.timeout_wake_id,
        }
    }

    fn install_resource_request_admission(
        &mut self,
        prepared: PreparedResourceRequestAdmission,
        mut telemetry: Option<&mut ResourceTelemetry>,
    ) -> ResourceRequestAdmissionReport {
        let request_id = prepared.admitted.handle().request_id();
        let node = prepared.in_flight.node();
        self.in_flight_by_request
            .insert(request_id, prepared.in_flight);
        self.active_request_by_node.insert(node, request_id);
        self.stale_after_wake_by_node.remove(&node);
        self.lifecycle_by_node.insert(node, prepared.lifecycle);
        self.clear_latest_denied_completion_for_node(node);

        if let Some(telemetry) = telemetry.as_deref_mut() {
            telemetry.resource_request_admission_count += 1;
            telemetry.resource_in_flight_request_count = self.in_flight_by_request.len() as u64;
            telemetry.resource_in_flight_frontier_width = telemetry
                .resource_in_flight_frontier_width
                .max(self.active_request_by_node.len() as u64);
        }

        let lifecycle_transition_count = u32::from(prepared.supersession.is_some()) + 1;
        let density_strategy =
            ResourceDensityStrategy::request_pressure(self.in_flight_by_request.len() as u32);
        let supersession_visibility_width = prepared
            .supersession
            .as_ref()
            .map(|record| {
                u32::from(
                    record.lifecycle_transition().output_continuity()
                        != ResourceOutputContinuity::NoPriorOutput,
                )
            })
            .unwrap_or(0);
        let performance = Self::record_boundary_performance_optional(
            telemetry,
            ResourceBoundaryPerformanceEnvelope::request_admission(
                1,
                0,
                lifecycle_transition_count,
            )
            .with_temporal_wake_footprint(u32::from(prepared.timeout_wake_id.is_some()))
            .with_density_strategy(density_strategy)
            .with_output_continuity_classification_width(1 + supersession_visibility_width),
        );
        ResourceRequestAdmissionReport::new(
            prepared.admitted,
            prepared.lifecycle,
            prepared.transition,
            prepared.supersession,
            None,
            performance,
        )
    }
}
