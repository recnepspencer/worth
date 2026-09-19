use worth_query_installation::facade::ApplicationSchema;

use super::bridge_denial::bridge_denial;
use super::progression::denial;
use super::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(super) fn evaluate_current_output_readiness(
        &self,
        producer_identity: &str,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        delivery: Option<&worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt>,
    ) -> Result<
        crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence,
        WorthQueryOutputDemandDenial,
    >{
        if self
            .primary_provider
            .take_failed_output_readiness_evaluation()
        {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::SchedulingDeferred,
                format!("{producer_identity}: injected transient readiness evaluation failure"),
            ));
        }
        let route = self
            .output_readiness_routes
            .get(producer_identity)
            .expect("installation requires one readiness route for every output producer");
        let producer = self
            .installed_producers
            .entries
            .get(producer_identity)
            .expect("readiness route must retain its installed producer");
        let source_record = producer.executor.readiness_record(receipt)?;
        #[cfg(feature = "test-primary-graph-faults")]
        let _held_world_observations = if self.primary_provider.take_readiness_snapshot_pressure() {
            Some(self.hold_world_snapshot_pressure_for_test(receipt))
        } else {
            None
        };
        let selected = self
            .on_branch(receipt.product_branch())
            .select()
            .map_err(|error| {
                WorthQueryOutputDemandDenial::product_selection(
                    error,
                    format!("{producer_identity}: readiness product selection"),
                )
            })?;
        let truth = crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalTruthBasis::from_selected(selected);
        let attempt = self
            .next_output_producer_attempt
            .fetch_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |current| current.checked_add(1),
            )
            .map_err(|_| {
                denial(
                    WorthQueryOutputDemandDenialKind::SchedulingRejected,
                    format!("{producer_identity}: readiness attempt identity exhausted"),
                )
            })?;
        let bridge = self.bridge.conditional();
        let execution = super::super::evaluate_output_readiness(
            &bridge,
            &route.lowering,
            &truth,
            producer_identity,
            source_record,
            attempt,
            attempt,
        )
        .map_err(|error| bridge_denial(producer_identity, error))?;
        let decision =
            crate::domain_computation::primary_graph::conditional_operation::classify_bridge_signal(
                &execution,
            );
        if decision != crate::domain_computation::primary_graph::WorthQueryConditionalSignalDecision::Eligible {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::SchedulingRejected,
                format!("{producer_identity}: readiness returned {decision:?}"),
            ));
        }
        Ok(crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence::from_execution(
            delivery,
            &execution,
        ))
    }
}
