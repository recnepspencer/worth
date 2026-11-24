//! Owned-read control and storage failures remain typed and nonserialized.
use crate::{
    durability::{
        branch_transition_artifact::frame_refusal::RelationalBranchTransitionArtifactFrameRefusal,
        canonical_commit_artifact::frame_refusal::RelationalCanonicalCommitArtifactFrameRefusal,
    },
    execution::denial_cause::RelationalExecutionDenialCause,
    storage::data::RelationalStorageUnavailableCause,
};
#[derive(Debug)]
pub enum RelationalDurableOwnedReadRefusal {
    UnsupportedProvider,
    ForeignSource,
    ForeignProtection,
    Bounds,
    Busy,
    Request(RelationalExecutionDenialCause),
    Buffer(worth_execution::ExecutionRequestArrayDenial),
    Storage(RelationalStorageUnavailableCause),
    CanonicalFrame(RelationalCanonicalCommitArtifactFrameRefusal),
    TransitionFrame(RelationalBranchTransitionArtifactFrameRefusal),
}
impl From<worth_execution::WorkCeilingDenial> for RelationalDurableOwnedReadRefusal {
    fn from(cause: worth_execution::WorkCeilingDenial) -> Self {
        Self::Request(cause.into())
    }
}
