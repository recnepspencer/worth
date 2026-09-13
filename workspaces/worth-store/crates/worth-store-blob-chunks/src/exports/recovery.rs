// --- Capabilities (admission handles, next-step types) ---
pub use crate::recovery::{
    BlobAdmittedRecoveryRecords, BlobRecoveryRecordSet, BlobRecoveryReplay,
    BlobReplayAdmissionDenial, BlobReplayAdmissionDenialKind, BlobReplaySourceAdmission,
    BlobReplaySourceKind, BlobReplaySourceOutcome, BlobReplaySourceOutcomeKind,
    BlobResumeReadmissionAuthority, BlobResumeReplay, BlobResumeReplayReadmission,
    BlobResumeSessionAdmitted, BlobResumeSessionDeclaration, BlobResumeStoreAuthority,
};
// --- Outcomes (transition receipts) ---
pub use crate::recovery::{
    BlobCheckpointFrontierRecord, BlobChunkAppendRecord, BlobGenerationPublicationRecord,
    BlobInterruptedIngestRecovery, BlobManifestAgreement, BlobPersistedResumeCheckpointSource,
    BlobPlacementManifestRow, BlobReachabilityManifestRow, BlobRecoveredPlacementObservation,
    BlobRecoveredPublishedGeneration, BlobRecoveredReachabilityStaging, BlobRecoveredResumeSession,
    BlobRecoveryOutcome, BlobResumeCheckpoint, BlobResumeCheckpointIdentity,
    BlobResumeCheckpointStateKind, BlobResumeChunkAppendStarted, BlobResumeChunkBytesDurable,
    BlobResumeChunkIntegrityAdmitted, BlobResumeFrontierCheckpointed, BlobResumeReplayOutcome,
    BlobResumeRootCandidateBuilt, BlobResumeRootPublicationReady,
    BlobResumeRootPublicationReadyReadmitted, BlobResumeSessionAbandoned,
    BlobResumeSessionCheckpointRecord, BlobResumeSessionClosed, BlobResumeSessionId,
    BlobResumeSessionReclaimed, BlobResumeUnfinishedState, BlobRootCandidateRecord,
};
// --- Denials (classified failure enums) ---
pub use crate::recovery::{
    BlobRecoveryRecordDenial, BlobRecoveryRecordDenialKind, BlobResumeDenial,
};
// --- Counter witnesses (read-only snapshots) ---
pub use crate::recovery::{BlobRecoveryRecordCounterSnapshot, BlobResumeCounterSnapshot};
