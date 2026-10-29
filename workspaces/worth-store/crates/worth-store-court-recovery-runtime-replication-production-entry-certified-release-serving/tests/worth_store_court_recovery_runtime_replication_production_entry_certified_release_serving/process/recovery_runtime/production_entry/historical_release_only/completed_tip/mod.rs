//! Two completed releases rejoin without an ordinary tail or new checkpoint,
//! then retain exact per-object progress across checkpoint and fresh recovery.

#[allow(unused_imports)] use crate::production_entry::historical_release_only::completed_tip::*;
use {std::path::Path, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_physical_format::RecordArtifactFile, worth_proof::TransitionOutcome, worth_store_recovery_runtime::WorthStoreRecovery, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving, std::fs, worth_store_testkit_recovery_runtime::production_entry::pending_wal_world, worth_store_testkit_recovery_runtime::production_entry::release_reopen, crate::production_entry::historical_release_only::selected_generation};
use {worth_store_physical_format::DurablePhysicalRootManifest, worth_store_physical_format::ReleaseCustodyHeadEntryV1};
use {worth_store_physical_integrity::walk_release_custody_head, worth_store_physical_integrity::ReleaseCustodyHeadWalkLimitsV1};

#[test]
fn two_completed_releases_without_ordinary_tail_serve_checkpoint_and_reopen_exact_heads() {
    std::thread::Builder::new()
        .name("completed-release-tip".into())
        .stack_size(16 << 20)
        .spawn(|| {
            let world = pending_wal_world::first();
            let checkpoint = std::fs::read(world.root().join("families/checkpoint.current")).unwrap();
            world.kill_distinct_release_before_checkpoint();
            let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()));
            let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("real pending second release must recover")
            };
            assert!(handoff.core().recovery_effect_count() > 0);
            let seal = handoff.into_core().certification_release_media_into_checkpoint_custody().unwrap();
            let serving = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(world.root(), seal);
            serving.close();
            let selected = selected_generation(world.root());
            let expected_heads = selected_root_heads(world.root());
            assert_eq!(expected_heads.len(), 2);
            assert_ne!(expected_heads[0].key(), expected_heads[1].key());
            assert!(expected_heads.iter().all(|head| head.cumulative_dropped() == 1));
            assert_eq!(std::fs::read(world.root().join("families/checkpoint.current")).unwrap(), checkpoint,
                "the pending-release recovery must preserve the original NoRelease checkpoint");
            let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()));
            let handoff = match outcome {
                worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) => handoff,
                worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(failure) => panic!(
                    "completed Store rejoin failed: handoff={:?}; reopen={:?}; effects={}",
                    failure.handoff_failure(), failure.reopen_failure(), failure.recovery_effects()),
                worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(block) => panic!(
                    "completed C8 admission blocked: kind={:?}; artifact={:?}; cause={:?}; effects={}",
                    block.cause(), block.evidence().artifact, block.evidence().planning_denial,
                    block.recovery_effects()),
                other => panic!("completed history must independently rejoin Store: {other:?}"),
            };
            assert_eq!(handoff.core().recovery_effect_count(), 0,
                "completed releases must not be repeated");
            assert_eq!(selected_generation(world.root()), selected);
            assert_eq!(std::fs::read(world.root().join("families/checkpoint.current")).unwrap(), checkpoint);
            let seal = handoff.into_core().certification_release_media_into_checkpoint_custody()
                .expect("independently verified completed history must issue a seal");
            let allocations = seal.certification_residency_allocations();
            let recovery = worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(
                worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery);
            assert!(allocations.snapshot().for_dimension(recovery).active_units() > 0,
                "completed-history evidence must retain native backing in the seal");
            let serving = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(world.root(), seal);
            checkpoint_completed_history(&serving);
            serving.close();
            let disposed = allocations.snapshot().for_dimension(recovery);
            assert_eq!(disposed.active_units(), 0);
            assert_eq!(disposed.admitted_units(), disposed.released_units());
            let (_, accumulator) = worth_store_testkit_recovery_runtime::production_entry::release_reopen::selected_release_certificates_from_bytes(
                &std::fs::read(world.root().join("families/checkpoint.current")).unwrap());
            assert_eq!(worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle::selected_heads(world.root(), accumulator),
                expected_heads, "checkpoint must retain both objects' exact progress");
            let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = worth_store_recovery_runtime::WorthStoreRecovery::recover(
                worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root())) else {
                panic!("completed-history checkpoint must freshly recover")
            };
            assert_eq!(handoff.core().recovery_effect_count(), 0);
            let seal = handoff.into_core().certification_release_media_into_checkpoint_custody().unwrap();
            worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal_without_checkpoint(world.root(), seal);
            assert_eq!(selected_root_heads(world.root()), expected_heads,
                "fresh Serving must preserve exact A/B custody");
        })
        .unwrap().join().expect("completed release tip worker");
}
