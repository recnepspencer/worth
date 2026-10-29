//! A certified Batch/Accumulator checkpoint is the starting point for two
//! completed postcheckpoint V3 releases and one final killed V3 descriptor.

use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving, worth_store_testkit_recovery_runtime::production_entry::pending_wal_world, worth_store_testkit_recovery_runtime::production_entry::release_reopen};
use {std::fs};
use {worth_store_physical_format::release_checkpoint_batch_records_digest_v1};
use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::WorthStoreRecovery};

#[test]
fn selected_batch_base_with_two_completed_and_one_pending_v3_reopens() {
    let world = pending_wal_world::first();
    world.kill_second_after_certified_retirement();
    let checkpoint = std::fs::read(world.root().join("families/checkpoint.current")).unwrap();
    let (base_batches, base_accumulator) =
        worth_store_testkit_recovery_runtime::production_entry::release_reopen::selected_release_certificates_from_bytes(&checkpoint);
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
        worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle::selected_heads(world.root(), base_accumulator);
    assert_eq!(base_heads.len(), 1);
    world.kill_distinct_release_before_checkpoint();
    world.kill_distinct_release_before_checkpoint();
    assert_eq!(
        std::fs::read(world.root().join("families/checkpoint.current")).unwrap(),
        checkpoint,
        "postcheckpoint V3 chain must not silently replace the Batch base",
    );
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("selected Batch-base ordered V3 history must recover: {outcome:?}");
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("Store must seal the addressed Batch base and all postcheckpoint V3 edges");
    let serving = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(world.root(), seal);
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0xc4; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("ordered Batch-base successor checkpoint must start");
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    serving.close();

    let next = std::fs::read(world.root().join("families/checkpoint.current")).unwrap();
    let (batches, accumulator) =
        worth_store_testkit_recovery_runtime::production_entry::release_reopen::selected_release_certificates_from_bytes(&next);
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
        worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle::selected_heads(world.root(), accumulator);
    assert_eq!(heads.len(), 3);
    assert!(heads.iter().any(|head| head.key() == base_heads[0].key()));
    let fresh = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = fresh else {
        panic!("folded Batch-base chain must fresh-reopen: {fresh:?}");
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("folded checkpoint must still yield a Store seal");
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal_without_checkpoint(world.root(), seal);
}

/// Once the killed descriptor is redone, a second recovery without any new
/// checkpoint sees only completed history above the Batch base: a retirement
/// edge, one release extending the checkpoint-source head, and two first
/// releases. Store must rejoin that history from the checkpoint-source heads.
#[test]
fn selected_batch_base_with_three_completed_v3_recovers_again_before_checkpoint() {
    let world = pending_wal_world::first();
    world.kill_second_after_certified_retirement();
    let checkpoint = std::fs::read(world.root().join("families/checkpoint.current")).unwrap();
    world.kill_distinct_release_before_checkpoint();
    world.kill_distinct_release_before_checkpoint();
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("pending V3 above the Batch base must recover: {outcome:?}");
    };
    drop(handoff);
    assert_eq!(
        std::fs::read(world.root().join("families/checkpoint.current")).unwrap(),
        checkpoint,
        "the second recovery must still start from the Batch base",
    );
    let again = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = again else {
        panic!("completed V3 history above the Batch base must recover: {again:?}");
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("Store must seal the completed head-anchored history");
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal_without_checkpoint(world.root(), seal);
}
