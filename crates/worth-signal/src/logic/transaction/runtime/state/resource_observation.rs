use crate::data::resource::{
    ResourceBoundaryPerformanceEnvelope, ResourceNodeId, ResourceObservationBatchReport,
    ResourceObservationEvent,
};
use crate::logic::transaction::runtime::ObservationBoundaryOutcome;

use super::runtime_state::SignalRuntime;

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn latest_resource_observation_batch_report(
        &mut self,
    ) -> Option<ResourceObservationBatchReport> {
        self.assert_construction_state_access();
        let latest = self.observe().latest_observation_summary()?.clone();
        let mut events = Vec::new();
        let mut delivered_width = 0u32;
        let mut denied_width = 0u32;
        let mut coalescing_width = 0u32;
        let mut output_continuity_width = 0u32;
        let mut observation_policy_decision_count = 0u64;
        let mut denied_completion_observation_count = 0u64;
        let mut retry_schedule_observation_count = 0u64;

        for event in latest.boundary_events {
            let mut matched_resource_nodes = Vec::new();
            for node in event.matched_nodes.iter() {
                let Some(observed) = self
                    .resource
                    .observed_resource_node_state(ResourceNodeId::from_node(node))
                else {
                    continue;
                };
                if observed.output_continuity().is_some() {
                    output_continuity_width = output_continuity_width.saturating_add(1);
                }
                if observed.denied_completion().is_some() {
                    denied_completion_observation_count += 1;
                }
                if observed.scheduled_retry().is_some() {
                    retry_schedule_observation_count += 1;
                }
                observation_policy_decision_count += 1;
                matched_resource_nodes.push(observed);
            }
            if matched_resource_nodes.is_empty() {
                continue;
            }
            coalescing_width = coalescing_width
                .saturating_add((matched_resource_nodes.len() as u32).saturating_sub(1));
            match event.outcome {
                ObservationBoundaryOutcome::Delivered => {
                    delivered_width = delivered_width.saturating_add(1)
                }
                ObservationBoundaryOutcome::RollbackSuppressed
                | ObservationBoundaryOutcome::BranchLocalSuppressed => {
                    denied_width = denied_width.saturating_add(1)
                }
            }
            events.push(ResourceObservationEvent::new(
                event.observer_id,
                event.handle_id,
                event.policy,
                event.outcome,
                event.touched,
                event.recomputed,
                event.meaningful_change,
                event.trigger_matched,
                matched_resource_nodes,
            ));
        }

        if events.is_empty() {
            return None;
        }

        let performance = ResourceBoundaryPerformanceEnvelope::observation_materialization(
            events.len() as u32,
            delivered_width,
            denied_width,
        )
        .with_coalescing_width(coalescing_width)
        .with_output_continuity_classification_width(output_continuity_width);
        self.with_resource_telemetry(|telemetry| {
            telemetry.resource_observation_policy_decision_count +=
                observation_policy_decision_count;
            telemetry.resource_observation_candidate_width = telemetry
                .resource_observation_candidate_width
                .saturating_add(events.len() as u64);
            telemetry.resource_observation_coalesced_width = telemetry
                .resource_observation_coalesced_width
                .saturating_add(coalescing_width as u64);
            telemetry.resource_observation_delivered_width = telemetry
                .resource_observation_delivered_width
                .saturating_add(delivered_width as u64);
            telemetry.resource_denied_completion_observation_count +=
                denied_completion_observation_count;
            telemetry.resource_retry_schedule_observation_count += retry_schedule_observation_count;
            telemetry.record_boundary_performance_envelope(performance);
        });

        Some(ResourceObservationBatchReport::new(events, performance))
    }
}
