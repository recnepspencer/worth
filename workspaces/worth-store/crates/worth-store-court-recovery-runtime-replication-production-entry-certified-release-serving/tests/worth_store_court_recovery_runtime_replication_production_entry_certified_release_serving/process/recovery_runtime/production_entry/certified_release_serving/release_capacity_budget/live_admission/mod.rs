//! Real competing Recovery custody must stop the first release before effects.

#[allow(unused_imports)] use crate::production_entry::certified_release_serving::release_capacity_budget::live_admission::*;
use {worth_proof::AdmittedBlobReleaseProof, worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobReclaimFailure, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimRequest, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::Mutation, std::num::NonZeroU16, std::num::NonZeroU64, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalOperationAllocationScope, worth_store::physical_runtime::PhysicalRecoveryRejoinResidentDenial, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::blob_scope_admission::admitted_blob_scope, crate::production_entry::certified_release_serving::release_capacity_budget::funded_one_head_limit, crate::production_entry::certified_release_serving::release_capacity_budget::numeric_one_head_closure, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery::recover, worth_store_testkit_recovery_runtime::production_entry::release_reopen, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request, crate::production_entry::certified_release_serving::release_capacity_budget::snapshot_family};
use {worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::PhysicalResidencyDimension, worth_store::physical_runtime::PhysicalResidencyRetryPosture, worth_store::physical_runtime::ServingPhysicalRuntime};

#[test]
fn competing_recovery_owner_denies_before_effects_then_partial_drop_reopens() {
    std::thread::Builder::new()
        .name("release-live-recovery-admission".to_owned())
        .stack_size(16 << 20)
        .spawn(run)
        .expect("live release admission worker")
        .join()
        .expect("live release admission worker did not panic");
}
