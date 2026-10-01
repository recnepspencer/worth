//! A certified Batch/Accumulator checkpoint is the starting point for two
//! completed postcheckpoint V3 releases and one final killed V3 descriptor.

use super::*;
use std::fs;
use worth_store_physical_format::release_checkpoint_batch_records_digest_v1;
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};

#[test]
fn selected_batch_base_with_two_completed_and_one_pending_v3_reopens() {
    let world = pending_wal_world::first();
    world.kill_second_after_certified_retirement();
    let checkpoint = fs::read(world.root().join("families/checkpoint.current")).unwrap();
    let (base_batches, base_accumulator) =
        super::release_reopen::selected_release_certificates_from_bytes(&checkpoint);
    assert_eq!(
        base_batches.len(),
        1,
        "retirement must select an actual Batch base"
    );
    assert_eq!(base_accumulator.base().batch_count(), 1);
    assert_eq!(
        base_accumulator.base().tip(),
        base_batches[0].tip_provenance().unwrap()
    );
    assert!(
        !base_accumulator.base().terminal(),
        "same-object successor needs a nonterminal certified tip"
    );
    let base_heads =
        super::release_reopen::selected_head_oracle::selected_heads(world.root(), base_accumulator);
    assert_eq!(base_heads.len(), 1);
    world.kill_distinct_release_before_checkpoint();
    world.kill_distinct_release_before_checkpoint();
    assert_eq!(
        fs::read(world.root().join("families/checkpoint.current")).unwrap(),
        checkpoint,
        "postcheckpoint V3 chain must not silently replace the Batch base",
    );
    let outcome = WorthStoreRecovery::recover(certified_release_serving::request(world.root()));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("selected Batch-base ordered V3 history must recover: {outcome:?}");
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("Store must seal the addressed Batch base and all postcheckpoint V3 edges");
    let serving = certified_release_serving::admit_serving_with_seal(world.root(), seal);
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0xc4; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("ordered Batch-base successor checkpoint must start");
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    serving.close();

    let next = fs::read(world.root().join("families/checkpoint.current")).unwrap();
    let (batches, accumulator) =
        super::release_reopen::selected_release_certificates_from_bytes(&next);
    assert_eq!(
        batches.len(),
        3,
        "all three postcheckpoint V3 drops must fold"
    );
    assert_eq!(accumulator.base().batch_count(), 3);
    assert_eq!(
        accumulator.base().prior_cumulative_dropped(),
        base_accumulator.base().cumulative_dropped()
    );
    assert_eq!(
        accumulator.base().prior_cumulative_digest(),
        base_accumulator.base().cumulative_digest()
    );
    for (index, batch) in batches.iter().enumerate() {
        assert_eq!(batch.ordinal(), index as u16);
        assert_eq!(
            batch.cumulative_dropped(),
            base_accumulator.base().cumulative_dropped() + index as u64 + 1,
        );
    }
    assert_eq!(
        accumulator.base().batch_records_digest(),
        release_checkpoint_batch_records_digest_v1(&batches).unwrap(),
    );
    assert_eq!(
        accumulator.base().tip(),
        batches[2].tip_provenance().unwrap()
    );
    let heads =
        super::release_reopen::selected_head_oracle::selected_heads(world.root(), accumulator);
    assert_eq!(heads.len(), 3);
    assert!(heads.iter().any(|head| head.key() == base_heads[0].key()));
    let fresh = WorthStoreRecovery::recover(certified_release_serving::request(world.root()));
    let PhysicalRecoveryOutcome::Recovered(handoff) = fresh else {
        panic!("folded Batch-base chain must fresh-reopen: {fresh:?}");
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("folded checkpoint must still yield a Store seal");
    certified_release_serving::open_serving_with_seal_without_checkpoint(world.root(), seal);
}
