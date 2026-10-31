//! Retained-source admission preserves exact component truth without current-head claims.
use super::*;
use crate::branch::RetainedProductBranchCreationSource;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn admit_retained_creation_source(
        &self,
        source: &ProductBranchObservation,
    ) -> Result<RetainedProductBranchCreationSource, RuntimeWorldBranchAdmissionDenial> {
        self.admit_retained_source_lifecycle(source)?;
        let native = self
            .state
            .relational
            .fork_port()
            .admit_retained_fork_source(source.basis().relational_basis())
            .map_err(map_retained_native_denial)?;
        self.state
            .signal
            .basis_port()
            .compare_current_exact(source.basis().signal_basis())
            .map_err(|_| RuntimeWorldBranchAdmissionDenial::ForkSourceChanged)?;
        Ok(RetainedProductBranchCreationSource {
            observation: source.clone(),
            native,
        })
    }

    pub(in crate::lifecycle::owner) fn admit_retained_source_lifecycle(
        &self,
        source: &ProductBranchObservation,
    ) -> Result<(), RuntimeWorldBranchAdmissionDenial> {
        if source.owner_identity() != self.owner_identity() {
            return Err(RuntimeWorldBranchAdmissionDenial::ForeignOwner);
        }
        if !self.branch_service_is_available() {
            return Err(RuntimeWorldBranchAdmissionDenial::OwnerUnavailable);
        }
        let cell = self
            .state
            .branches
            .branch_cell(source.branch_identity())
            .ok_or(RuntimeWorldBranchAdmissionDenial::RetiredBranch)?;
        if cell.atomic_snapshot().lifecycle_incarnation() != source.lifecycle_incarnation() {
            return Err(RuntimeWorldBranchAdmissionDenial::RetiredBranch);
        }
        Ok(())
    }
}

fn map_retained_native_denial(
    cause: worth_relational::facade::branch::RelationalForkDenial,
) -> RuntimeWorldBranchAdmissionDenial {
    use worth_relational::facade::branch::RelationalForkDenial as Native;
    match cause {
        Native::RetainedSourceDurabilityUnsupported => {
            RuntimeWorldBranchAdmissionDenial::RetainedSourceDurabilityUnsupported
        }
        Native::ForeignRuntime => RuntimeWorldBranchAdmissionDenial::ForeignOwner,
        Native::SourceBranchMissing | Native::SourceArchived | Native::SourceDeleting => {
            RuntimeWorldBranchAdmissionDenial::RetiredBranch
        }
        Native::EmptySource | Native::MissingArtifact => {
            RuntimeWorldBranchAdmissionDenial::HistoryEntryUnavailable
        }
        _ => RuntimeWorldBranchAdmissionDenial::OwnerUnavailable,
    }
}
