//! Both parent grammars refuse before root restoration; current bytes recover.
use super::*;

#[test]
fn branch_root_image_versions_refuse_before_restoration_and_current_recovers() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "current-root-format");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();

    for found in [4, 5, 7] {
        let mut plan = crate::tests::support::recovery_plan_for(
            execution,
            &runtime,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
        let roots = &mut plan.checkpoint.as_mut().unwrap().branch_roots;
        assert!(
            !roots.is_empty(),
            "the real checkpoint must contain an owner root"
        );
        for root in roots {
            root.format_version = found;
        }
        let mut recovered = durable_runtime_with_test_schema();
        let failure = recovered
            .durability_recovery()
            .recover(execution, plan)
            .unwrap_err();
        let failure = crate::tests::support::expect_durability_failure(failure);
        assert_eq!(
            failure.class,
            RecoveryFailureClass::UnsupportedBranchRootImageVersion {
                found,
                supported: 6,
            }
        );
        assert_eq!(recovered.history().immutable_commit_count(), 0);
    }

    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = durable_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(
        recovered.history().immutable_commit_count(),
        runtime.history().immutable_commit_count()
    );
}
