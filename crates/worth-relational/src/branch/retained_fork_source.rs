//! Process-local fork custody for one actual retained immutable root.
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::fork::RelationalForkDenial;

/// A move-only retained source, distinct from a freshly current fork source.
/// It retains the original admitted basis; descriptors cannot construct it.
/// This profile refuses a durable medium because historical-root reopening is
/// not part of this receiver's contract.
/// A retained source cannot enter the current-source fork receiver:
/// ```compile_fail,E0308
/// use worth_relational::facade::branch::{AdmittedRelationalRetainedForkSource, RelationalForkPort};
/// use worth_relational::facade::history::BranchId;
/// fn current(port: &RelationalForkPort, request: &worth_execution::ExecutionRequest<'_, '_>, target: BranchId, source: AdmittedRelationalRetainedForkSource) {
///     let _ = port.fork_branch(request, target, source);
/// }
/// ```
/// Transfer it through its actual retained-source receiver:
/// ```
/// use worth_relational::facade::branch::{AdmittedRelationalRetainedForkSource, RelationalForkPort};
/// use worth_relational::facade::history::BranchId;
/// fn retained(port: &RelationalForkPort, request: &worth_execution::ExecutionRequest<'_, '_>, target: BranchId, source: AdmittedRelationalRetainedForkSource) {
///     let _ = port.fork_retained_branch(request, target, source);
/// }
/// ```
#[derive(Debug)]
pub struct AdmittedRelationalRetainedForkSource {
    pub(super) basis: AdmittedRelationalBranchBasis,
}

pub(super) fn require_retained_fork_cell(
    source: &AdmittedRelationalBranchBasis,
    cell: &super::reference::RelationalBranchReferenceCell,
) -> Result<(), RelationalForkDenial> {
    if cell.identity() != source.identity() {
        return Err(RelationalForkDenial::ForeignRuntime);
    }
    match cell.lifecycle_posture() {
        crate::branch::lifecycle::RelationalBranchLifecyclePosture::Live => {}
        crate::branch::lifecycle::RelationalBranchLifecyclePosture::Archived => {
            return Err(RelationalForkDenial::SourceArchived)
        }
        crate::branch::lifecycle::RelationalBranchLifecyclePosture::Deleting => {
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
