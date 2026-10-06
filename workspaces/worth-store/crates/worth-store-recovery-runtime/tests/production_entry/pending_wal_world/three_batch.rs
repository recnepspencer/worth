//! Two completed distinct-object V3 batches plus a third OS-killed WAL batch
//! under the same NoRelease checkpoint. No synthetic root or WAL link is used.

use super::*;

#[test]
fn three_distinct_v3_descriptors_before_checkpoint_reach_durable_wal() {
    let world = first();
    world.kill_distinct_release_before_checkpoint();
    world.kill_distinct_release_before_checkpoint();
}

#[test]
fn three_distinct_v3_batches_recover_and_open_serving() {
    let world = first();
    world.kill_distinct_release_before_checkpoint();
    world.kill_distinct_release_before_checkpoint();
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        super::super::certified_release_serving::request(world.root()),
    );
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        match outcome {
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) => panic!(
                "third distinct V3 C8 denied: kind={:?}, artifact={:?}, effects={}",
                blocked.cause(),
                blocked.evidence().artifact,
                blocked.recovery_effects(),
            ),
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
                indeterminate,
            ) => panic!(
                "third distinct V3 Store denied: handoff={:?}, reopen={:?}, effects={}",
                indeterminate.handoff_failure(),
                indeterminate.reopen_failure(),
                indeterminate.recovery_effects(),
            ),
            other => panic!("third distinct V3 recovery denied: {other:?}"),
        }
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("third V3 ordered custody seal");
    super::super::certified_release_serving::open_serving_with_seal_without_checkpoint(
        world.root(),
        seal,
    );
}
