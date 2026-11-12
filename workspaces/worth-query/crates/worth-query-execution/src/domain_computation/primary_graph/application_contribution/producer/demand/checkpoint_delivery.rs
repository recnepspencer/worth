use super::own_effect_observation::{self, Stage};
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
mod publication;
mod recovered;
mod selected;
mod selected_binding;
use selected_binding::SelectedDeliveryBinding;
mod unpublished;

pub struct PreparedSelectedDelivery<'a> {
    lowering: SelectedDeliveryBinding<'a>,
    ordinal: usize,
    preserved: bool,
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(super) fn advance_output_checkpoint(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        interest: &WorthQueryOutputDemandInterest,
        producer_identity: &str,
        claim: WorthQueryOutputClaimIdentity,
        checkpoint: Checkpoint,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        invocation: &super::super::ProducerInvocationAuthority<'_>,
        returned: &mut Option<Checkpoint>,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        match checkpoint {
            Checkpoint::PublicationPending {
                owner,
                ready_backing,
            } => self.advance_unpublished_checkpoint(
                phase,
                producer_identity,
                principal,
                request,
                invocation,
                owner,
                ready_backing,
                |checkpoint| {
                    self.finish_output_checkpoint(interest, claim, checkpoint, None, returned)
                        .map(|_| ())
                },
            ),
            Checkpoint::Published {
                receipt,
                delivery,
                ready_backing,
            } => {
                // A delivery that is still waiting leaves the checkpoint
                // Published: nothing moved, so the caller does not spin on it.
                own_effect_observation::record(Stage::DeliveryStarted);
                let mut delivered = false;
                self.deliver_output_checkpoint_with_finish(
                    phase,
                    producer_identity,
                    receipt.into(),
                    delivery,
                    ready_backing,
                    false,
                    None,
                    |checkpoint, denial| {
                        delivered = matches!(&checkpoint, Checkpoint::Delivered { .. });
                        self.finish_output_checkpoint(interest, claim, checkpoint, denial, returned)
                    },
                )?;
                own_effect_observation::record(if delivered {
                    Stage::Delivered
                } else {
                    Stage::DeliveryWait
                });
                Ok(delivered)
            }
            Checkpoint::Delivered {
                receipt,
                delivery,
                ready_backing,
            } => {
                let readiness = match self.evaluate_current_output_readiness(
                    execution,
                    phase,
                    producer_identity,
                    (&receipt).into(),
                    delivery.as_ref(),
                    &ready_backing,
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
                                returned,
                            )
                            .map(|_| false)
                    }
                };
                own_effect_observation::record(Stage::Ready);
                let completion = ready_backing.complete(WorthQueryCompletedOutputDemand {
                    authority: WorthQueryAcceptedOutputAuthority::Committed(receipt),
                    readiness,
                });
                self.finish_output_checkpoint(
                    interest,
                    claim,
                    Checkpoint::Ready(completion),
                    None,
                    returned,
                )?;
                Ok(true)
            }
            checkpoint @ (Checkpoint::RecoveredPublished { .. }
            | Checkpoint::RecoveredDelivered { .. }) => self.advance_recovered_checkpoint(
                execution,
                phase,
                producer_identity,
                checkpoint,
                None,
                |checkpoint, denial| {
                    self.finish_output_checkpoint(interest, claim, checkpoint, denial, returned)
                },
            ),
            Checkpoint::Ready(_) => unreachable!("ready output opens reads without a claim"),
        }
    }

    fn deliver_output_checkpoint_with_finish(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

        producer_identity: &str,
        receipt: publication::Publication,
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
            .runtime
            .primary_provider
            .take_delayed_output_readiness_delivery()
        {
            return finish(receipt.published(delivery, ready_backing), None);
        }
        let Delivery::Change(change) = delivery else {
            return finish(receipt.delivered(None, ready_backing), None);
        };
        let (lowering, ordinal) = if let Some(prepared) = prepared.as_ref() {
            (prepared.lowering, prepared.ordinal)
        } else {
            let route = self
                .runtime
                .output_readiness_routes
                .get(producer_identity)
                .expect("installation requires one readiness route for every output producer");
            (
                SelectedDeliveryBinding::Installed(&route.lowering),
                route.delivery_dependency_ordinal,
            )
        };
        let root = self
            .granular_invalidation_installation()
            .retain_product_shared_root();
        let owner = self.runtime.bridge.conditional();
        let binding = match lowering.admit_signal_basis(&owner, change.signal_basis()) {
            Ok(binding) => binding,
            Err(cause) => {
                return finish(
                    receipt.published(Delivery::Change(change), ready_backing),
                    Some(denial(
                        WorthQueryOutputDemandDenialKind::BridgeConditional(Box::new(cause.kind())),
                        cause.detail().to_owned(),
                    )),
                )
            }
        };
        drop(owner);
        let outcome = root.deliver_performed_relational_change(phase, &binding, ordinal, change);
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
                    receipt.published(Delivery::Change(change), ready_backing),
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
                    receipt.published(Delivery::Change(cause.into_change()), ready_backing),
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
                    .preserved_readiness_output(receipt.view())
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
                    receipt.delivered(Some(delivered), ready_backing),
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
        finish(receipt.delivered(Some(delivered), ready_backing), None)
    }

    fn finish_output_checkpoint(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        claim: WorthQueryOutputClaimIdentity,
        checkpoint: Checkpoint,
        denial: Option<WorthQueryOutputDemandDenial>,
        returned: &mut Option<Checkpoint>,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial> {
        use crate::domain_computation::primary_graph::application_output_demand::CheckpointFinishStop;
        match self.runtime.output_demands.finish_checkpoint_owned(
            interest,
            claim,
            checkpoint,
            denial.as_ref(),
        ) {
            Ok(()) => {}
            Err(CheckpointFinishStop::Restored(stop)) => return Err(stop),
            Err(CheckpointFinishStop::Returned(stop, checkpoint)) => {
                assert!(returned.is_none(), "one original checkpoint owner");
                *returned = Some(checkpoint);
                return Err(stop);
            }
        }
        match denial {
            Some(denial) => Err(denial),
            None => Ok(WorthQueryOutputDemandAdvance::Pending),
        }
    }
}

mod court_doors_for_test;
