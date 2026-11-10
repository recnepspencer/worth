use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn durability_contract_failure_schema_mismatch_is_explicit() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "main-a");
    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mismatched_registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(3),
            kind_name: "other.entity".to_string(),
            schema_id: SchemaId("other".to_string()),
            schema_version_id: SchemaVersionId(2),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .unwrap();
    let mut recovered = RelationalRuntimeApi::builder()
        .schema_registry(mismatched_registry)
        .build()
        .expect("valid native runtime configuration");
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::SchemaMismatch);
    assert!(matches!(
        error.authority_continuity_mismatch,
        Some(RecoveryAuthorityContinuityMismatch::SchemaRegistryShape {
            expected_primary_schema_version: SchemaVersionId(1),
            found_primary_schema_version: SchemaVersionId(2),
            ..
        })
    ));
}

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

#[test]
fn durability_recovery_plan_preserves_explicit_verification_mode() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "main-a");

    let normal = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    let audit = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryVerificationMode::AuditRecoveryVerification,
    )
    .unwrap();

    assert_eq!(
        normal.verification_mode(),
        RecoveryVerificationMode::NormalRecoveryVerification
    );
    assert_eq!(
        audit.verification_mode(),
        RecoveryVerificationMode::AuditRecoveryVerification
    );
}

#[test]
fn durability_recovery_emits_authority_continuity_diagnostic_before_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "main-a");
    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();

    let mut recovered = durable_runtime_with_test_schema();
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
        .expect("recovery authority continuity diagnostic");
    assert_eq!(
        diagnostic_field(authority_continuity_entry, "verification_rejected"),
        &RelationalDiagnosticValue::Bool(false)
    );
    assert_eq!(
        diagnostic_field(authority_continuity_entry, "verification_layer"),
        &RelationalDiagnosticValue::string("DigestParity")
    );
}

#[test]
fn durability_certification_recovery_authority_continuity_is_explained_and_counted() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "main-a");
    runtime.performance_access().reset_counters();
    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();

    let mut recovered = durable_runtime_with_test_schema();
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
