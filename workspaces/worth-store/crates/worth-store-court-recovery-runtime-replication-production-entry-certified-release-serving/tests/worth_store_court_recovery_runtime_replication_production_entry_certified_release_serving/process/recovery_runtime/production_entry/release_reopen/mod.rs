//! Real Store release, checkpoint, close, and separate-process recovery.

#[allow(unused_imports)] use worth_store_testkit_recovery_runtime::production_entry::release_reopen::*;
use {worth_proof::AdmittedBlobReleaseProof, worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimRequest, std::process::Command, std::num::NonZeroU16, std::num::NonZeroU64, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_proof::TransitionOutcome};
use {std::num::NonZeroU32};
use {worth_store::physical_runtime::BlobReclaimReceipt, worth_store::physical_runtime::BlobReclaimRetirement, worth_store::physical_runtime::BlobReclaimRetirementBudget, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordCountLimit, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanRequest, worth_store::physical_runtime::ServingPhysicalRuntime};
use {worth_store_physical_format::checkpoint_certificate_frame_bytes, worth_store_physical_format::decode_blob_record, worth_store_physical_format::decode_checkpoint_certificate, worth_store_physical_format::BlobReclaimSourceKind, worth_store_physical_format::BlobRecordV1, worth_store_physical_format::CheckpointCertificateKind, worth_store_physical_format::CheckpointStreamFooter, worth_store_physical_format::PersistedRecordIdentity, worth_store_physical_format::ReleaseCheckpointAccumulatorV2, worth_store_physical_format::ReleaseCheckpointBatchV1, worth_store_physical_format::ReleaseCheckpointCertificateV1, worth_store_physical_format::CHECKPOINT_CERTIFICATE_PREFIX_BYTES, worth_store_physical_format::CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES, worth_store_physical_format::CHECKPOINT_CERTIFIED_SCHEMA};

#[cfg(all())]
fn released_world_with_wal_segment_bytes(
    maximum_dropped_records: u16,
    wal_segment_bytes: NonZeroU64,
) -> (
    PhysicalResidencyStoreWorld,
    BlobReclaimReceipt,
    PersistedRecordIdentity,
    AdmittedBlobReleaseProof,
) {
    released_world_in_tier(maximum_dropped_records, false, Some(wal_segment_bytes))
}
#[path = "candidate_crash/mod.rs"]
pub(crate) mod candidate_crash;
#[cfg(all())]
#[path = "per_object_pruned_successor/mod.rs"]
pub(crate) mod per_object_pruned_successor;
#[cfg(all())]
#[path = "shared_reuse_custody/mod.rs"]
pub(crate) mod shared_reuse_custody;
