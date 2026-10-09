//! Final live planning state must fit before any staging or publication effect.

use std::{fs, io::ErrorKind, path::Path};

use worth_store_physical_format::RecordArtifactFile;
use worth_store_recovery_physics::RecoveryPlanCostDenial;
use worth_store_recovery_runtime::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDimension, PhysicalRecoveryOutcome,
    PhysicalRecoveryPlanningDenial, WorthStoreRecovery,
};

#[test]
fn two_object_pending_final_live_cost_denies_at_peak_minus_one_before_effects() {
    let world = super::super::pending_wal_world::first();
    world.kill_distinct_release_before_checkpoint();
    let root = world.root();
    let current = selector(root, RecordArtifactFile::CurrentRootSelector);
    let previous = selector(root, RecordArtifactFile::PreviousRootSelector);
    let checkpoint = fs::read(root.join("families/checkpoint.current"))
        .expect("genuine selected checkpoint before planning");

    let planned = super::super::certified_release_serving::request(root)
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
    let PhysicalRecoveryOutcome::Refused(cancelled) = planned.cancel_before_execution() else {
        panic!("baseline planning must cancel without execution")
    };
    assert_eq!(cancelled.recovery_effects(), 0);
    assert_unchanged(root, &current, &previous, &checkpoint);

    let outcome = WorthStoreRecovery::recover(
        super::super::certified_release_serving::request_with_memory(root, peak - 1),
    );
    let PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
        panic!("same media at measured peak - 1 must block before effects: {outcome:?}")
    };
    assert_eq!(
        blocked.cause().phase(),
        PhysicalRecoveryBlockKind::RedoPlanning
    );
    assert_eq!(blocked.recovery_effects(), 0);
    let evidence = blocked.evidence();
    assert_eq!(
        evidence.planning_denial,
        Some(PhysicalRecoveryPlanningDenial::Cost(
            RecoveryPlanCostDenial::RecoveryMemoryBytes,
        )),
        "must be the final plan-cost gate, not an earlier source admission"
    );
    let limit = blocked
        .cause()
        .limit()
        .expect("typed final recovery-memory cost");
    assert_eq!(
        limit.dimension(),
        PhysicalRecoveryLimitDimension::RecoveryMemoryBytes
    );
    assert_eq!(limit.observed(), peak);
    assert_eq!(limit.admitted(), peak - 1);
    assert_unchanged(root, &current, &previous, &checkpoint);

    let outcome =
        WorthStoreRecovery::recover(super::super::certified_release_serving::request(root));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("same media with sufficient budget must recover: {outcome:?}")
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("genuine ordered pending Store custody seal");
    super::super::certified_release_serving::open_serving_with_seal_without_checkpoint(root, seal);
}

fn selector(root: &Path, artifact: RecordArtifactFile) -> Option<Vec<u8>> {
    let path = root.join("families/records").join(artifact.file_name());
    match fs::read(path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => panic!("selected selector read failed: {error}"),
    }
}

fn assert_unchanged(
    root: &Path,
    current: &Option<Vec<u8>>,
    previous: &Option<Vec<u8>>,
    checkpoint: &[u8],
) {
    assert_eq!(
        &selector(root, RecordArtifactFile::CurrentRootSelector),
        current,
    );
    assert_eq!(
        &selector(root, RecordArtifactFile::PreviousRootSelector),
        previous,
    );
    assert_eq!(
        fs::read(root.join("families/checkpoint.current")).unwrap(),
        checkpoint
    );
}
