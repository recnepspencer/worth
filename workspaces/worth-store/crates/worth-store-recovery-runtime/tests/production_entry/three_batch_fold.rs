//! Three distinct pending V3 drops become one ordered selected custody ledger.

use std::fs;

use super::*;
use worth_store_physical_format::release_checkpoint_batch_records_digest_v1;
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};

#[test]
fn three_distinct_pending_v3_batches_fold_and_fresh_reopen() {
    let world = super::pending_wal_world::first();
    world.kill_distinct_release_before_checkpoint();
    world.kill_distinct_release_before_checkpoint();

    let serving = recover_serving(world.root());
    let checkpoint = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x9d; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(checkpoint).into_raw()
    else {
        panic!("three pending V3 batches must admit one successor checkpoint");
    };
    let outcome = handle.wait();
    assert!(
        matches!(outcome, PhysicalCheckpointOutcome::Completed(_)),
        "three-batch checkpoint must complete: {outcome:?}"
    );
    serving.close();

    let bytes = fs::read(world.root().join("families/checkpoint.current")).unwrap();
    let (batches, accumulator) =
        super::release_reopen::selected_release_certificates_from_bytes(&bytes);
    assert_eq!(
        batches.len(),
        3,
        "checkpoint must fold every genuine V3 member"
    );
    assert_eq!(accumulator.base().batch_count(), 3);
    for (index, batch) in batches.iter().enumerate() {
        assert_eq!(batch.ordinal(), index as u16);
        assert_eq!(batch.cumulative_dropped(), (index + 1) as u64);
    }
    for pair in batches.windows(2) {
        assert_ne!(pair[0].descriptor_record(), pair[1].descriptor_record());
        assert!(
            pair[0].candidate_root_generation() < pair[1].candidate_root_generation(),
            "selected Batch order must follow actual publication order"
        );
    }
    assert_ne!(
        batches[0].descriptor_record(),
        batches[2].descriptor_record()
    );
    assert_eq!(
        accumulator.base().batch_records_digest(),
        release_checkpoint_batch_records_digest_v1(&batches).unwrap()
    );
    assert_eq!(
        accumulator.base().tip(),
        batches[2].tip_provenance().unwrap()
    );
    assert_eq!(
        accumulator.base().cumulative_dropped(),
        batches[2].cumulative_dropped()
    );
    assert_eq!(
        accumulator.base().cumulative_digest(),
        batches[2].cumulative_digest()
    );
    let heads =
        super::release_reopen::selected_head_oracle::selected_heads(world.root(), accumulator);
    assert_eq!(heads.len(), 3);

    let fresh = recover_serving(world.root());
    fresh.close();
}

fn recover_serving(root: &Path) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    let outcome = WorthStoreRecovery::recover(super::certified_release_serving::request(root));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("three-V3 C8/Store recovery must issue custody: {outcome:?}")
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("three-V3 selected custody seal");
    super::certified_release_serving::admit_serving_with_seal(root, seal)
}
