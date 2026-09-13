#![forbid(unsafe_code)]
#![deny(unreachable_pub)]

pub mod append;
pub mod artifact_store;
pub mod checkpoint;
pub mod recovery_read;
pub mod wal_topology;

mod blob_records;
mod durability;
mod operation_denial;
mod publication_declaration;
mod security_metadata;
#[cfg(test)]
mod security_metadata_tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurablePublicationPhase {
    Prepared,
    Logged,
    Acknowledged,
    Recovered,
}

pub use append::{
    plan_wal_frame_append, PlannedWalFrameAppend, WalAppendFrontier, WalFramePlanningDenial,
};
pub use artifact_store::{
    inspect_bounded_wal_active_tail_with_evidence, inspect_complete_wal_segment,
    inspect_interrupted_wal_segment_start, inspect_verified_wal_active_tail,
    inspect_verified_wal_segment, observe_checkpoint_artifact, observe_wal_frame_artifact,
    prepare_wal_frame_append, wal_frame_integrity_scope_identity, CheckpointArtifactObservation,
    InterruptedWalSegmentStart, InterruptedWalTail, VerifiedWalActiveTail, VerifiedWalArtifact,
    VerifiedWalFrame, VerifiedWalFramePayload, VerifiedWalSegment, WalActiveTailInspectionDenial,
    WalActiveTailInspectionFailure, WalArtifactInventory, WalArtifactInventoryIdentity,
    WalArtifactInventoryScan, WalArtifactObservation, WalArtifactObservationRead,
    WalArtifactScanCounters, WalArtifactStoreDenial, WalFrameAppendPlan,
    WalFrameArtifactObservation, WalSegmentArtifactIdentity, WalSegmentInspection,
};
#[cfg(feature = "certification-authority")]
pub use artifact_store::{WalAppendPlanner, WalAppendPlannerDenial};
pub use blob_records::{
    durable_phase_for_record_kind, record_kind_admits_recovery_replay, BlobWalRecordEnvelope,
    BlobWalRecordIdentity, BlobWalRecordKind, BlobWalRecordScopeDenial,
    BlobWalReplayRebuildWitness,
};
pub use durability::{WalQueueExecutionDeclaration, WalQueueExecutionKind, WalQueueGroupingScope};
pub use operation_denial::{WalOperationDenial, WalOperationDenialKind};
pub use publication_declaration::{
    CheckpointPublicationScope, PublicationDeclaration, PublicationScope, WalFramePublicationScope,
};
pub use recovery_read::{admit_replay_cursor, inspect_replay_tail_record};
pub use recovery_read::{
    AdmittedReplayTailCursor, WalReplayTailCursorReport, WalReplayTailRecordReport,
};
pub use security_metadata::{
    CheckpointRecordSecurityMetadataEnvelope, StoreCheckpointRecordIdentity,
    StoreWalRecordIdentity, WalRecordSecurityMetadataEnvelope, WalSecurityMetadataCarrier,
    WalSecurityMetadataEnvelope,
};
pub use wal_topology::{
    LogSequenceNumber, ReplayCursor, ReplayCursorSegment, WalFrameOrderingProof, WalLsnRange,
    WalSegmentGeneration, WalSegmentId, WalSegmentScanRecord, WalTopologyDenial,
    WalTopologyDenialKind, WalTopologyScan,
};

pub const PHYSICAL_MUTATION_ATTEMPT_BINDING_DOMAIN: &[u8] =
    b"store.physical.mutation-attempt-binding.v1";
