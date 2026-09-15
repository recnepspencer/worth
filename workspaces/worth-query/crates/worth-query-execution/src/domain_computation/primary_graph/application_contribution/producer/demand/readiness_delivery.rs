use worth_query_installation::facade::ApplicationSchema;

use super::progression::{denial, scheduling_failed};
use super::{
    WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryProducerOutputFamily,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(super) fn finish_output_readiness_delivery<Family>(
        &self,
        interest: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandInterest,
        producer_identity: &str,
        pending: crate::domain_computation::primary_graph::application_output_demand::WorthQueryPendingOutputDelivery,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let route = self
            .output_readiness_routes
            .get(producer_identity)
            .expect("installation requires one readiness route for every output producer");
        if self
            .primary_provider
            .take_delayed_output_readiness_delivery()
        {
            self.output_demands
                .finish_delivery_pending(interest, pending);
            return Ok(WorthQueryOutputDemandAdvance::Pending);
        }
        let root = self
            .granular_invalidation_installation()
            .retain_product_shared_root();
        let crate::domain_computation::primary_graph::application_output_demand::WorthQueryPendingOutputDelivery {
            receipt,
            change,
        } = pending;
        let outcome = root.deliver_performed_relational_change(
            &route.lowering,
            route.delivery_dependency_ordinal,
            change,
        );
        let delivery = match outcome {
            Ok(crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChangeDeliveryOutcome::Success(delivery)) => delivery,
            Ok(outcome) => {
                let detail = format!("{producer_identity}: readiness delivery returned {outcome:?}");
                let change = outcome
                    .into_retry_change()
                    .expect("non-success delivery returns its performed change");
                self.output_demands.finish_delivery_pending(
                    interest,
                    crate::domain_computation::primary_graph::application_output_demand::WorthQueryPendingOutputDelivery {
                        receipt,
                        change,
                    },
                );
                return Err(denial(
                    WorthQueryOutputDemandDenialKind::SchedulingDeferred,
                    detail,
                ));
            }
            Err(delivery_denial) => {
                let failure = denial(
                    WorthQueryOutputDemandDenialKind::SchedulingRejected,
                    format!(
                        "{producer_identity}: readiness delivery denied ({:?}): {}",
                        delivery_denial.kind(),
                        delivery_denial.detail(),
                    ),
                );
                self.output_demands.finish_delivery_pending(
                    interest,
                    crate::domain_computation::primary_graph::application_output_demand::WorthQueryPendingOutputDelivery {
                        receipt,
                        change: delivery_denial.into_change(),
                    },
                );
                return Err(failure);
            }
        };
        if !delivery.has_conditional_successor() {
            let preserved = self
                .installed_producers
                .entries
                .get(producer_identity)
                .expect("readiness route must retain its installed producer")
                .executor
                .preserved_readiness_output(&receipt);
            let counters = delivery.counters();
            let delivery_is_exact_noop = delivery.truth_targets_admitted() == 0
                && delivery.change_set().changes().is_empty()
                && delivery.signal_seeds_emitted() == 0
                && delivery.node_fan_out() == 0
                && delivery.slots_touched() == 0
                && counters.failed_deliveries() == 0;
            if preserved && delivery_is_exact_noop {
                return self.finish_output_readiness_evaluation::<Family>(
                    interest,
                    producer_identity,
                    crate::domain_computation::primary_graph::application_output_demand::WorthQueryPendingOutputReadiness {
                        receipt,
                        delivery,
                    },
                );
            }
            let result = Err(denial(
                WorthQueryOutputDemandDenialKind::SchedulingRejected,
                format!(
                    "{}: readiness delivery lacked a Signal successor (targets={}, seeds={}, fan_out={}, slots={}, aspect_rejections={}, binding_rejections={}, change_rejections={}, projection_rejections={})",
                    Family::IDENTITY,
                    delivery.truth_targets_admitted(),
                    delivery.signal_seeds_emitted(),
                    delivery.node_fan_out(),
                    delivery.slots_touched(),
                    counters.aspect_rejections(),
                    counters.binding_rejections(),
                    counters.change_kind_rejections(),
                    counters.projection_rejections(),
                ),
            ));
            self.output_demands.finish(interest, &result);
            return result.map(WorthQueryOutputDemandAdvance::Settled);
        }
        self.finish_output_readiness_evaluation::<Family>(
            interest,
            producer_identity,
            crate::domain_computation::primary_graph::application_output_demand::WorthQueryPendingOutputReadiness {
                receipt,
                delivery,
            },
        )
    }

    pub(super) fn finish_output_readiness_evaluation<Family>(
        &self,
        interest: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandInterest,
        producer_identity: &str,
        pending: crate::domain_computation::primary_graph::application_output_demand::WorthQueryPendingOutputReadiness,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let readiness = match self.evaluate_current_output_readiness(
            producer_identity,
            &pending.receipt,
            Some(&pending.delivery),
        ) {
            Ok(readiness) => readiness,
            Err(denial) => {
                self.output_demands
                    .finish_readiness_pending(interest, pending);
                return Err(denial);
            }
        };
        let result = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandSettlement::from_commit(
            self,
            pending.receipt,
            readiness,
        );
        self.output_demands.finish(interest, &result);
        result.map(WorthQueryOutputDemandAdvance::Settled)
    }

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
        let selected = self
            .on_branch(receipt.product_branch())
            .select()
            .map_err(|error| scheduling_failed(producer_identity, error))?;
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
        .map_err(|error| scheduling_failed(producer_identity, error))?;
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
