use crate::branch::RelationalBranchTransitionSequence;
use crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifactFrameRefusal;

/// A historical transition read failed. Ordinary reopen never reads covered
/// transitions; explicit replay uses this fallible journal door on demand.
#[derive(Debug)]
pub enum RelationalDurableTransitionReadFailure {
    MediumMoved,
    Missing(RelationalBranchTransitionSequence),
    Unavailable { detail: String },
    Frame(RelationalBranchTransitionArtifactFrameRefusal),
}
