//! Actual partial-creation cleanup is shared by retry and explicit discard.
use super::ForkRecovery;
use worth_query_execution::facade::product::WorthQueryProductBranchCreationRecoveryReleaseFailure;

impl ForkRecovery {
    pub(in crate::application_entry::history) fn release(self) -> Result<(), Self> {
        match self {
            Self::Creation(owner) => match owner.release_obligations(0) {
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
                .retry()
                .map(|_| ())
                .map_err(Self::Cleanup),
        }
    }
}
