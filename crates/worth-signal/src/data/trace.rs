//! Artifact authority, runtime tiers, retained records, and trace evidence.

mod authority;
mod evidence;
mod records;
mod summary;
mod tiers;
mod writes;

#[cfg(test)]
pub use authority::ArtifactAuthorityClass;
pub use authority::{
    ArtifactMergeAuthority, ArtifactTransitionKey, CompactChangedScopeProof,
    ContinuityAuthorityToken, MergeAdoptability, ReuseOperationalBasis,
};
pub use evidence::CausalityMetadata;
#[cfg(test)]
pub use evidence::SemanticArtifactParity;
pub use records::{ExecutionTraceStamp, HistoricalArtifactRecord, RetainedDiagnosticArtifact};
#[cfg(test)]
pub use summary::assemble_trace_summary;
pub use summary::{
    assemble_historical_artifact_record, assemble_trace_summary_with_execution, TraceSummary,
};
pub use tiers::{
    RuntimeArtifactFinalizeImage, RuntimeArtifactHot, RuntimeArtifactOperationalSummary,
    RuntimeArtifactReuseBoundarySnapshot, RuntimeArtifactState, RuntimeArtifactWarm,
};
pub(crate) use writes::COLD_ARTIFACT_INTENT_LABEL_LIMIT;
pub use writes::{ArtifactWriteDelta, ColdArtifactIntent, ColdArtifactRecord, HotArtifactWrite};
