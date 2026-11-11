use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn durability_contract_failure_descriptor_semantics_version_mismatch_is_explicit() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "main-a");
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    plan.descriptor_semantics_version = DescriptorSemanticsVersion(99);

    let mut recovered = runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::SchemaMismatch);
    assert!(matches!(
        error.authority_continuity_mismatch,
        Some(
            RecoveryAuthorityContinuityMismatch::DescriptorSemanticsVersion {
                expected: DescriptorSemanticsVersion(99),
                found: DescriptorSemanticsVersion(1),
            }
        )
    ));
}
