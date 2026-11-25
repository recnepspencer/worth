//! Fork custody for one actual retained immutable historical root.
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::fork::RelationalForkDenial;

/// A move-only retained source, distinct from a freshly current fork source.
/// It retains the original admitted basis; descriptors cannot construct it.
/// A durable receiver records the fork as `RetainedFork`; recovery rebuilds the
/// recorded historical root from durable history and replays the same fork.
#[cfg_attr(not(doctest), doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/facade_examples/retained_fork_source.md")))]
#[derive(Debug)]
pub struct AdmittedRelationalRetainedForkSource {
    pub(super) basis: AdmittedRelationalBranchBasis,
}

pub fn require_retained_fork_cell(
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

pub trait AdmittedRelationalRetainedForkSourceFieldsPort {
    fn into_basis(self) -> AdmittedRelationalBranchBasis;
}

impl AdmittedRelationalRetainedForkSourceFieldsPort for AdmittedRelationalRetainedForkSource {
    fn into_basis(self) -> AdmittedRelationalBranchBasis {
        self.basis
    }
}

pub fn assemble_admitted_relational_retained_fork_source(
    basis: AdmittedRelationalBranchBasis,
) -> AdmittedRelationalRetainedForkSource {
    AdmittedRelationalRetainedForkSource { basis }
}
