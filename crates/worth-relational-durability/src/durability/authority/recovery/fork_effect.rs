//! Point admission consumes actual Complete-or-tail originals and exact root images.
use crate::durability::authority::{
    RecoveredRelationalBranchBasis, RefusedRelationalRecordedComponent,
};
use std::sync::Arc;
use worth_relational_branch::internal_port::branch::transition::positioned::PositionedBranchTransition;
use worth_relational_contracts::internal_port::branch::transition::sequence::RelationalBranchTransitionSequence;

#[derive(Debug)]
pub enum RelationalForkReadmissionCause {
    Request(worth_relational_contracts::internal_port::execution::denial_cause::RelationalExecutionDenialCause),
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
    pub fn source_basis(
        &self,
    ) -> &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis {
        &self.source.basis
    }
    pub fn target_basis(
        &self,
    ) -> &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis {
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

pub mod original_evidence;

#[doc(hidden)]
pub trait RefusedRelationalForkReadmissionOwnerPort: Sized {
    fn from_refusal(
        cause: RelationalForkReadmissionCause,
        source: Option<RecoveredRelationalBranchBasis>,
    ) -> Self;
}

impl RefusedRelationalForkReadmissionOwnerPort for RefusedRelationalForkReadmission {
    fn from_refusal(
        cause: RelationalForkReadmissionCause,
        source: Option<RecoveredRelationalBranchBasis>,
    ) -> Self {
        Self { cause, source }
    }
}

#[doc(hidden)]
pub trait ReadmittedRelationalAssociatedForkOwnerPort: Sized {
    fn from_readmission(
        source: RecoveredRelationalBranchBasis,
        target: RecoveredRelationalBranchBasis,
        original: Arc<PositionedBranchTransition>,
    ) -> Self;
}

impl ReadmittedRelationalAssociatedForkOwnerPort for ReadmittedRelationalAssociatedFork {
    fn from_readmission(
        source: RecoveredRelationalBranchBasis,
        target: RecoveredRelationalBranchBasis,
        original: Arc<PositionedBranchTransition>,
    ) -> Self {
        Self {
            source,
            target,
            original,
        }
    }
}
