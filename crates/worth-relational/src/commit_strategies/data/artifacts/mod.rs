mod artifact_bundle;
pub(crate) mod merge_descriptor;
mod replay_descriptor;

#[cfg(test)]
mod tests;

pub use artifact_bundle::StrategyCommitArtifactBundle;
pub use merge_descriptor::{
    StrategyIntentScopeDigest, StrategyMergeConflictClass, StrategyMergeDescriptor,
    StrategyMergeSemantics,
};
pub use replay_descriptor::StrategyReplayDescriptor;
