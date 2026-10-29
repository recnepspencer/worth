//! A selected Tier checkpoint and process-killed V3 drop share one recovery suffix.

#[allow(unused_imports)] use worth_store_testkit_recovery_runtime::production_entry::tier_release_pending_wal::*;
use {worth_proof::AdmittedBlobReleaseProof, worth_store::physical_runtime::BlobCheckpointLimit, worth_store_blob_chunks::BlobChunkSize, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobIngestFailure, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReclaimRequest, std::num::NonZeroU16, std::num::NonZeroU64, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope, worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving};
use {std::collections::BTreeSet, std::fs, std::path::Path, std::path::PathBuf, std::process::Command, std::process::Stdio, std::sync::atomic::AtomicBool, std::sync::atomic::Ordering, std::thread, std::time::Duration, std::time::Instant};
use {worth_store::physical_runtime::production::PhysicalMutationCheckpoint, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordCountLimit, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanRequest};
use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobReclaimSourceBasisV1, worth_store_physical_format::BlobRecordKind, worth_store_physical_format::BlobRecordV1, worth_store_physical_format::SelectedRecordContentClass};


use {worth_proof::AdmittedBlobReleaseProof, worth_store::physical_runtime::BlobCheckpointLimit, worth_store_blob_chunks::BlobChunkSize, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobIngestFailure, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReclaimRequest, std::num::NonZeroU16, std::num::NonZeroU64, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope, worth_store_testkit_recovery_runtime::production_entry::tier_release_pending_wal::checkpoint::assert_tier_and_batch, worth_store_testkit_recovery_runtime::production_entry::tier_release_pending_wal::checkpoint::assert_tier_and_no_release, worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving, worth_store_testkit_recovery_runtime::production_entry::tier_release_pending_wal::checkpoint::selected_checkpoint};

#[test]
fn tier_checkpoint_then_killed_released_wal_reopens_once() {
    std::thread::Builder::new()
        .name("mixed-tier-release-recovery".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(run_tier_checkpoint_then_killed_released_wal)
        .unwrap()
        .join()
        .expect("mixed Tier/V3 recovery worker");
}

#[test]
fn mixed_tier_pending_child() {
    let Some(marker) = std::env::var_os(MARKER_ENV) else {
        return;
    };
    assert_eq!(std::env::var(ROLE_ENV).as_deref(), Ok("produce"));
    produce_pending_world(std::path::Path::new(&marker));
}
