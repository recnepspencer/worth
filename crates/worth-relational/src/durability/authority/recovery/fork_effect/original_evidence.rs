//! Match the original fork artifact against the exact recorded source and target.
use super::RelationalForkReadmissionCause as Cause;
use crate::branch::recorded_basis::RecordedRelationalComponentReference;
use crate::branch::transition::{PositionedBranchTransition, RelationalBranchTransition};
use crate::branch::version::RelationalBranchVersion;
use crate::history::data::RelationalCallerAssociation;

pub(super) fn validate(
    original: &PositionedBranchTransition,
    association: &RelationalCallerAssociation,
    digest: &[u8; 32],
    source: &RecordedRelationalComponentReference,
    target: &RecordedRelationalComponentReference,
) -> Result<(), Cause> {
    if original.caller_association() != Some(association) {
        return Err(Cause::AssociationMismatch);
    }
    if original.recorded_artifact_digest().as_ref() != Some(digest) {
        return Err(Cause::ArtifactDigestMismatch);
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
                return Err(Cause::SourceMismatch);
            }
            let after = target.descriptor();
            if actual_target != after.branch_id()
                || after.reference().generation().get() != 0
                || after.truth_version() != RelationalBranchVersion::initial()
                || after.reference().target().as_basis() != Some(source_head)
            {
                return Err(Cause::TargetMismatch);
            }
        }
        RelationalBranchTransition::RetainedFork { .. }
        | RelationalBranchTransition::Archive { .. }
        | RelationalBranchTransition::DeletionRequested { .. }
        | RelationalBranchTransition::Delete { .. } => return Err(Cause::NotFork),
    }
    Ok(())
}
