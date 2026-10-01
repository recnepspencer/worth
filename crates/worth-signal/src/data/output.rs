mod evaluation;
mod partitions;
mod scope_path;
mod tokens;

pub use evaluation::{
    IntoNodeEvaluationResult, KeyedComputation, MemoizedResultOrigin, NodeEvaluationResult,
    OutputChange,
};
#[cfg(test)]
pub(crate) use partitions::PartitionTokenId;
pub(crate) use partitions::{
    scope_touched_by_artifact_state, scopes_overlap, PreparedPartitionInternerExpansion,
};
pub use partitions::{
    CanonicalChangedRegions, ChangedRegion, InternedPartitionSubscription, InternedScopePath,
    PartitionInterner, PartitionSubscription, PartitionToken,
};
pub use scope_path::{ScopeCoverage, ScopePath, ScopePathError};
pub use tokens::{
    ArtifactContinuityToken, ComputationFamily, ComputationKey, OutputIdentity, StructuralMemoKey,
};
