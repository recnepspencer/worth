use crate::basis::{self, AdmittedCompositeRuntimeWorldBasis};
use crate::publication::CompositeAttemptProgress;

use super::RuntimeWorldOwnerRoot;

use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_signal::facade::branch::AdmittedSignalBranchBasis;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn issue_successor_basis(
        &self,
        relational: AdmittedRelationalBranchBasis,
        signal: AdmittedSignalBranchBasis,
        correspondence: worth_runtime_bridge::facade::AdmittedRuntimeWorldCorrespondenceBasis,
    ) -> AdmittedCompositeRuntimeWorldBasis {
        let identities = self
            .state
            .identities
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        basis::admit_validated(&identities, relational, signal, correspondence)
    }

    pub(crate) fn issue_successor_basis_from_progress(
        &self,
        progress: &CompositeAttemptProgress,
        predecessor: &AdmittedCompositeRuntimeWorldBasis,
    ) -> AdmittedCompositeRuntimeWorldBasis {
        self.issue_successor_basis(
            progress
                .relational()
                .successor_basis()
                .cloned()
                .unwrap_or_else(|| predecessor.relational_basis().clone()),
            progress
                .signal()
                .successor_basis()
                .cloned()
                .unwrap_or_else(|| predecessor.signal_basis().clone()),
            predecessor.correspondence_basis().clone(),
        )
    }

    pub(crate) fn successor_correspondence_is_valid(
        &self,
        successor: &AdmittedCompositeRuntimeWorldBasis,
    ) -> bool {
        self.state
            .bridge
            .compare_current_exact(successor.correspondence_basis())
            .is_ok()
    }
}
