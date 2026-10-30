//! Three distinct pending V3 drops become one ordered selected custody ledger.

pub(crate) use std::fs;

pub(crate) use worth_store_physical_format::release_checkpoint_batch_records_digest_v1;
pub(crate) use {
    std::path::Path, worth_proof::TransitionOutcome,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving,
    worth_store_testkit_recovery_runtime::production_entry::pending_wal_world,
    worth_store_testkit_recovery_runtime::production_entry::release_reopen,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

pub(crate) fn recover_serving(
    root: &Path,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(root));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("three-V3 C8/Store recovery must issue custody: {outcome:?}")
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("three-V3 selected custody seal");
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(root, seal)
}
