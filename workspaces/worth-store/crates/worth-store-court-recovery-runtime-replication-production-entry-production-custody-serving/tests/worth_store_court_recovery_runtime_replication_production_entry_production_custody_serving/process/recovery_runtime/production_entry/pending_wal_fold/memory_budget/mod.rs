//! Final live planning state must fit before any staging or publication effect.

#[allow(unused_imports)]
use crate::pending_wal_fold::memory_budget::*;
use {std::fs, std::io::ErrorKind, std::path::Path};

use worth_store_physical_format::RecordArtifactFile;
use worth_store_recovery_physics::RecoveryPlanCostDenial;
use {
    worth_store_recovery_runtime::PhysicalRecoveryBlockKind,
    worth_store_recovery_runtime::PhysicalRecoveryLimitDimension,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

#[test]
fn two_object_pending_final_live_cost_denies_at_peak_minus_one_before_effects() {
    let world = worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::first();
    world.kill_distinct_release_before_checkpoint();
    let root = world.root();
    let current = selector(
        root,
        worth_store_physical_format::RecordArtifactFile::CurrentRootSelector,
    );
    let previous = selector(
        root,
        worth_store_physical_format::RecordArtifactFile::PreviousRootSelector,
    );
    let checkpoint = std::fs::read(root.join("families/checkpoint.current"))
        .expect("genuine selected checkpoint before planning");

    let planned = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(root)
        .admit()
        .expect("genuine recovery request admitted")
        .discover()
        .expect("genuine media discovery")
        .select()
        .expect("genuine selected source")
        .plan()
        .expect("sufficient 16 MiB plan");
    let peak = planned.plan_cost().peak_recovery_bytes();
    assert!(peak > 1 && peak <= 16 << 20, "measured peak={peak}");
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Refused(cancelled) =
        planned.cancel_before_execution()
    else {
        panic!("baseline planning must cancel without execution")
    };
    assert_eq!(cancelled.recovery_effects(), 0);
    assert_unchanged(root, &current, &previous, &checkpoint);

    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request_with_memory(root, peak - 1),
    );
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
        panic!("same media at measured peak - 1 must block before effects: {outcome:?}")
    };
    assert_eq!(
        blocked.cause().phase(),
        worth_store_recovery_runtime::PhysicalRecoveryBlockKind::RedoPlanning
    );
    assert_eq!(blocked.recovery_effects(), 0);
    let evidence = blocked.evidence();
    assert_eq!(
        evidence.planning_denial,
        Some(
            worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::Cost(
                worth_store_recovery_physics::RecoveryPlanCostDenial::RecoveryMemoryBytes,
            )
        ),
        "must be the final plan-cost gate, not an earlier source admission"
    );
    let limit = blocked
        .cause()
        .limit()
        .expect("typed final recovery-memory cost");
    assert_eq!(
        limit.dimension(),
        worth_store_recovery_runtime::PhysicalRecoveryLimitDimension::RecoveryMemoryBytes
    );
    assert_eq!(limit.observed(), peak);
    assert_eq!(limit.admitted(), peak - 1);
    assert_unchanged(root, &current, &previous, &checkpoint);

    let outcome =
        worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(root));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("same media with sufficient budget must recover: {outcome:?}")
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("genuine ordered pending Store custody seal");
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal_without_checkpoint(root, seal);
}
