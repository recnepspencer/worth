use crate::indexes::data::{DerivedIndexBuildRequest, DerivedIndexMaintenanceDenialKind as Denial};
use crate::mvcc::RelationalBranchObservation;
use crate::runtime::VisibilityProjectionView;

pub(super) fn validate(
    request: &DerivedIndexBuildRequest,
    observation: &RelationalBranchObservation,
    before: Option<&VisibilityProjectionView<'_>>,
    after: &VisibilityProjectionView<'_>,
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
    if let Some(before) = before {
        let prior = before.selected_root().ok_or(Denial::BeforeRootMismatch)?;
        let checkpoint = envelope
            .branch_cell_checkpoint
            .as_ref()
            .ok_or(Denial::BeforeRootMismatch)?;
        match checkpoint.observation.target() {
            worth_foundational::FoundationalBranchTarget::Basis(target) => {
                if prior.descriptor() != Some(target.roots())
                    || before.version_id().as_u64() != target.version_id()
                    || prior.commit_id().map(|id| id.0) != Some(target.selected_commit_id())
                {
                    return Err(Denial::BeforeRootMismatch);
                }
            }
            worth_foundational::FoundationalBranchTarget::Empty => {
                if prior.id() != 0 || prior.commit_id().is_some() || !before.version_id().is_zero()
                {
                    return Err(Denial::BeforeRootMismatch);
                }
            }
        }
    }
    Ok(())
}
