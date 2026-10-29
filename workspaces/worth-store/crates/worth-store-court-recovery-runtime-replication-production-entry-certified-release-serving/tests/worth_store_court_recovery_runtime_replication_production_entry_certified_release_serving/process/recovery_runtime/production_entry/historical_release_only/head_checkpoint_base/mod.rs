//! Completed B history must rejoin independently from A's selected head checkpoint.

#[allow(unused_imports)] use crate::production_entry::historical_release_only::head_checkpoint_base::*;
use {std::path::Path, worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_proof::TransitionOutcome, worth_store_recovery_runtime::WorthStoreRecovery, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving, crate::production_entry::historical_release_only::completed_tip, std::fs, worth_store_testkit_recovery_runtime::production_entry::pending_wal_world, worth_store_testkit_recovery_runtime::production_entry::release_reopen, crate::production_entry::historical_release_only::selected_generation};
use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest};

#[test]
fn released_checkpoint_base_survives_completed_postcheckpoint_release() {
    std::thread::Builder::new()
        .name("completed-head-checkpoint-base".into())
        .stack_size(16 << 20)
        .spawn(run)
        .unwrap()
        .join()
        .expect("completed HeadV2 recovery worker");
}
