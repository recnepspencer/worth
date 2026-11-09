//! Durable sibling creation retains the original before either owner runs.
mod driving;
mod preparation;
mod release;
mod selection;
mod state;
use super::*;
use crate::lifecycle::{RuntimeWorldHomeBranchCreationRequest, RuntimeWorldPreparationService};
use crate::publication::{
    ActiveAttemptCustody, RetainedCommitDisposition, RuntimeWorldCancellationToken,
};
use crate::recovery::ProductUnpublishedCause;
use state::CreationPhase;
pub(crate) use state::WorldDurableCreationCustody;
impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn create_durable_product_branch(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: RuntimeWorldHomeBranchCreationRequest,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> Result<RuntimeWorldBranchCreationOutcome, RuntimeWorldBranchAdmissionDenial> {
        let custody = self.prepare_durable_creation(execution, request, cancellation)?;
        self.drive_durable_creation(execution, custody, cancellation)
    }
    pub(super) fn drive_durable_creation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mut custody: ActiveAttemptCustody,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> Result<RuntimeWorldBranchCreationOutcome, RuntimeWorldBranchAdmissionDenial> {
        let mut body = custody.loan_durable_creation();
        let answer =
            self.advance_durable_creation(execution, &mut custody, &mut body, cancellation);
        let phase = body.phase;
        let released = body.released.take();
        let no_effect = body.no_effect.take();
        custody.begin_publication();
        drop(body); // Restore original driver before materializing or abandoning the same seat.
        match (phase, answer) {
            (CreationPhase::Performed, Ok(())) => Ok(RuntimeWorldBranchCreationOutcome::Performed(
                custody.take_creation_response(),
            )),
            (CreationPhase::NoEffect, Err(_)) => Ok(RuntimeWorldBranchCreationOutcome::NoEffect(
                no_effect.expect("actual original no-effect proof"),
            )),
            (CreationPhase::Released, Ok(())) => {
                Ok(RuntimeWorldBranchCreationOutcome::ReleasedOriginal(
                    released.expect("actual release receipt"),
                ))
            }
            (
                CreationPhase::Intent
                | CreationPhase::Owners
                | CreationPhase::Releasing
                | CreationPhase::Prepared
                | CreationPhase::Selected
                | CreationPhase::ReopenRequired,
                Ok(()) | Err(_),
            ) => Ok(RuntimeWorldBranchCreationOutcome::ProductUnpublished(
                custody.retain(
                    ProductUnpublishedCause::OpenAttemptRetained,
                    None,
                    RetainedCommitDisposition::KeepOriginal,
                ),
            )),
            (CreationPhase::Performed | CreationPhase::Released, Err(_))
            | (CreationPhase::NoEffect, Ok(())) => {
                unreachable!("terminal agrees with original proof")
            }
        }
    }
}
