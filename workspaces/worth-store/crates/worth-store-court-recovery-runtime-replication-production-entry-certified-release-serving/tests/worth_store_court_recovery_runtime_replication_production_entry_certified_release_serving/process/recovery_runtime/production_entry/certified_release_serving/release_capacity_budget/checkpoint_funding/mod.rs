//! A completed partial drop leaves a standing checkpoint reservation. Its
//! checkpoint runs on that reservation even when every other Recovery byte
//! is held, and a cancelled capture leaves the reservation whole.

#[allow(unused_imports)] use crate::production_entry::certified_release_serving::release_capacity_budget::checkpoint_funding::*;
use {worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimRequest, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::Mutation, std::num::NonZeroU16, std::num::NonZeroU64, std::path::Path, std::path::PathBuf, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalOperationAllocationScope, worth_proof::TransitionOutcome, std::fs, crate::production_entry::certified_release_serving::release_capacity_budget::funded_one_head_limit, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery::recover, worth_store_testkit_recovery_runtime::production_entry::release_reopen, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request, crate::production_entry::certified_release_serving::release_capacity_budget::snapshot_family};
use {worth_store::physical_runtime::production::PhysicalCheckpointStep};
use {worth_store::physical_runtime::BlobReclaimRetirement, worth_store::physical_runtime::PhysicalCheckpointCancellationOutcome, worth_store::physical_runtime::PhysicalCheckpointHandle, worth_store::physical_runtime::PhysicalCheckpointProvenNoEffectCause, worth_store::physical_runtime::PhysicalRetirementDenial, worth_store::physical_runtime::ServingPhysicalRuntime};

#[test]
fn partial_drop_checkpoints_on_its_standing_reservation_and_reopens() {
    std::thread::Builder::new()
        .name("release-checkpoint-reservation".to_owned())
        .stack_size(16 << 20)
        .spawn(run)
        .expect("checkpoint reservation worker")
        .join()
        .expect("checkpoint reservation worker did not panic");
}
