use crate::phase_four_planning::{
    ordinary_recovery_declaration, prepare_ordinary_recovery_root_with_pending_record,
};
use worth_store_recovery_physics::RecoveryPlanCostDenial;
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimitDimension, PhysicalRecoveryLimits, PhysicalRecoveryPlanningDenial,
};
use worth_store_testkit_recovery_runtime::phase_three_support::{
    admitted_recovery_with_limits, expect_blocked,
};

#[test]
fn later_staging_refusal_preserves_the_larger_construction_peak_without_effects() {
    // Preserve the ordinary checkpoint/root/WAL lineage, with a genuine
    // multi-frame pending extent instead of the ordinary singleton page.
    let root = prepare_ordinary_recovery_root_with_pending_record(
        "planning-construction-refusal-peak",
        &[0x5a; 16_384],
    );

    let mut early = ordinary_recovery_declaration(4_096);
    early.staging_bytes = 3_276_799;
    let mut later = ordinary_recovery_declaration(4_096);
    later.dirty_frames = 1;
    let mut peaks = Vec::new();
    for (declaration, expected, dimension) in [
        (
            early,
            RecoveryPlanCostDenial::StagingBytes,
            PhysicalRecoveryLimitDimension::StagingBytes,
        ),
        (
            later,
            RecoveryPlanCostDenial::DirtyFrames,
            PhysicalRecoveryLimitDimension::DirtyFrames,
        ),
    ] {
        let selected = admitted_recovery_with_limits(
            root.path(),
            PhysicalRecoveryLimits::admit(declaration).unwrap(),
        )
        .discover()
        .unwrap()
        .select()
        .unwrap();
        let blocked = match selected.plan() {
            Ok(_) => panic!("the original staging limit must refuse before execution"),
            Err(outcome) => expect_blocked(outcome),
        };
        assert_eq!(
            blocked.evidence().planning_denial,
            Some(PhysicalRecoveryPlanningDenial::Cost(expected)),
            "unexpected refusal: {blocked:?}"
        );
        assert_eq!(blocked.cause().limit().unwrap().dimension(), dimension);
        assert_eq!(blocked.recovery_effects(), 0);
        peaks.push(
            blocked
                .evidence()
                .planning_counters
                .unwrap()
                .peak_recovery_bytes(),
        );
    }
    assert!(
        peaks[1] > peaks[0],
        "later scratch must remain measured: {peaks:?}"
    );
}
