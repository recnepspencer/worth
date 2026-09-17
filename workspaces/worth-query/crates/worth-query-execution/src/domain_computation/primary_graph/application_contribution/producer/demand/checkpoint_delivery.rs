use worth_query_installation::facade::ApplicationSchema;

use super::progression::denial;
use super::{
    WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};
use crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChangeDeliveryDenialKind as DeliveryDenialKind;
use crate::domain_computation::primary_graph::application_output_demand::{
    WorthQueryCompletedOutputDemand, WorthQueryOutputCheckpoint as Checkpoint,
    WorthQueryOutputClaimIdentity, WorthQueryOutputDemandInterest,
    WorthQueryPendingOutputDelivery as Delivery,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(super) fn advance_output_checkpoint(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        producer_identity: &str,
        claim: WorthQueryOutputClaimIdentity,
        checkpoint: Checkpoint,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial> {
        match checkpoint {
            Checkpoint::Published { receipt, delivery } => self.deliver_output_checkpoint(
                interest,
                producer_identity,
                claim,
                receipt,
                delivery,
            ),
            Checkpoint::Delivered { receipt, delivery } => {
                let readiness = match self.evaluate_current_output_readiness(
                    producer_identity,
                    &receipt,
                    delivery.as_ref(),
                ) {
                    Ok(readiness) => readiness,
                    Err(cause) => {
                        return self.finish_output_checkpoint(
                            interest,
                            claim,
                            Checkpoint::Delivered { receipt, delivery },
                            Some(cause),
                        )
                    }
                };
                self.finish_output_checkpoint(
                    interest,
                    claim,
                    Checkpoint::Ready(WorthQueryCompletedOutputDemand { receipt, readiness }),
                    None,
                )
            }
            Checkpoint::Ready(_) => unreachable!("ready output opens reads without a claim"),
        }
    }

    fn deliver_output_checkpoint(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        producer_identity: &str,
        claim: WorthQueryOutputClaimIdentity,
        receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        delivery: Delivery,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial> {
        if self
            .primary_provider
            .take_delayed_output_readiness_delivery()
        {
            return self.finish_output_checkpoint(
                interest,
                claim,
                Checkpoint::Published { receipt, delivery },
                None,
            );
        }
        let Delivery::Change(change) = delivery else {
            return self.finish_output_checkpoint(
                interest,
                claim,
                Checkpoint::Delivered {
                    receipt,
                    delivery: None,
                },
                None,
            );
        };
        let route = self
            .output_readiness_routes
            .get(producer_identity)
            .expect("installation requires one readiness route for every output producer");
        let root = self
            .granular_invalidation_installation()
            .retain_product_shared_root();
        let outcome = root.deliver_performed_relational_change(
            &route.lowering,
            route.delivery_dependency_ordinal,
            change,
        );
        let delivered = match outcome {
            Ok(crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChangeDeliveryOutcome::Success(delivered)) => delivered,
            Ok(outcome) => {
                let detail = format!("{producer_identity}: readiness delivery returned {outcome:?}");
                let kind = if outcome.is_retryable() {
                    WorthQueryOutputDemandDenialKind::SchedulingDeferred
                } else {
                    WorthQueryOutputDemandDenialKind::SchedulingRejected
                };
                let change = outcome.into_undelivered_change()
                    .expect("non-success delivery returns its performed change");
                return self.finish_output_checkpoint(
                    interest,
                    claim,
                    Checkpoint::Published { receipt, delivery: Delivery::Change(change) },
                    Some(denial(kind, detail)),
                );
            }
            Err(cause) => {
                let kind = match cause.kind() {
                    DeliveryDenialKind::ForeignProductRoot
                    | DeliveryDenialKind::ForeignProductOccurrence
                    | DeliveryDenialKind::ForeignConditionalOperation
                    | DeliveryDenialKind::ProductAdmission
                    | DeliveryDenialKind::ConditionalProductAdmission
                    | DeliveryDenialKind::Bridge => WorthQueryOutputDemandDenialKind::SchedulingRejected,
                };
                let failure = denial(
                    kind,
                    format!(
                        "{producer_identity}: readiness delivery denied ({:?}): {}",
                        cause.kind(), cause.detail(),
                    ),
                );
                return self.finish_output_checkpoint(
                    interest,
                    claim,
                    Checkpoint::Published {
                        receipt,
                        delivery: Delivery::Change(cause.into_change()),
                    },
                    Some(failure),
                );
            }
        };
        if !delivered.has_conditional_successor() {
            let preserved = self
                .installed_producers
                .entries
                .get(producer_identity)
                .expect("readiness route retains its installed producer")
                .executor
                .preserved_readiness_output(&receipt);
            let counters = delivered.counters();
            let exact_noop = delivered.truth_targets_admitted() == 0
                && delivered.change_set().changes().is_empty()
                && delivered.signal_seeds_emitted() == 0
                && delivered.node_fan_out() == 0
                && delivered.slots_touched() == 0
                && counters.failed_deliveries() == 0;
            if !preserved || !exact_noop {
                return self.finish_output_checkpoint(
                    interest,
                    claim,
                    Checkpoint::Delivered {
                        receipt,
                        delivery: Some(delivered),
                    },
                    Some(denial(
                        WorthQueryOutputDemandDenialKind::SchedulingRejected,
                        format!(
                            "{producer_identity}: readiness delivery lacked a Signal successor"
                        ),
                    )),
                );
            }
        }
        self.finish_output_checkpoint(
            interest,
            claim,
            Checkpoint::Delivered {
                receipt,
                delivery: Some(delivered),
            },
            None,
        )
    }

    fn finish_output_checkpoint(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        claim: WorthQueryOutputClaimIdentity,
        checkpoint: Checkpoint,
        denial: Option<WorthQueryOutputDemandDenial>,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial> {
        self.output_demands
            .finish_checkpoint(interest, claim, checkpoint, denial.as_ref())?;
        match denial {
            Some(denial) => Err(denial),
            None => Ok(WorthQueryOutputDemandAdvance::Pending),
        }
    }
}
