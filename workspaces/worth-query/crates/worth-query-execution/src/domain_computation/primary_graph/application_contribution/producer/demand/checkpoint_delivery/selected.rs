//! Reenter the real Published/Delivered checkpoint under the required meter.

use super::*;
use crate::domain_computation::primary_graph::{
    application_contribution::producer::WorthQueryProducerDemandResources,
    application_output_demand::{PreparedSelectedCheckpointFinish, SelectedCheckpointFinishStop},
    output_lineage::invalidation::InvalidationEditAdmission,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// The caller owns the exact successor Interest and any checkpoint that a
    /// racing registry finish returns. Delivery and readiness use the ordinary
    /// stage, while Query preparation and finish stay on the carried meter.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn advance_selected_output_checkpoint(
        &self,
        producer_identity: &str,
        claim: WorthQueryOutputClaimIdentity,
        checkpoint: Checkpoint,
        request_scope: &WorthQueryRequestScope,
        resources: Option<WorthQueryProducerDemandResources>,
        returned: &mut Option<Checkpoint>,
        finish: PreparedSelectedCheckpointFinish<'_>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        match checkpoint {
            Checkpoint::Published {
                receipt,
                delivery,
                ready_backing,
            } => {
                // These are Query owner visits. Bridge/Signal delivery keeps
                // its installed serial admission class.
                if let Err(stop) = admission.charge_external_work(2) {
                    finish_selected(
                        finish,
                        claim,
                        Checkpoint::Published {
                            receipt,
                            delivery,
                            ready_backing,
                        },
                        Some(admission_denial(stop)),
                        returned,
                    )?;
                    return Ok(false);
                }
                let work = producer_identity
                    .len()
                    .checked_add(16)
                    .and_then(|work| work.checked_add(std::mem::size_of::<Checkpoint>() * 2));
                let Some(work) = work else {
                    finish_selected(
                        finish,
                        claim,
                        Checkpoint::Published {
                            receipt,
                            delivery,
                            ready_backing,
                        },
                        Some(work_denial()),
                        returned,
                    )?;
                    return Ok(false);
                };
                if let Err(cause) = admission.charge_external_work(work as u64) {
                    let cause = admission_denial(cause);
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
                let prepared =
                    match self.prepare_selected_delivery(producer_identity, &receipt, admission) {
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
                if let Err(cause) = check_request(request_scope, admission) {
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
                    producer_identity,
                    receipt,
                    delivery,
                    ready_backing,
                    true,
                    Some(prepared),
                    |checkpoint, denial| {
                        progressed =
                            matches!(&checkpoint, Checkpoint::Delivered { .. }) && denial.is_none();
                        let denial =
                            denial.or_else(|| check_request(request_scope, admission).err());
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
                    producer_identity,
                    &receipt,
                    delivery.as_ref(),
                    request_scope,
                    admission,
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
                let work = std::mem::size_of::<WorthQueryCompletedOutputDemand>()
                    .checked_add(std::mem::size_of::<Checkpoint>() * 2)
                    .and_then(|work| work.checked_add(4));
                let Some(work) = work else {
                    finish_selected(
                        finish,
                        claim,
                        Checkpoint::Delivered {
                            receipt,
                            delivery,
                            ready_backing,
                        },
                        Some(work_denial()),
                        returned,
                    )?;
                    return Ok(false);
                };
                if let Err(stop) = admission.charge_external_work(work as u64) {
                    finish_selected(
                        finish,
                        claim,
                        Checkpoint::Delivered {
                            receipt,
                            delivery,
                            ready_backing,
                        },
                        Some(admission_denial(stop)),
                        returned,
                    )?;
                    return Ok(false);
                }
                let completion = ready_backing.complete(WorthQueryCompletedOutputDemand {
                    authority: WorthQueryAcceptedOutputAuthority::Committed(receipt),
                    producer_commit_authority: None,
                    readiness,
                    resources,
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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PreparedSelectedDelivery<'a>, WorthQueryOutputDemandDenial> {
        admission
            .charge_external_work(4)
            .map_err(admission_denial)?;
        let width = producer_identity
            .len()
            .checked_add(1)
            .ok_or_else(work_denial)?;
        for count in [
            self.output_readiness_routes.len(),
            self.installed_producers.entries.len(),
        ] {
            let levels = usize::BITS as usize - count.max(1).leading_zeros() as usize;
            let work = count
                .min(11)
                .checked_mul(levels)
                .and_then(|comparisons| comparisons.checked_mul(width))
                .and_then(|work| work.checked_add(4))
                .ok_or_else(work_denial)?;
            admission
                .charge_external_work(work as u64)
                .map_err(admission_denial)?;
        }
        let route = self
            .output_readiness_routes
            .get(producer_identity)
            .expect("installation retains one readiness route per producer");
        let installed = self
            .installed_producers
            .entries
            .get(producer_identity)
            .expect("readiness route retains its installed producer");
        let preserved = installed
            .executor
            .preserved_readiness_output_admitted(receipt, admission)?;
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

fn check_request(
    request: &WorthQueryRequestScope,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), WorthQueryOutputDemandDenial> {
    admission
        .charge_external_work(1)
        .map_err(admission_denial)?;
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

fn work_denial() -> WorthQueryOutputDemandDenial {
    denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, "")
}

fn admission_denial(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> WorthQueryOutputDemandDenial {
    use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
    let kind = match stop {
        Stop::WorkExhausted { .. } | Stop::WorkCounterOverflow => {
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        }
        _ => WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
    };
    denial(kind, "")
}
