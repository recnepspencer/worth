//! Actual partial-creation cleanup is shared by retry and explicit discard.
use super::{ForkRecovery, WorthQueryHistoryEditDenial as Denial};
use crate::application_entry::{WorthQueryApplicationHistoryDenial, WorthQueryApplicationRequest};
use worth_query_execution::facade::product::WorthQueryProductBranchCreationRecoveryReleaseFailure;
use worth_query_installation::facade::ApplicationSchema;

impl ForkRecovery {
    pub(in crate::application_entry::history) fn release_in_request<S: ApplicationSchema>(
        pending: &mut Option<Self>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), Denial> {
        if pending.is_none() {
            return Ok(());
        }
        request
            .application
            .with_application_advancement(request.scope, |phase| {
                let original = pending
                    .take()
                    .expect("the admitted call owns the pending recovery");
                match original.release_in_phase(&phase) {
                    Ok(()) => Ok(()),
                    Err(original) => {
                        *pending = Some(original);
                        Err(Denial::ForkRecovery)
                    }
                }
            })
            .map_err(|cause| {
                Denial::History(WorthQueryApplicationHistoryDenial::ExecutionRequest(cause))
            })?
    }

    fn release_in_phase(
        self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
    ) -> Result<(), Self> {
        match self {
            Self::Creation(owner) => match owner.release_obligations_in_advancement(phase, 0) {
                Ok(_) => Ok(()),
                Err(WorthQueryProductBranchCreationRecoveryReleaseFailure::Recovery(failure)) => {
                    Err(Self::Creation(failure.into_recovery()))
                }
                Err(WorthQueryProductBranchCreationRecoveryReleaseFailure::OwnerCleanup(
                    failure,
                )) => Err(Self::Cleanup(failure)),
            },
            Self::Cleanup(failure) => failure
                .into_cleanup()
                .retry_in_advancement(phase)
                .map(|_| ())
                .map_err(Self::Cleanup),
        }
    }
}
