//! The C.8 rejoin hands Serving a ledger holding released-drop custody. Entry
//! funds its standing capture reservation before Serving is sealed, so a
//! foreign Recovery hold taken after entry, leaving the pool one byte, cannot
//! starve the checkpoint that must certify the custody.

#[allow(unused_imports)] use crate::production_entry::certified_release_serving::rejoin_liveness::*;
use {std::num::NonZeroU64};

use {worth_store::physical_runtime::RecoveryPhysicalAllocation, worth_store::physical_runtime::ServingPhysicalRuntime};
use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::WorthStoreRecovery};

use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery::recover, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request, worth_store_testkit_recovery_runtime::production_entry::release_reopen, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request};

#[test]
fn rejoined_release_custody_checkpoints_under_a_foreign_hold() {
    let (world, _, _) = worth_store_testkit_recovery_runtime::production_entry::release_reopen::released_world(1024);
    checkpoint(world.serving(), [0xc1; 32]);
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);

    let worker = std::thread::Builder::new()
        .name("rejoined-release-custody-liveness".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(&root));
            let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("genuine selected release recovery denied: {outcome:?}")
            };
            let seal = handoff
                .into_core()
                .certification_release_media_into_checkpoint_custody()
                .expect("selected release grants a one-shot custody seal");
            let serving = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(&root, seal);
            assert!(
                serving
                    .certification_checkpoint_capture_custody_bytes()
                    .is_some(),
                "Serving entry funds the rejoined custody's capture reservation"
            );
            let hold = hold_all_but_one_byte(&serving);
            checkpoint(&serving, [0xc2; 32]);
            drop(hold);
            assert!(!serving.close().residency().requires_inspection());
        })
        .expect("rejoin liveness worker");
    worker.join().expect("rejoin liveness worker did not panic");
}
