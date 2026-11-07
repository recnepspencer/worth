use crate::indexes::data::{DerivedIndexBuildRequest, DerivedIndexMaintenanceDenialKind as Denial};
use crate::mvcc::RelationalBranchObservation;
use crate::runtime::VisibilityProjectionView;

pub(super) fn validate(
    request: &DerivedIndexBuildRequest,
    observation: &RelationalBranchObservation,
    before: Option<&VisibilityProjectionView>,
    after: &VisibilityProjectionView,
) -> Result<(), Denial> {
    if observation.commit_id() != Some(request.source_commit_id)
        || observation.identity().branch_id() != &request.branch_id
    {
        return Err(Denial::CommitMismatch);
    }
    let root = after.selected_root().ok_or(Denial::CommitMismatch)?;
    let envelope = root.canonical_envelope().ok_or(Denial::CommitMismatch)?;
    if envelope.commit.commit_id != request.source_commit_id {
        return Err(Denial::CommitMismatch);
    }
    validate_before(envelope, before)
}

pub(super) fn validate_prepared(
    request: &DerivedIndexBuildRequest,
    before: Option<&VisibilityProjectionView>,
    after: &VisibilityProjectionView,
) -> Result<(), Denial> {
    let root = after.selected_root().ok_or(Denial::CommitMismatch)?;
    let envelope = root.canonical_envelope().ok_or(Denial::CommitMismatch)?;
    if envelope.commit.commit_id != request.source_commit_id
        || envelope.branch_context != request.branch_id
        || envelope.commit.version_id != after.version_id()
    {
        return Err(Denial::CommitMismatch);
    }
    validate_before(envelope, before)
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
