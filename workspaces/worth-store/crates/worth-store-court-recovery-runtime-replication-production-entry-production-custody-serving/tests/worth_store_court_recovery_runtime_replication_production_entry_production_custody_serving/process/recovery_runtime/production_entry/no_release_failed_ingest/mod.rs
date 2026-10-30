//! Genuine failed-ingest cleanup can coexist with selected NoRelease custody.

#[allow(unused_imports)]
use crate::production_entry::no_release_failed_ingest::*;
use worth_store::physical_runtime::BlobTerminalLimits;
use {
    std::num::NonZeroU16, std::num::NonZeroU64, std::path::Path, worth_proof::TransitionOutcome,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store_blob_chunks::BlobChunkSize,
    worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope,
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving,
    worth_store_testkit_recovery_runtime::production_entry::initialized_recovery_world,
};
use {
    worth_store_physical_format::BlobRecordKind,
    worth_store_physical_format::CurrentPhysicalRecordPlacement,
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::SelectedRecordContentClass,
};
use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

#[test]
fn selected_no_release_with_failed_ingest_drop_opens_serving() {
    exercise(false, false);
}

#[test]
fn selected_tier_no_release_with_failed_ingest_drop_opens_serving() {
    exercise(true, false);
}

#[test]
fn changed_failed_ingest_control_after_seal_denies_serving_open() {
    exercise(false, true);
}
