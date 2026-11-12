//! Advance only the missing original World occurrence, never Native execution.
use super::{phase_loan::OriginalContinuationPhaseLoan, *};
use crate::{
    durability::WorldDurablePublicationStop, publication::WorldHomeTransitionEffectDenial as Denial,
};
use worth_execution::ExecutionRequest;

impl OriginalHomeTransitionContinuation {
    pub(crate) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<OriginalContinuationAdvance, Denial> {
        let mut loan = OriginalContinuationPhaseLoan::take(&mut self.progress);
        loop {
            match loan.phase() {
                OriginalContinuationProgress::Prepared(work) => {
                    request.consult().map_err(|cause| {
                        Denial::Medium(crate::durability::WorldDurableMediumDenial::Request(cause))
                    })?;
                    let subject = work
                        .common
                        .occurrence
                        .subject
                        .as_ref()
                        .expect("original occurrence finalized before submission")
                        .clone();
                    let mut permit = work
                        .common
                        .cell
                        .begin_durable_movement(&work.common.before, subject)
                        .map_err(|_| Denial::WorldHeadUnavailable)?;
                    permit.bind_home_transition_control(work.common.control.clone());
                    loan.submit(permit);
                    return Ok(OriginalContinuationAdvance::Pending);
                }
                OriginalContinuationProgress::Submitted(work) => {
                    let common = &work.work.common;
                    match common.binding.publish(
                        common
                            .occurrence
                            .subject
                            .as_ref()
                            .expect("same original occurrence"),
                        request,
                    ) {
                        Ok(committed) => {
                            // The actual cell retains the witness before any other work.
                            work.permit.acknowledge(committed.into_witness());
                            loan.acknowledge();
                            let OriginalContinuationProgress::Acknowledged(work) = loan.phase()
                            else {
                                unreachable!("the actual acknowledgement is retained")
                            };
                            if let Some(origin) =
                                work.work.common.occurrence.take_unretained_origin()
                            {
                                work.work
                                    .common
                                    .binding
                                    .retain_checkpoint_occurrence(origin);
                            }
                            return Ok(OriginalContinuationAdvance::Pending);
                        }
                        Err(
                            WorldDurablePublicationStop::Pending
                            | WorldDurablePublicationStop::Busy,
                        ) => return Ok(OriginalContinuationAdvance::Pending),
                        Err(WorldDurablePublicationStop::EvidenceRequired) => {
                            return Ok(OriginalContinuationAdvance::EvidenceRequired)
                        }
                        Err(WorldDurablePublicationStop::Unresolved) => {
                            return Ok(OriginalContinuationAdvance::Indeterminate)
                        }
                        Err(WorldDurablePublicationStop::Refused(cause)) => {
                            loan.unsubmit();
                            return Err(Denial::Medium(cause.into_cause()));
                        }
                    }
                }
                OriginalContinuationProgress::Acknowledged(work) => {
                    let common = &mut work.work.common;
                    if let Some(origin) = common.occurrence.take_unretained_origin() {
                        common.binding.retain_checkpoint_occurrence(origin);
                    }
                    let occurrence = work.admit_occurrence(request)?;
                    loan.materialize(occurrence);
                }
                OriginalContinuationProgress::Materialized(work) => {
                    let common = &mut work.common;
                    let (head, publication) = common
                        .capacity
                        .install_readmitted_product_head(
                            &common.history,
                            &common.after.shared_commit(),
                            &mut common.history_pins,
                            &work.record,
                        )
                        .map_err(Denial::History)?;
                    loan.install_history(head, publication);
                }
                OriginalContinuationProgress::HistoryInstalled(work) => {
                    let transfer = work
                        .common
                        .publication_pins
                        .try_transfer_product_head(work.common.after.basis())
                        .map_err(Denial::transfer)?;
                    loan.transfer(transfer);
                }
                OriginalContinuationProgress::Transferred(_) => loan.install_head(),
                OriginalContinuationProgress::Installed(_) => {
                    // A consumed permit cannot be reconstructed after a loss or
                    // the actual reference owner's post-commit unwind hook.
                    loan.prearm_reopen();
                    let OriginalContinuationProgress::ReopenRequired(work) = loan.phase() else {
                        unreachable!("reopen is armed before consuming movement custody")
                    };
                    let Some(permit) = work.permit.take() else {
                        return Ok(OriginalContinuationAdvance::ReopenRequired);
                    };
                    if permit
                        .publish_readmitted(
                            &work.common.before,
                            &mut work.head,
                            work.record.record(),
                        )
                        .is_err()
                    {
                        return Ok(OriginalContinuationAdvance::ReopenRequired);
                    }
                    loan.complete();
                }
                OriginalContinuationProgress::ReopenRequired(_) => {
                    return Ok(OriginalContinuationAdvance::ReopenRequired)
                }
                OriginalContinuationProgress::Completed(_) => return Ok(loan.deliver()),
                OriginalContinuationProgress::Delivered => {
                    unreachable!("delivery removes the original catalog row")
                }
            }
        }
    }
}
