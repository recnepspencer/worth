//! C8 must admit the exact completed Released tip before Store rejoin.
//! This gate does not pretend the unfinished independent Store seal exists.

use super::*;
use worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial;

#[test]
fn two_completed_releases_without_ordinary_tail_reach_independent_store_gate() {
    std::thread::Builder::new()
        .name("completed-release-tip".into())
        .stack_size(16 << 20)
        .spawn(|| {
            let world = pending_wal_world::first();
            let checkpoint = fs::read(world.root().join("families/checkpoint.current")).unwrap();
            world.kill_distinct_release_before_checkpoint();
            let outcome = WorthStoreRecovery::recover(certified_release_serving::request(world.root()));
            let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
                panic!("real pending second release must recover")
            };
            assert!(handoff.core().recovery_effect_count() > 0);
            let seal = handoff.into_core().into_checkpoint_custody().unwrap();
            let serving = certified_release_serving::admit_serving_with_seal(world.root(), seal);
            serving.close();
            let selected = selected_generation(world.root());
            assert_eq!(fs::read(world.root().join("families/checkpoint.current")).unwrap(), checkpoint,
                "the pending-release recovery must preserve the original NoRelease checkpoint");
            let outcome = WorthStoreRecovery::recover(certified_release_serving::request(world.root()));
            match outcome {
                PhysicalRecoveryOutcome::PublicationIndeterminate(failure) => {
                    assert_eq!(failure.handoff_failure(),
                        Some(RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch),
                        "C8 admission must reach the still-closed independent Store boundary");
                    assert_eq!(failure.recovery_effects(), 0,
                        "completed historical releases must not be repeated");
                }
                PhysicalRecoveryOutcome::Blocked(block) => panic!(
                    "completed C8 admission blocked: kind={:?}; artifact={:?}; cause={:?}; effects={}",
                    block.kind, block.evidence().artifact, block.evidence().planning_denial,
                    block.recovery_effects()),
                _ => panic!("completed history must reach independent Store rejoin"),
            }
            assert_eq!(selected_generation(world.root()), selected);
            assert_eq!(fs::read(world.root().join("families/checkpoint.current")).unwrap(), checkpoint);
        })
        .unwrap().join().expect("completed release tip worker");
}
