//! The durability adapter frames the Branch-owned canonical body for export.
use worth_relational_contracts::internal_port::durability::canonical_commit_artifact::encode_denial::RelationalCanonicalCommitArtifactEncodeDenial as Denial;
use super::RelationalCanonicalCommitArtifact as Artifact;
use worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit;
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
