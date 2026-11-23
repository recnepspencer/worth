use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn durability_contract_failure_descriptor_semantics_version_mismatch_is_explicit() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "main-a");
    let mut plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
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

#[test]
fn durability_certification_recovery_authority_continuity_is_explained_and_counted() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "main-a");
    runtime.performance_access().reset_counters();
    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();

    let mut recovered = runtime_with_test_schema();
    let _ = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    let diagnostics = recovered.publication().diagnostics();
    let authority_continuity_entry = diagnostics
        .by_scope(DiagnosticsScope::History)
        .into_iter()
        .flat_map(|artifact| artifact.entries.iter())
        .find(|entry| entry.code == DiagnosticCode::DurableRecoveryAuthorityContinuityEvaluated)
        .expect("recovery certification diagnostic");
    assert_eq!(
        diagnostic_field(authority_continuity_entry, "verification_mode"),
        &RelationalDiagnosticValue::string("NormalRecoveryVerification")
    );
    assert_eq!(
        diagnostic_field(authority_continuity_entry, "verification_rejected"),
        &RelationalDiagnosticValue::Bool(false)
    );
    assert_eq!(
        diagnostic_field(authority_continuity_entry, "verification_layer"),
        &RelationalDiagnosticValue::string("DigestParity")
    );
    assert_eq!(
        diagnostic_field(authority_continuity_entry, "descriptor_version_parity"),
        &verified_at_digest_parity_value()
    );
    assert_eq!(
        diagnostic_field(authority_continuity_entry, "schema_transition_parity"),
        &verified_at_digest_parity_value()
    );
    let counters = runtime.performance_access().counters();
    assert!(counters.replay_digest_parity_checks >= 1);
    assert_eq!(counters.descriptor_version_mismatches_encountered, 0);
}
