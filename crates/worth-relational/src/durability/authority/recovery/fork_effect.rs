//! Point admission consumes actual Complete-or-tail originals and exact root images.
use crate::branch::{
    PositionedBranchTransition, RecordedRelationalComponentReference, RelationalBranchTransition,
    RelationalBranchTransitionSequence,
};
use crate::durability::authority::{
    RecoveredRelationalBranchBasis, RecoveredRelationalRuntimeAuthority,
    RefusedRelationalRecordedComponent,
};
use crate::history::data::RelationalCallerAssociation;
use crate::runtime::RelationalRuntime;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

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
    pub fn source_basis(&self) -> &crate::branch::AdmittedRelationalBranchBasis {
        &self.source.basis
    }
    pub fn target_basis(&self) -> &crate::branch::AdmittedRelationalBranchBasis {
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

impl RecoveredRelationalRuntimeAuthority {
    /// No branch-name fallback or replay constructs the requested effect.
    pub fn admit_recorded_fork(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        sequence: RelationalBranchTransitionSequence,
        association: &RelationalCallerAssociation,
        digest: &[u8; 32],
        source: &RecordedRelationalComponentReference,
        target: &RecordedRelationalComponentReference,
    ) -> Result<ReadmittedRelationalAssociatedFork, RefusedRelationalForkReadmission> {
        use RelationalForkReadmissionCause as Cause;
        let refused = |cause| RefusedRelationalForkReadmission {
            cause,
            source: None,
        };
        crate::execution::check_request(execution)
            .map_err(|cause| refused(Cause::Request(cause)))?;
        if self.runtime_instance_id != runtime.runtime_instance_id() {
            return Err(refused(Cause::ForeignRuntime));
        }
        let _operation = runtime
            .owner_binding()
            .admit()
            .ok_or_else(|| refused(Cause::OwnerUnavailable))?;
        let original = runtime
            .history
            .canonical_checkpoint_gate()
            .original_branch_transition(sequence)
            .ok_or_else(|| refused(Cause::MissingOriginal(sequence)))?;
        if original.caller_association() != Some(association) {
            return Err(refused(Cause::AssociationMismatch));
        }
        if original.recorded_artifact_digest().as_ref() != Some(digest) {
            return Err(refused(Cause::ArtifactDigestMismatch));
        }
        match original.transition() {
            RelationalBranchTransition::Fork {
                target: actual_target,
                source: actual_source,
                source_generation,
                source_truth_version,
                source_head,
            } => {
                let before = source.descriptor();
                if actual_source != before.branch_id()
                    || *source_generation != before.reference().generation()
                    || *source_truth_version != before.truth_version()
                    || before.reference().target().as_basis() != Some(source_head)
                {
                    return Err(refused(Cause::SourceMismatch));
                }
                let after = target.descriptor();
                if actual_target != after.branch_id()
                    || after.reference().generation().get() != 0
                    || after.truth_version() != crate::branch::RelationalBranchVersion::initial()
                    || after.reference().target().as_basis() != Some(source_head)
                {
                    return Err(refused(Cause::TargetMismatch));
                }
            }
            RelationalBranchTransition::RetainedFork { .. }
            | RelationalBranchTransition::Archive { .. }
            | RelationalBranchTransition::DeletionRequested { .. }
            | RelationalBranchTransition::Delete { .. } => return Err(refused(Cause::NotFork)),
        }
        let source = self
            .admit_recorded_component(execution, runtime, source)
            .map_err(|cause| refused(Cause::Component(Box::new(cause))))?;
        match self.admit_recorded_component(execution, runtime, target) {
            Ok(target) => Ok(ReadmittedRelationalAssociatedFork {
                source,
                target,
                original,
            }),
            Err(cause) => Err(RefusedRelationalForkReadmission {
                cause: Cause::Component(Box::new(cause)),
                source: Some(source),
            }),
        }
    }
}
