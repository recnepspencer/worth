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
fn recovery_plan_records_typed_authority_parity_before_recovery() {
    use worth_relational::facade::durability::{RecoveryAuthorityParity, RecoveryVerificationOutcome};
    use worth_relational::facade::replay::ReplayVerificationLayer;
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let committed = create_entity_outcome(execution, &runtime, "main-a");

    let plan = runtime
        .durability()
        .recovery_plan_from_source(execution, medium.recovery_source(), RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let authority = &plan.authority_continuity;
    let layer = ReplayVerificationLayer::DigestParity;
    assert_eq!(authority.verification_outcome, RecoveryVerificationOutcome::VerifiedAtLayer(layer));
    assert_eq!(authority.descriptor_version_parity, RecoveryAuthorityParity::VerifiedAtLayer(layer));
    assert_eq!(authority.schema_transition_parity, RecoveryAuthorityParity::VerifiedAtLayer(layer));
    assert_eq!(authority.first_mismatch, None);

    let mut recovered = durable_runtime(&medium);
    recovered.durability_recovery().recover(execution, plan).unwrap();
    assert_eq!(recovered.history().historical_latest_commit(), Some(committed.commit.clone()));
}
