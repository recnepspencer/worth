//! Point admission consumes actual Complete-or-tail originals and exact root images.
use crate::branch::transition::{PositionedBranchTransition, RelationalBranchTransitionSequence};
use crate::durability::authority::{
    RecoveredRelationalBranchBasis, RefusedRelationalRecordedComponent,
};
use std::sync::Arc;

#[derive(Debug)]
pub enum RelationalForkReadmissionCause {
    Request(crate::execution::RelationalExecutionDenialCause),
    ForeignRuntime,
    OwnerUnavailable,
    MissingOriginal(RelationalBranchTransitionSequence),
    AssociationMismatch,
    ArtifactDigestMismatch,
    NotFork,
    SourceMismatch,
    TargetMismatch,
    Component(Box<RefusedRelationalRecordedComponent>),
}

#[derive(Debug)]
pub struct RefusedRelationalForkReadmission {
    cause: RelationalForkReadmissionCause,
    source: Option<RecoveredRelationalBranchBasis>,
}
impl RefusedRelationalForkReadmission {
    pub fn cause(&self) -> &RelationalForkReadmissionCause {
        &self.cause
    }
    pub fn retains_source_custody(&self) -> bool {
        self.source.is_some()
    }
}

#[derive(Debug)]
pub struct ReadmittedRelationalAssociatedFork {
    source: RecoveredRelationalBranchBasis,
    target: RecoveredRelationalBranchBasis,
    original: Arc<PositionedBranchTransition>,
}
impl ReadmittedRelationalAssociatedFork {
    pub fn source_basis(&self) -> &crate::branch::basis::AdmittedRelationalBranchBasis {
        &self.source.basis
    }
    pub fn target_basis(&self) -> &crate::branch::basis::AdmittedRelationalBranchBasis {
        &self.target.basis
    }

    pub fn original(&self) -> &PositionedBranchTransition {
        &self.original
    }
    pub fn into_parts(
        self,
    ) -> (
        RecoveredRelationalBranchBasis,
        RecoveredRelationalBranchBasis,
        Arc<PositionedBranchTransition>,
    ) {
        (self.source, self.target, self.original)
    }
}

mod runtime_admission;
