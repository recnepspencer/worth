//! Process-local fork custody for one actual retained immutable root.
use super::{AdmittedRelationalBranchBasis, RelationalForkDenial, RelationalForkPort};

/// A move-only retained source, distinct from a freshly current fork source.
/// It retains the original admitted basis; descriptors cannot construct it.
/// This profile refuses a durable medium because historical-root reopening is
/// not part of this receiver's contract.
/// A retained source cannot enter the current-source fork receiver:
/// ```compile_fail,E0308
/// use worth_relational::facade::{branch::{AdmittedRelationalRetainedForkSource, RelationalForkPort}, history::BranchId};
/// fn current(port: &RelationalForkPort, request: &worth_execution::ExecutionRequest<'_, '_>, target: BranchId, source: AdmittedRelationalRetainedForkSource) {
///     let _ = port.fork_branch(request, target, source);
/// }
/// ```
/// Transfer it through its actual retained-source receiver:
/// ```
/// use worth_relational::facade::{branch::{AdmittedRelationalRetainedForkSource, RelationalForkPort}, history::BranchId};
/// fn retained(port: &RelationalForkPort, request: &worth_execution::ExecutionRequest<'_, '_>, target: BranchId, source: AdmittedRelationalRetainedForkSource) {
///     let _ = port.fork_retained_branch(request, target, source);
/// }
/// ```
#[derive(Debug)]
pub struct AdmittedRelationalRetainedForkSource {
    pub(super) basis: AdmittedRelationalBranchBasis,
}

impl RelationalForkPort {
    pub fn admit_retained_fork_source(
        &self,
        source: &AdmittedRelationalBranchBasis,
    ) -> Result<AdmittedRelationalRetainedForkSource, RelationalForkDenial> {
        self.validate_retained_fork_profile()?;
        let _operation = self
            .lifecycle
            .admit()
            .ok_or(RelationalForkDenial::OwnerUnavailable)?;
        self.validate_retained_basis(source)?;
        Ok(AdmittedRelationalRetainedForkSource {
            basis: source.clone(),
        })
    }

    pub(super) fn validate_retained_fork_profile(&self) -> Result<(), RelationalForkDenial> {
        if self.medium.is_some() {
            return Err(RelationalForkDenial::RetainedSourceDurabilityUnsupported);
        }
        Ok(())
    }

    pub(super) fn validate_retained_basis(
        &self,
        source: &AdmittedRelationalBranchBasis,
    ) -> Result<(), RelationalForkDenial> {
        if source.identity().runtime_instance_id() != self.runtime_instance_id {
            return Err(RelationalForkDenial::ForeignRuntime);
        }
        let cell = self
            .owner
            .branch_cell(source.identity().branch_id())
            .ok_or(RelationalForkDenial::SourceBranchMissing)?;
        if cell.identity() != source.identity() {
            return Err(RelationalForkDenial::ForeignRuntime);
        }
        match cell.lifecycle_posture() {
            super::RelationalBranchLifecyclePosture::Live => {}
            super::RelationalBranchLifecyclePosture::Archived => {
                return Err(RelationalForkDenial::SourceArchived)
            }
            super::RelationalBranchLifecyclePosture::Deleting => {
                return Err(RelationalForkDenial::SourceDeleting)
            }
        }
        if source.reference().target().is_empty() {
            return Err(RelationalForkDenial::EmptySource);
        }
        super::basis_axis_validation::require_root_matches_reference(
            source.reference(),
            &source.inner.root,
        )
        .map_err(|_| RelationalForkDenial::MissingArtifact)
    }
}
