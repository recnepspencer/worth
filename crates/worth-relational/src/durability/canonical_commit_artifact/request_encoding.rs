//! The durability adapter frames the Branch-owned canonical body for export.
use super::encode_denial::RelationalCanonicalCommitArtifactEncodeDenial as Denial;
use super::RelationalCanonicalCommitArtifact as Artifact;
use crate::history::data::PositionedCanonicalCommit;
use worth_execution::ExecutionRequest;

impl Artifact {
    pub(crate) fn from_positioned_request(
        commit: &PositionedCanonicalCommit,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        super::frame_encoding::encode_canonical_frame_for_request(commit, execution)
            .map(Self::from_verified)
    }
}
