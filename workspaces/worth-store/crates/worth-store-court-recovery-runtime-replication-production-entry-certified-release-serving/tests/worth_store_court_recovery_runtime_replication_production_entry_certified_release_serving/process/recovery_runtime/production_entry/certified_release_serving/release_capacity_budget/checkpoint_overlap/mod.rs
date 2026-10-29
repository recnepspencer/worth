//! A captured checkpoint prefix and a later object's drop coexist without losing either custody.

#[allow(unused_imports)] use crate::production_entry::certified_release_serving::release_capacity_budget::checkpoint_overlap::*;
use {worth_proof::AdmittedBlobReleaseProof, worth_store::physical_runtime::BlobCheckpointLimit, worth_store_blob_chunks::BlobChunkSize, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimRequest, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::Mutation, std::num::NonZeroU16, std::num::NonZeroU64, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalOperationAllocationScope, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope, std::fs, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery::recover, worth_store_testkit_recovery_runtime::production_entry::release_reopen, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request};
use {worth_store::physical_runtime::production::PhysicalCheckpointStep};
use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::BlobReclaimReceipt, worth_store::physical_runtime::PhysicalCheckpointHandle, worth_store::physical_runtime::ServingPhysicalRuntime};
use {worth_store_physical_format::PersistedRecordIdentity, worth_store_physical_format::PhysicalCheckpointSource, worth_store_physical_format::ReleaseCustodyHeadKeyV1, worth_store_physical_format::CHECKPOINT_STREAM_HEADER_RECORD_BYTES};
use {worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld};

#[test]
fn paused_checkpoint_commits_its_prefix_then_later_drop_tail_reopens() {
    std::thread::Builder::new()
        .name("release-checkpoint-overlap".to_owned())
        .stack_size(16 << 20)
        .spawn(run)
        .expect("overlapping checkpoint worker")
        .join()
        .expect("overlapping checkpoint worker did not panic");
}
