//! Owned-read control and storage failures remain typed and nonserialized.
use crate::{
    execution::RelationalExecutionDenialCause, storage::data::RelationalStorageUnavailableCause,
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
    CanonicalFrame(crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactFrameRefusal),
    TransitionFrame(crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifactFrameRefusal),
}
impl From<worth_execution::WorkCeilingDenial> for RelationalDurableOwnedReadRefusal {
    fn from(cause: worth_execution::WorkCeilingDenial) -> Self {
        Self::Request(cause.into())
    }
}
