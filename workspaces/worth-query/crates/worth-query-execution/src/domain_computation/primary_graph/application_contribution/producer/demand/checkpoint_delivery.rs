use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_installation::facade::ApplicationSchema;

use super::progression::denial;
use super::{
    WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};
use crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChangeDeliveryDenialKind as DeliveryDenialKind;
use crate::domain_computation::primary_graph::application_output_demand::{
    WorthQueryAcceptedOutputAuthority, WorthQueryCompletedOutputDemand,
    WorthQueryOutputCheckpoint as Checkpoint, WorthQueryOutputClaimIdentity,
    WorthQueryOutputDemandInterest, WorthQueryPendingOutputDelivery as Delivery,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
mod selected;

struct PreparedSelectedDelivery<'a> {
    lowering: &'a std::sync::Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering>,
    ordinal: usize,
    preserved: bool,
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(super) fn advance_output_checkpoint(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

        interest: &WorthQueryOutputDemandInterest,
        producer_identity: &str,
        claim: WorthQueryOutputClaimIdentity,
        checkpoint: Checkpoint,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        match checkpoint {
            Checkpoint::Published {
                receipt,
                delivery,
                ready_backing,
            } => {
                // A delivery that is still waiting leaves the checkpoint
                // Published: nothing moved, so the caller does not spin on it.
                let mut delivered = false;
                self.deliver_output_checkpoint_with_finish(
                    phase,
                    producer_identity,
                    receipt,
                    delivery,
                    ready_backing,
                    false,
                    None,
                    |checkpoint, denial| {
                        delivered = matches!(&checkpoint, Checkpoint::Delivered { .. });
                        self.finish_output_checkpoint(interest, claim, checkpoint, denial)
                    },
                )?;
                Ok(delivered)
            }
            Checkpoint::Delivered {
                receipt,
                delivery,
                ready_backing,
            } => {
                let readiness = match self.evaluate_current_output_readiness(
                    phase,
                    producer_identity,
                    &receipt,
                    delivery.as_ref(),
                ) {
                    Ok(readiness) => readiness,
                    Err(cause) => {
                        return self
                            .finish_output_checkpoint(
                                interest,
                                claim,
                                Checkpoint::Delivered {
                                    receipt,
                                    delivery,
                                    ready_backing,
                                },
                                Some(cause),
                            )
                            .map(|_| false)
                    }
                };
                let resources = self
                    .primary_provider
                    .graph
                    .output_lineage
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .producer_resources_for_receipt(&receipt);
                let completion = ready_backing.complete(WorthQueryCompletedOutputDemand {
                    authority: WorthQueryAcceptedOutputAuthority::Committed(receipt),
                    readiness,
                    resources,
                });
                self.finish_output_checkpoint(
                    interest,
                    claim,
                    Checkpoint::Ready(completion),
                    None,
                )?;
                Ok(true)
            }
            Checkpoint::Ready(_) => unreachable!("ready output opens reads without a claim"),
        }
    }

    fn deliver_output_checkpoint_with_finish(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

        producer_identity: &str,
        receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        delivery: Delivery,
        ready_backing: crate::domain_computation::primary_graph::application_output_demand::PreparedReadyBacking,
        selected: bool,
        prepared: Option<PreparedSelectedDelivery<'_>>,
        finish: impl FnOnce(
            Checkpoint,
            Option<WorthQueryOutputDemandDenial>,
        )
            -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial> {
        if self
            .primary_provider
            .take_delayed_output_readiness_delivery()
        {
            return finish(
                Checkpoint::Published {
                    receipt,
                    delivery,
                    ready_backing,
                },
                None,
            );
        }
        let Delivery::Change(change) = delivery else {
            return finish(
                Checkpoint::Delivered {
                    receipt,
                    delivery: None,
                    ready_backing,
                },
                None,
            );
        };
        let (lowering, ordinal) = if let Some(prepared) = prepared.as_ref() {
            (prepared.lowering, prepared.ordinal)
        } else {
            let route = self
                .output_readiness_routes
                .get(producer_identity)
                .expect("installation requires one readiness route for every output producer");
            (&route.lowering, route.delivery_dependency_ordinal)
        };
        let root = self
            .granular_invalidation_installation()
            .retain_product_shared_root();
        let outcome = root.deliver_performed_relational_change(phase, lowering, ordinal, change);
        let delivered = match outcome {
            Ok(crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChangeDeliveryOutcome::Success(delivered)) => delivered,
            Ok(outcome) => {
                let detail = if selected { String::new() } else { format!("{producer_identity}: readiness delivery returned {outcome:?}") };
                use crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChangeDeliveryOutcome as Outcome;
                use worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryStop as Stop;
                let (posture, change) = match outcome {
                    Outcome::Success(_) => unreachable!("success was consumed above"),
                    Outcome::Denied { posture, change } => (Stop::Denied(posture), change),
                    Outcome::Deferred { posture, change } => (Stop::Deferred(posture), change),
                    Outcome::Stale { posture, change } => (Stop::Stale(posture), change),
                    Outcome::RebindRequired { posture, change } => (Stop::RebindRequired(posture), change),
                    Outcome::Failed { posture, change } => (Stop::Failed(posture), change),
                };
                let kind = WorthQueryOutputDemandDenialKind::CorrespondenceDelivery(Box::new(posture));
                return finish(
                    Checkpoint::Published { receipt, delivery: Delivery::Change(change), ready_backing },
                    Some(denial(kind, detail)),
                );
            }
            Err(cause) => {
                let kind = match cause.kind() {
                    DeliveryDenialKind::ExecutionRequest(cause) => WorthQueryOutputDemandDenialKind::of_execution_stop(cause),
                    kind @ (DeliveryDenialKind::ForeignProductRoot
                    | DeliveryDenialKind::ForeignProductOccurrence
                    | DeliveryDenialKind::ForeignConditionalOperation
                    | DeliveryDenialKind::ProductAdmission
                    | DeliveryDenialKind::ConditionalProductAdmission
                    | DeliveryDenialKind::BridgeRuntimeClosed) => WorthQueryOutputDemandDenialKind::ProductDelivery(Box::new(kind)),
                    DeliveryDenialKind::Bridge(kind) => WorthQueryOutputDemandDenialKind::BridgeConditional(Box::new(kind)),
                };
                let detail = if selected { String::new() } else { format!(
                    "{producer_identity}: readiness delivery denied ({:?}): {}",
                    cause.kind(), cause.detail(),
                ) };
                let failure = denial(kind, detail);
                return finish(
                    Checkpoint::Published {
                        receipt,
                        delivery: Delivery::Change(cause.into_change()),
                        ready_backing,
                    },
                    Some(failure),
                );
            }
        };
        if !delivered.has_conditional_successor() {
            let preserved = if let Some(prepared) = prepared.as_ref() {
                prepared.preserved
            } else {
                self.installed_producers
                    .entries
                    .get(producer_identity)
                    .expect("readiness route retains its installed producer")
                    .executor
                    .preserved_readiness_output(&receipt)
            };
            let counters = delivered.counters();
            let exact_noop = delivered.truth_targets_admitted() == 0
                && delivered.change_set().changes().is_empty()
                && delivered.signal_seeds_emitted() == 0
                && delivered.node_fan_out() == 0
                && delivered.slots_touched() == 0
                && counters.failed_deliveries() == 0;
            if !preserved || !exact_noop {
                return finish(
                    Checkpoint::Delivered {
                        receipt,
                        delivery: Some(delivered),
                        ready_backing,
                    },
                    Some(denial(
                        WorthQueryOutputDemandDenialKind::SchedulingRejected,
                        if selected {
                            String::new()
                        } else {
                            format!(
                                "{producer_identity}: readiness delivery lacked a Signal successor"
                            )
                        },
                    )),
                );
            }
        }
        finish(
            Checkpoint::Delivered {
                receipt,
                delivery: Some(delivered),
                ready_backing,
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
