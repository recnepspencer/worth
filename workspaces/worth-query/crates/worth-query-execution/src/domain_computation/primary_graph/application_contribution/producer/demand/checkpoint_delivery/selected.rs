//! Reenter the real Published/Delivered checkpoint under the required meter.

use super::*;
use crate::domain_computation::primary_graph::application_output_demand::PreparedSelectedCheckpointFinish;
use crate::domain_computation::primary_graph::application_output_demand::SelectedCheckpointFinishStop;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// The caller owns the exact successor Interest and any checkpoint that a
    /// racing registry finish returns. Delivery and readiness use the ordinary
    /// stage, while Query preparation and finish stay on the carried meter.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn advance_selected_output_checkpoint(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        producer_identity: &str,
        claim: WorthQueryOutputClaimIdentity,
        checkpoint: Checkpoint,
        request_scope: &WorthQueryRequestScope,
        returned: &mut Option<Checkpoint>,
        finish: PreparedSelectedCheckpointFinish<'_>,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        match checkpoint {
            Checkpoint::Published {
                receipt,
                delivery,
                ready_backing,
            } => {
                // These are Query owner visits. Bridge/Signal delivery keeps
                // its installed serial admission class.

                let prepared = match self.prepare_selected_delivery(producer_identity, &receipt) {
                    Ok(prepared) => prepared,
                    Err(cause) => {
                        finish_selected(
                            finish,
                            claim,
                            Checkpoint::Published {
                                receipt,
                                delivery,
                                ready_backing,
                            },
                            Some(cause),
                            returned,
                        )?;
                        return Ok(false);
                    }
                };
                if let Err(cause) = check_request(request_scope) {
                    finish_selected(
                        finish,
                        claim,
                        Checkpoint::Published {
                            receipt,
                            delivery,
                            ready_backing,
                        },
                        Some(cause),
                        returned,
                    )?;
                    return Ok(false);
                }
                let mut progressed = false;
                self.deliver_output_checkpoint_with_finish(
                    phase,
                    producer_identity,
                    receipt,
                    delivery,
                    ready_backing,
                    true,
                    Some(prepared),
                    |checkpoint, denial| {
                        progressed =
                            matches!(&checkpoint, Checkpoint::Delivered { .. }) && denial.is_none();
                        let denial = denial.or_else(|| check_request(request_scope).err());
                        finish_selected(finish, claim, checkpoint, denial, returned)
                    },
                )?;
                Ok(progressed)
            }
            Checkpoint::Delivered {
                receipt,
                delivery,
                ready_backing,
            } => {
                let readiness = self.evaluate_current_output_readiness_admitted(
                    execution,
                    phase,
                    producer_identity,
                    &receipt,
                    delivery.as_ref(),
                    request_scope,
                );
                let readiness = match readiness {
                    Ok(readiness) => readiness,
                    Err(cause) => {
                        finish_selected(
                            finish,
                            claim,
                            Checkpoint::Delivered {
                                receipt,
                                delivery,
                                ready_backing,
                            },
                            Some(cause),
                            returned,
                        )?;
                        return Ok(false);
                    }
                };

                let completion = ready_backing.complete(WorthQueryCompletedOutputDemand {
                    authority: WorthQueryAcceptedOutputAuthority::Committed(receipt),
                    readiness,
                });
                finish_selected(finish, claim, Checkpoint::Ready(completion), None, returned)?;
                Ok(true)
            }
            Checkpoint::Ready(_) => unreachable!("ready output opens without a checkpoint claim"),
        }
    }

    fn prepare_selected_delivery<'a>(
        &'a self,
        producer_identity: &str,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Result<PreparedSelectedDelivery<'a>, WorthQueryOutputDemandDenial> {
        let route = self
            .output_readiness_routes
            .get(producer_identity)
            .expect("installation retains one readiness route per producer");
        let installed = self
            .installed_producers
            .entries
            .get(producer_identity)
            .expect("readiness route retains its installed producer");
        let preserved = installed.executor.preserved_readiness_output(receipt);
        Ok(PreparedSelectedDelivery {
            lowering: &route.lowering,
            ordinal: route.delivery_dependency_ordinal,
            preserved,
        })
    }
}

fn finish_selected(
    finish: PreparedSelectedCheckpointFinish<'_>,
    claim: WorthQueryOutputClaimIdentity,
    checkpoint: Checkpoint,
    denial: Option<WorthQueryOutputDemandDenial>,
    returned: &mut Option<Checkpoint>,
) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial> {
    match finish.finish(claim, checkpoint, denial.as_ref()) {
        Ok(()) => match denial {
            Some(denial) => Err(denial),
            None => Ok(WorthQueryOutputDemandAdvance::Pending),
        },
        Err(SelectedCheckpointFinishStop::Restored(denial)) => Err(denial),
        Err(SelectedCheckpointFinishStop::Returned(denial, checkpoint)) => {
            assert!(returned.is_none(), "one selected checkpoint owner");
            *returned = Some(checkpoint);
            Err(denial)
        }
    }
}

fn check_request(request: &WorthQueryRequestScope) -> Result<(), WorthQueryOutputDemandDenial> {
    match request.interruption() {
        Some(WorthQueryRequestInterruption::Cancelled) => {
            Err(denial(WorthQueryOutputDemandDenialKind::Cancelled, ""))
        }
        Some(WorthQueryRequestInterruption::DeadlineExceeded) => {
            Err(denial(WorthQueryOutputDemandDenialKind::TimedOut, ""))
        }
        None => Ok(()),
    }
}
