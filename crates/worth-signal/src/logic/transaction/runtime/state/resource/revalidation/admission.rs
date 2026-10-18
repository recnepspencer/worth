use super::super::timeout::plan::ScheduledResourceTimeoutAdmission;
use super::super::ResourceRuntimeState;
use super::preparation::{
    PreparedResourceRevalidation, PreparedResourceRevalidationDisposition,
    ResourceRevalidationAdmissionPreview,
};
use crate::data::resource::*;
use crate::data::telemetry::ResourceTelemetry;
use crate::state::SignalBranchId;

impl ResourceRuntimeState {
    pub(in crate::logic::transaction::runtime::state) fn admit_prepared_resource_revalidation(
        &mut self,
        prepared: PreparedResourceRevalidation,
        branch_id: SignalBranchId,
        generation_started_tick: crate::data::temporal::ClockTick,
        resolved_timeout: Option<ScheduledResourceTimeoutAdmission>,
        mut telemetry: Option<&mut ResourceTelemetry>,
    ) -> ResourceRevalidationReport {
        let PreparedResourceRevalidation {
            intent,
            revalidation_decision_digest,
            freshness_decision,
            evidence,
            disposition,
        } = prepared;
        if let PreparedResourceRevalidationDisposition::Coalesce {
            descriptor_id,
            active_request_id,
        } = disposition
        {
            return self.coalesce_revalidation(
                intent,
                descriptor_id,
                active_request_id,
                branch_id,
                generation_started_tick,
                freshness_decision,
                evidence,
                revalidation_decision_digest,
                resolved_timeout,
                telemetry.as_deref_mut(),
            );
        }
        let PreparedResourceRevalidationDisposition::Proceed { descriptor_id } = disposition else {
            unreachable!("coalescing disposition returned before request admission")
        };
        let temporal_wake_footprint = u32::from(resolved_timeout.is_some());
        let request_report = self.admit_resource_request_with_descriptor(
            match intent.transaction_deadline() {
                Some(deadline) => {
                    ResourceRequestIntent::with_transaction_deadline(intent.node(), deadline)
                }
                None => ResourceRequestIntent::new(intent.node()),
            },
            descriptor_id,
            branch_id,
            generation_started_tick,
            false,
            resolved_timeout,
            telemetry.as_deref_mut(),
        );
        let admitted_request = request_report.admitted_request();
        if let Some(in_flight) = self
            .in_flight_by_request
            .get_mut(&admitted_request.handle().request_id())
        {
            in_flight.attach_revalidation_freshness(&freshness_decision);
        }
        let supersession_record = request_report.supersession_record();
        let lifecycle = request_report.lifecycle();
        let transition = request_report.transition();
        let lifecycle_transition_count = request_report.performance().lifecycle_transition_count();

        if let Some(telemetry) = telemetry.as_deref_mut() {
            telemetry.resource_revalidation_admission_count += 1;
        }
        let performance = Self::record_boundary_performance_optional(
            telemetry,
            ResourceBoundaryPerformanceEnvelope::revalidation_admission(
                1,
                0,
                lifecycle_transition_count,
                temporal_wake_footprint,
            )
            .with_output_continuity_classification_width(
                request_report
                    .performance()
                    .output_continuity_classification_width(),
            ),
        );

        ResourceRevalidationReport::admitted(
            AdmittedResourceRevalidation::new(
                admitted_request,
                freshness_decision,
                evidence,
                None,
                supersession_record,
                revalidation_decision_digest,
            ),
            lifecycle,
            transition,
            performance,
        )
    }
    pub(in crate::logic::transaction::runtime::state::resource) fn preview_revalidation_admission(
        &self,
        intent: ResourceRevalidationIntent,
        freshness_decision: &ResourceRevalidationFreshnessDecision,
    ) -> ResourceRevalidationAdmissionPreview {
        let node = intent.node();
        let Some(descriptor_id) = self.descriptors_by_node.get(&node).copied() else {
            return ResourceRevalidationAdmissionPreview::Deny(
                ResourceRevalidationDenialClass::UndeclaredResourceNode,
            );
        };

        let request_intent = match intent.transaction_deadline() {
            Some(deadline) => ResourceRequestIntent::with_transaction_deadline(node, deadline),
            None => ResourceRequestIntent::new(node),
        };
        let request_intent_digest = request_intent.canonical_digest();
        if let Some(active_request_id) = self.active_request_by_node.get(&node).copied()
            && let Some(active_in_flight) = self.in_flight_by_request.get(&active_request_id)
            && active_in_flight.status() == ResourceInFlightStatus::Active
            && active_in_flight.lifecycle() == ResourceLifecycleClass::Pending
            && active_in_flight.request_intent_digest() == &request_intent_digest
            && active_in_flight
                .revalidation_freshness_decision()
                .as_ref()
                .is_some_and(|existing| {
                    existing.class() == freshness_decision.class()
                        && existing.freshness_digest() == freshness_decision.freshness_digest()
                })
        {
            return ResourceRevalidationAdmissionPreview::Coalesce {
                descriptor_id,
                active_request_id,
            };
        }

        self.validate_resource_revalidation_intent(intent)
            .map(ResourceRevalidationAdmissionPreview::Deny)
            .unwrap_or(ResourceRevalidationAdmissionPreview::Proceed { descriptor_id })
    }
}
