//! Index deltas compare their actual before root to authenticated original axes.
use super::{require_encoded, CheckpointAdmittedCommit, Refusal};
use crate::branch::RelationalBranchRoot;
use crate::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload;
use crate::identity::data::VersionId;
use worth_execution::ExecutionRequest;

impl CheckpointAdmittedCommit {
    pub(crate) fn require_index_predecessor(
        &self,
        prior: &RelationalBranchRoot,
        version: VersionId,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        let row = self
            .0
            .image
            .row(self.index().commit.commit_id)
            .ok_or(Refusal::InvalidProjectedMeaning)?;
        let mut target = BorrowedPayload(self.0.image.group(&row.groups[1]));
        for field in ["predecessor", "reference", "observation", "target"] {
            target = target
                .field(field, request)
                .map_err(super::super::root_positions::projected_stop)?;
        }
        match prior.commit_id() {
            None => {
                if prior.id() != 0 || !version.is_zero() {
                    return Err(Refusal::InvalidProjectedMeaning);
                }
                require_encoded(
                    &worth_foundational::facade::NativeCheckpointBranchTarget::Empty,
                    target.0,
                    request,
                )
            }
            Some(commit) => {
                let target = target
                    .field("Canonical", request)
                    .map_err(super::super::root_positions::projected_stop)?;
                let descriptor = prior.descriptor().ok_or(Refusal::InvalidProjectedMeaning)?;
                for (field, expected) in [("commit", commit.0), ("version", version.as_u64())] {
                    let raw = target
                        .field(field, request)
                        .map_err(super::super::root_positions::projected_stop)?;
                    require_encoded(&expected, raw.0, request)?;
                }
                for (field, expected) in [
                    ("truth_root", descriptor.truth_root()),
                    ("schema_root", descriptor.schema_root()),
                ] {
                    let raw = target
                        .field(field, request)
                        .map_err(super::super::root_positions::projected_stop)?;
                    require_encoded(expected, raw.0, request)?;
                }
                Ok(())
            }
        }
    }
}
