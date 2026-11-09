use crate::indexes::data::{DerivedIndexBuildRequest, DerivedIndexMaintenanceDenialKind as Denial};
use crate::mvcc::RelationalBranchObservation;
use crate::runtime::VisibilityProjectionView;

pub(super) fn validate(
    request: &DerivedIndexBuildRequest,
    observation: &RelationalBranchObservation,
    before: Option<&VisibilityProjectionView>,
    after: &VisibilityProjectionView,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    if observation.commit_id() != Some(request.source_commit_id)
        || observation.identity().branch_id() != &request.branch_id
    {
        return Err(Denial::CommitMismatch);
    }
    let root = after.selected_root().ok_or(Denial::CommitMismatch)?;
    if root.commit_id() != Some(request.source_commit_id) {
        return Err(Denial::CommitMismatch);
    }
    validate_before_root(root, before, execution)
}

fn validate_before_root(
    root: &crate::branch::RelationalBranchRoot,
    before: Option<&VisibilityProjectionView>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let Some(before) = before else {
        return Ok(());
    };
    match root.commit_evidence().ok_or(Denial::CommitMismatch)? {
        crate::branch::RelationalRootCommitEvidence::Original(envelope) => {
            validate_before(envelope.envelope(), Some(before))
        }
        crate::branch::RelationalRootCommitEvidence::Checkpoint(commit) => {
            let prior = before.selected_root().ok_or(Denial::BeforeRootMismatch)?;
            commit
                .require_index_predecessor(prior, before.version_id(), execution)
                .map_err(|cause| image_stop(cause, Denial::BeforeRootMismatch))
        }
    }
}

pub(super) fn image_stop(
    cause: crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal,
    mismatch: Denial,
) -> Denial {
    use crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal as Refusal;
    match cause {
        Refusal::Request(cause) => Denial::Execution(cause),
        Refusal::Content(cause) => Denial::Execution(cause.into_execution_cause()),
        Refusal::Allocation(cause) => allocation_stop(cause),
        Refusal::Buffer(cause) => buffer_stop(cause),
        Refusal::CapacityOverflow => Denial::WorkBudgetExceeded,
        Refusal::MissingArtifact
        | Refusal::PredecessorFormat { .. }
        | Refusal::UnsupportedVersion(_)
        | Refusal::InvalidVersion
        | Refusal::Malformed
        | Refusal::BindingMismatch(_)
        | Refusal::AuthenticationMismatch(_)
        | Refusal::MissingRootImage
        | Refusal::InvalidProjectedMeaning
        | Refusal::MissingCaptureOrigin
        | Refusal::OwnerUnavailable
        | Refusal::Native(_) => mismatch,
    }
}

pub(super) fn buffer_stop(cause: worth_execution::ExecutionRequestArrayDenial) -> Denial {
    match cause {
        worth_execution::ExecutionRequestArrayDenial::Request(cause) => {
            Denial::Execution(cause.into())
        }
        worth_execution::ExecutionRequestArrayDenial::Allocation(cause) => allocation_stop(cause),
    }
}

fn allocation_stop(cause: worth_execution::ExecutionAllocationDenial) -> Denial {
    match cause.classify() {
        worth_execution::ExecutionAllocationDenialClass::Request(cause) => {
            Denial::Execution(cause.into())
        }
        worth_execution::ExecutionAllocationDenialClass::Physical(_) => {
            Denial::Execution(crate::execution::RelationalExecutionDenialCause::AllocationFailed)
        }
    }
}

fn validate_before(
    envelope: &crate::history::data::CanonicalCommitEnvelope,
    before: Option<&VisibilityProjectionView>,
) -> Result<(), Denial> {
    if let Some(before) = before {
        validate_root_predecessor(
            envelope,
            before.selected_root().ok_or(Denial::BeforeRootMismatch)?,
        )?;
    }
    Ok(())
}

pub(super) fn validate_root_predecessor(
    envelope: &crate::history::data::CanonicalCommitEnvelope,
    prior: &std::sync::Arc<crate::branch::RelationalBranchRoot>,
) -> Result<(), Denial> {
    let checkpoint = envelope
        .branch_cell_checkpoint
        .as_ref()
        .ok_or(Denial::BeforeRootMismatch)?;
    match checkpoint.observation.target() {
        worth_foundational::FoundationalBranchTarget::Basis(target) => {
            if prior.descriptor() != Some(target.roots())
                || prior.axes().map_or(0, |axes| axes.storage_version) != target.version_id()
                || prior.commit_id().map(|id| id.0) != Some(target.selected_commit_id())
            {
                return Err(Denial::BeforeRootMismatch);
            }
        }
        worth_foundational::FoundationalBranchTarget::Empty => {
            if prior.id() != 0 || prior.commit_id().is_some() || prior.axes().is_some() {
                return Err(Denial::BeforeRootMismatch);
            }
        }
    }
    Ok(())
}
