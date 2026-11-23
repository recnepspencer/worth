mod artifacts;
pub(crate) mod birth_digest;
pub(crate) mod birth_wire;
pub(crate) mod discriminator_scan;
mod event_serde;
mod events;
mod graph;
mod invariants;
mod metrics;
mod name_resolution;
mod naming;
mod naming_denial;
mod postures;
mod resolution;

pub use artifacts::{
    LineageArtifactCounters, LineageCheckpointArtifact, LineageCheckpointCounters,
    LineageCheckpointDigestBasis, LineageDecisionKind, LineageDecisionLogDigestBasis,
    LineageDecisionRecord, LineageDigestBasis, LineageEventBatchDigestBasis,
};
pub use events::{LineageEventKind, LineageEventRecord, LineageSubject};
pub use graph::{
    LineageDivergenceMetrics, LineageDivergenceRequest, LineageDivergenceSummary,
    LineageDivergenceTraversalBasis, LineageGraphDigestBasis, LineageGraphDigestMode,
    LineageGraphMetrics, LineageGraphRequest, LineageGraphSnapshot, LineageGraphTraversalBasis,
    LineageNode,
};
pub use invariants::LineageInvariant;
pub use metrics::LineageFinalizationCounters;
pub use name_resolution::{
    PersistentNameKey, PersistentNameOfRecord, PersistentNameResolution,
    PersistentNameResolutionDenial, PersistentNameResolutionFootprint,
    PersistentNameResolutionMetrics, PersistentNameResolutionOutcome, PersistentNameSelection,
    PersistentNameTarget,
};
pub use naming::{
    InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry, PersistentNameBirthParts,
    PersistentNameBirthRecord, PersistentNameBirthRequest, PersistentNameDiscriminatorReference,
    PersistentNameHolder, PersistentNameIdentity, PersistentNameReferenceBasis,
    PersistentNameSubjectKind,
};
pub use naming_denial::{DiscriminatorBasisDenialReason, LineageNameDenial, LineagePostureKind};
pub(crate) use postures::LineageBasisView;
#[cfg(test)]
pub use postures::LineageCommitPostureInput;
pub use postures::{
    CommitNamingPlan, LineageMergeKeeper, LineagePostureClaim, LineageRecordTarget,
    LineageSplitChild, PendingBirthTarget, PendingCommitBirth, PendingMergeClaim,
    PendingMergeKeeper, PendingMergeTarget, PendingSplitChild, PendingSplitClaim,
};
pub use resolution::{
    HistoricalLineageResolution, HistoricalLineageResolutionDigestBasis,
    HistoricalLineageResolutionMetrics, HistoricalResolutionBoundednessBasis,
    HistoricalResolutionDigestMode, HistoricalResolutionRequest, HistoricalResolutionTrace,
    RecordHistoryRequest,
};

pub(crate) use artifacts::{
    FinalizedLineageEventBatch, LineageDecisionLog, LineageFinalizationArtifact,
    PreparedLineageFinalization, PublishedLineageArtifact,
};
