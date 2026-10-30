#[allow(unused_imports)]
use crate::phase_five_cancellation::*;
use {std::path::Path, std::path::PathBuf};

use worth_proof::TransitionOutcome;
use {
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
};
use {
    worth_store_recovery_runtime::PhysicalRecoveryLimits,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::PhysicalRecoveryStagingDenial,
};
use {
    worth_store_test_support::harness::physical_residency::canonical_durable_wal_attempt_without_execution,
    worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};
use {
    worth_store_testkit_recovery_runtime::phase_three_support::admitted_recovery_with_limits,
    worth_store_testkit_recovery_runtime::phase_three_support::limit_declaration,
};

#[test]
fn cancellation_between_staging_commands_retains_exact_settled_effects() {
    let (_parent, root) = multi_artifact_recovery_world("phase5-partial-cancellation");
    let planned = plan(&root);
    assert!(
        planned.staging_layout().commands().len() >= 2,
        "the causal fixture must stage more than one artifact"
    );
    let cancellation = planned
        .cancellation_after_command(0)
        .expect("the first command is a declared cancellation safe point");

    let Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked)) =
        planned.stage_with_cancellation(cancellation)
    else {
        panic!("partial-effect cancellation terminates Blocked")
    };
    assert!(matches!(
        blocked.evidence().staging_denial,
        Some(
            worth_store_recovery_runtime::PhysicalRecoveryStagingDenial::CancelledAfterPartialStaging {
                settled_commands: 2,
            }
        )
    ));
    let counters = blocked.evidence().staging_counters.unwrap();
    assert_eq!(counters.commands_submitted, 2);
    assert_eq!(counters.commands_settled, 2);
    assert_eq!(counters.scheduler_settlements, 2);
    assert_eq!(counters.performed_effects, 2);
    assert_eq!(
        blocked
            .evidence()
            .staging_settlements
            .as_ref()
            .unwrap()
            .completed(),
        1
    );
    assert_eq!(blocked.recovery_effects(), 2);
}

#[test]
fn cancellation_authority_from_another_plan_has_no_effect() {
    let (_left_parent, left_root) = multi_artifact_recovery_world("phase5-cancel-left");
    let (_right_parent, right_root) = multi_artifact_recovery_world("phase5-cancel-right");
    let left = plan(&left_root);
    let cancellation = left.cancellation_after_command(0).unwrap();
    let right = plan(&right_root);

    let Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked)) =
        right.stage_with_cancellation(cancellation)
    else {
        panic!("a foreign cancellation request cannot enter staging")
    };
    assert_eq!(
        blocked.evidence().staging_denial,
        Some(worth_store_recovery_runtime::PhysicalRecoveryStagingDenial::InvalidPlan)
    );
    let counters = blocked.evidence().staging_counters.unwrap();
    assert_eq!(counters.commands_submitted, 0);
    assert_eq!(counters.performed_effects, 0);
    assert_eq!(blocked.recovery_effects(), 0);
    assert!(matches!(
        left.cancel_before_execution(),
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Refused(_)
    ));
}
