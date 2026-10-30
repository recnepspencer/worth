//! Real killed pending-WAL worlds and their shared child dispatcher.

use worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::first;

#[test]
fn pending_wal_world_reaches_verified_recovery_and_sealed_serving() {
    let world = first();
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()),
    );
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("real pending V3 WAL recovery failed: {outcome:?}")
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("C8 pending V3 custody seal");
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal_without_checkpoint(world.root(), seal);
}

#[test]
fn second_v3_descriptor_after_certified_retirement_checkpoint_is_durable() {
    let world = first();
    world.kill_second_after_certified_retirement();
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()),
    );
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        match outcome {
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) => panic!(
                "third-process C8 blocked at {:?}, effects={}, planning={:?}",
                blocked.cause(),
                blocked.recovery_effects(),
                blocked.evidence().planning_counters
            ),
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
                indeterminate,
            ) => panic!(
                "third-process C8 publication indeterminate, effects={}, handoff={:?}, reopen={:?}",
                indeterminate.recovery_effects(),
                indeterminate.handoff_failure(),
                indeterminate.reopen_failure()
            ),
            other => panic!("third-process C8 must recover second V3 WAL: {other:?}"),
        }
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("second V3 selected custody seal");
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal_without_checkpoint(world.root(), seal);
}

#[test]
fn distinct_v3_descriptor_before_checkpoint_reaches_durable_wal() {
    let world = first();
    world.kill_distinct_release_before_checkpoint();
}

#[test]
fn distinct_v3_descriptor_before_checkpoint_is_durable() {
    let world = first();
    world.kill_distinct_release_before_checkpoint();
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()),
    );
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        match outcome {
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) => panic!(
                "same-checkpoint distinct V3 blocked at {:?}, denial={:?}, artifact={:?}, effects={}",
                blocked.cause(),
                blocked.evidence().planning_denial,
                blocked.evidence().artifact,
                blocked.recovery_effects()
            ),
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
                indeterminate,
            ) => panic!(
                "same-checkpoint distinct V3 publication indeterminate: handoff={:?}, reopen={:?}, effects={}",
                indeterminate.handoff_failure(),
                indeterminate.reopen_failure(),
                indeterminate.recovery_effects(),
            ),
            other => panic!("same-checkpoint distinct V3 sequence must recover: {other:?}"),
        }
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("two-batch C8/Store custody seal");
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal_without_checkpoint(world.root(), seal);
}

#[test]
fn pending_wal_child() {
    worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::dispatch_child();
}
#[path = "three_batch/mod.rs"]
pub(crate) mod three_batch;
