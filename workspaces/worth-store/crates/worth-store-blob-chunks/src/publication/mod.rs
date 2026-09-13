//! Publication proof grammar: root candidate → staging → WAL → closeout → visible generation.
//! Parallel recovery lane: pre-WAL replay evidence → recovery evidence → recovered state.
mod classification;
mod counters;
mod denial;
pub(crate) mod evidence;
mod intent;
pub(crate) mod receipt_construction;
pub(crate) mod transitions;
pub(crate) mod types;
pub(crate) mod verification;

pub(crate) use evidence::{
    BlobPublicationCrashBoundaryReport, BlobPublicationCrashOutcome, BlobPublicationDurableWal,
    BlobPublicationReplayCounterSnapshot, BlobPublicationReplayedCrashEdge,
};

#[cfg(test)]
pub(crate) use evidence::{
    BlobPublicationBeforeWalReplayRead, BlobPublicationObservationSet,
    BlobPublicationReplayReadArtifact, BlobPublicationReplayReadDenial,
};

#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
mod tests;

pub use classification::BlobPublicationCrashPoint;
pub use counters::BlobPublicationCounterSnapshot;
pub use denial::{
    reject_copied_publication_record_as_blob_visibility, reject_root_candidate_as_blob_visibility,
    reject_semantic_reference_as_blob_visibility, reject_staged_reachability_as_blob_visibility,
    BlobPublicationDenial,
};
pub use intent::BlobPublicationIntent;
pub use types::{
    BlobGenerationPublished, BlobPublicationAuthority, BlobPublicationPreWalReplayEvidence,
    BlobPublicationRecoveredState, BlobPublicationRecoveryEvidence, BlobPublicationRecoveryReplay,
    BlobPublicationSessionCloseout, BlobPublicationWalCommit, BlobPublicationWalPayload,
    BlobPublicationWalRecord, BlobReachabilityStaging, BlobReachabilityStagingIdentity,
    BlobRootCandidateForPublication, BlobSemanticVisibilityHandoff, BlobSemanticVisibilityOutcome,
    BlobVisibleGeneration,
};
