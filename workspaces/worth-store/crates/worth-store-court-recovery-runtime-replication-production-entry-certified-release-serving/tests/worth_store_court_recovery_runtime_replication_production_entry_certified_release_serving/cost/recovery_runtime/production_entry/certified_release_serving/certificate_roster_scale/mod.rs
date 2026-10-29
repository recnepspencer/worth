//! Maximum production certificate roster through checkpoint, C8 and Serving.

use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_proof::TransitionOutcome, worth_store_recovery_runtime::WorthStoreRecovery, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery::recover, worth_store_testkit_recovery_runtime::production_entry::release_reopen, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request_with_memory};
use {worth_store::physical_runtime::PhysicalOperationAllocationScope};
use {worth_store_physical_format::MAX_CHECKPOINT_CERTIFICATE_RECORDS};



#[test]
#[ignore = "scheduled certificate-roster scale lane"]
fn maximum_certificate_roster_checkpoints_recovers_and_rejoins() {
    std::thread::Builder::new()
        .name("maximum-certificate-roster".to_owned())
        .stack_size(16 << 20)
        .spawn(|| {
            let started = std::time::Instant::now();
            let world = crate::production_entry::certified_release_serving::certificate_roster_scale::world::checkpointed_partial_release_world();
            let selected = std::fs::read(world.root().join("families/checkpoint.current"))
                .expect("maximum roster checkpoint bytes");
            let (batches, accumulator) =
                worth_store_testkit_recovery_runtime::production_entry::release_reopen::selected_release_certificates_from_bytes(&selected);
            assert_eq!(batches.len() as u64 + 1, MAX_CHECKPOINT_CERTIFICATE_RECORDS);
            let policy = world.serving().residency_observation().admitted_policy();
            let recovery_bytes = policy
                .operation_bytes()
                .min(policy.scope_bytes(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery));
            let retained = world.retained_root();
            let root = retained.path();
            drop(world);
            let checkpointed_at = started.elapsed();

            let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(request_with_memory(root, recovery_bytes));
            let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("maximum certificate roster C8 recovery: {outcome:?}")
            };
            let seal = handoff
                .into_core()
                .certification_release_media_into_checkpoint_custody()
                .expect("maximum roster grants genuine C8 custody");
            let recovered_at = started.elapsed();
            let serving = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(root, seal);

            let selected = std::fs::read(root.join("families/checkpoint.current"))
                .expect("selected checkpoint after Serving rejoin");
            let (rejoined_batches, rejoined_accumulator) =
                worth_store_testkit_recovery_runtime::production_entry::release_reopen::selected_release_certificates_from_bytes(&selected);
            assert_eq!(
                rejoined_batches, batches,
                "all 63 Batch certificates survive"
            );
            assert_eq!(rejoined_accumulator, accumulator, "Accumulator survives");
            let checkpoint = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
                worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0xb8; 32]),
                worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(120_000).unwrap(),
            );
            let worth_proof::TransitionOutcome::Success(handle) =
                serving.checkpoints().start(checkpoint).into_raw()
            else {
                panic!("rejoined maximum roster must admit carryforward");
            };
            assert!(matches!(
                handle.wait(),
                worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
            ));
            let carried = std::fs::read(root.join("families/checkpoint.current")).unwrap();
            let (carried_batches, carried_accumulator) =
                worth_store_testkit_recovery_runtime::production_entry::release_reopen::selected_release_certificates_from_bytes(&carried);
            assert!(
                carried_batches.is_empty(),
                "the next checkpoint folds all 63 Batches"
            );
            assert_eq!(carried_accumulator.base().tip(), accumulator.base().tip());
            assert_eq!(
                carried_accumulator.base().cumulative_dropped(),
                accumulator.base().cumulative_dropped()
            );
            assert_eq!(
                carried_accumulator.base().cumulative_digest(),
                accumulator.base().cumulative_digest()
            );
            assert_eq!(
                carried_accumulator.base().prior_checkpoint_sequence(),
                accumulator.base().checkpoint().sequence().get()
            );
            assert_eq!(carried_accumulator.head_count(), accumulator.head_count());
            assert_eq!(
                carried_accumulator.head_roster_digest(),
                accumulator.head_roster_digest()
            );
            serving.close();
            eprintln!(
                "certificate roster: batches={} checkpoint={:?} recovery={:?} rejoin={:?}",
                batches.len(),
                checkpointed_at,
                recovered_at - checkpointed_at,
                started.elapsed() - recovered_at,
            );
        })
        .expect("certificate roster worker")
        .join()
        .expect("certificate roster worker did not panic");
}
