use super::merge_execution_observation::diagnostic_field;
use worth_relational::facade::diagnostics::DiagnosticCode;
use worth_relational::facade::diagnostics::DiagnosticsScope;
use worth_relational::facade::diagnostics::RelationalDiagnosticValue;
use worth_relational::facade::durability::RecoveryAuthorityContinuityMismatch;
use worth_relational::facade::durability::RecoveryFailureClass;
use worth_relational::facade::durability::RecoveryVerificationMode as RecoveryMode;
use worth_relational::facade::durability::RecoveryVerificationMode;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_relational::facade::schema::EntityKindRegistration;
use worth_relational::facade::schema::KindAspectContractDeclarations;
use worth_relational::facade::schema::RelationalSchemaRegistry;
use worth_relational::facade::schema::SchemaId;
use worth_relational::facade::schema::SchemaVersionId;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::create_entity_outcome;
use worth_testkit_relational::durable_publication::durable_runtime;

#[test]
fn durability_contract_failure_schema_mismatch_is_explicit() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity_outcome(execution, &runtime, "main-a");
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
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
    let error = super::checkpoint_recovery_observation::failure(error);

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
fn durability_recovery_plan_preserves_explicit_verification_mode() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity_outcome(execution, &runtime, "main-a");

    let normal = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let audit = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
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
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity_outcome(execution, &runtime, "main-a");
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();

    let mut recovered = durable_runtime(&medium);
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
fn recovery_explains_each_authority_continuity_comparison() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity_outcome(execution, &runtime, "main-a");

    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();

    let mut recovered = durable_runtime(&medium);
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
}
fn verified_at_digest_parity_value() -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "parity",
            RelationalDiagnosticValue::string("VerifiedAtLayer"),
        ),
        (
            "verification_layer",
            RelationalDiagnosticValue::string("DigestParity"),
        ),
    ])
}
