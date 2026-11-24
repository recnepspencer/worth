use super::{
    ReadmittedRelationalAssociatedFork, RefusedRelationalForkReadmission,
    RelationalForkReadmissionCause,
};
use crate::branch::{
    PositionedBranchTransition, RecordedRelationalComponentReference,
    RelationalBranchTransitionSequence,
};
use crate::durability::authority::recovered_runtime::RecoveredRelationalRuntimeAuthority;
use crate::durability::authority::RecoveredRelationalBranchBasis;
use crate::history::data::RelationalCallerAssociation;
use crate::runtime::RelationalRuntime;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

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
        let original = self.validate_recorded_fork(
            execution,
            runtime,
            sequence,
            association,
            digest,
            source,
            target,
        )?;
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

    /// Validate the exact recorded fork transition when the source commit was
    /// durably disposed before reopen, and admit only the surviving target basis.
    pub fn admit_recorded_horizon_fork(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        sequence: RelationalBranchTransitionSequence,
        association: &RelationalCallerAssociation,
        digest: &[u8; 32],
        source: &RecordedRelationalComponentReference,
        target: &RecordedRelationalComponentReference,
    ) -> Result<RecoveredRelationalBranchBasis, RefusedRelationalForkReadmission> {
        use RelationalForkReadmissionCause as Cause;
        let refused = |cause| RefusedRelationalForkReadmission {
            cause,
            source: None,
        };
        let _original = self.validate_recorded_fork(
            execution,
            runtime,
            sequence,
            association,
            digest,
            source,
            target,
        )?;
        self.admit_recorded_component(execution, runtime, target)
            .map_err(|cause| refused(Cause::Component(Box::new(cause))))
    }

    #[allow(clippy::too_many_arguments)]
    fn validate_recorded_fork(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        sequence: RelationalBranchTransitionSequence,
        association: &RelationalCallerAssociation,
        digest: &[u8; 32],
        source: &RecordedRelationalComponentReference,
        target: &RecordedRelationalComponentReference,
    ) -> Result<Arc<PositionedBranchTransition>, RefusedRelationalForkReadmission> {
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
        super::original_evidence::validate(&original, association, digest, source, target)
            .map_err(refused)?;
        Ok(original)
    }
}
