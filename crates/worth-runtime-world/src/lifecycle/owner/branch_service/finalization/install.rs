use crate::branch::registry::{
    ProductBranchSourceInstallDenial, ProductBranchSourceInstallFailure,
};
use crate::lifecycle::RuntimeWorldBranchCreationOutcome;
use crate::publication::RetainedCommitDisposition;
use crate::recovery::ProductUnpublishedCause;

use super::state::{ForkedBranchFinalization, ObservedForkedBranch};

impl ObservedForkedBranch {
    /// The registry borrows the destination cell until actual insertion under
    /// the source guard, and stamps the exact witness before releasing either
    /// guard. Caller unwind therefore distinguishes installation from refusal.
    pub(super) fn install(
        self,
        cancellation: &crate::publication::RuntimeWorldCancellationToken,
    ) -> RuntimeWorldBranchCreationOutcome {
        let ForkedBranchFinalization {
            branch,
            reservation,
            mut custody,
            ..
        } = self.state;
        #[cfg(test)]
        super::super::super::install_control::pause_before_source_guarded_install(
            branch.owner_identity(),
        );
        #[cfg(not(test))]
        let _ = branch;
        match custody.install_creation_cell(reservation, cancellation) {
            Ok(observation) => RuntimeWorldBranchCreationOutcome::Performed(observation),
            Err(ProductBranchSourceInstallFailure {
                reservation,
                denial,
            }) => {
                drop(reservation);
                let (cause, observed) = match denial {
                    ProductBranchSourceInstallDenial::Cancelled => {
                        (ProductUnpublishedCause::CancellationAfterEffect, None)
                    }
                    ProductBranchSourceInstallDenial::Registry(_) => {
                        (ProductUnpublishedCause::DestinationAdmissionDenied, None)
                    }
                    ProductBranchSourceInstallDenial::SourceRetired => {
                        (ProductUnpublishedCause::StaleProductHead, None)
                    }
                    ProductBranchSourceInstallDenial::SourceDisplaced(observed) => {
                        (ProductUnpublishedCause::StaleProductHead, Some(observed))
                    }
                };
                RuntimeWorldBranchCreationOutcome::ProductUnpublished(custody.retain(
                    cause,
                    observed,
                    RetainedCommitDisposition::InstallSuccessor,
                ))
            }
        }
    }
}
