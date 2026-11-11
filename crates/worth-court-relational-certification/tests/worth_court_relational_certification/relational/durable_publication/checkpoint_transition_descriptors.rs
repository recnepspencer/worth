use super::merge_execution_observation::diagnostic_field;
use super::merge_execution_observation::diagnostic_field_optional;
use worth_relational::facade::config::CascadeDeletePolicy;
use worth_relational::facade::config::RelationalRuntimeProfile;
use worth_relational::facade::diagnostics::DiagnosticCode;
use worth_relational::facade::diagnostics::DiagnosticsScope;
use worth_relational::facade::diagnostics::RelationalDiagnosticValue;
use worth_relational::facade::durability::RecoveryAuthorityContinuityMismatch;
use worth_relational::facade::durability::RecoveryFailureClass;
use worth_relational::facade::durability::RecoveryVerificationMode as RecoveryMode;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_relational::facade::schema::EntityKindRegistration;
use worth_relational::facade::schema::KindAspectContractDeclarations;
use worth_relational::facade::schema::RelationalSchemaRegistry;
use worth_relational::facade::schema::SchemaId;
use worth_relational::facade::schema::SchemaReconciliationPolicy;
use worth_relational::facade::schema::SchemaVersionId;
use worth_testkit_relational::aspect_storage::AspectSchemaFixture;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::batch_create;
use worth_testkit_relational::durable_publication::create_entity_outcome;
use worth_testkit_relational::durable_publication::durable_runtime;

#[test]
fn durable_recovery_and_schema_mismatch_test() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let _baseline = create_entity_outcome(execution, &runtime, "main-a");

    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut txn = runtime
        .begin_branch_schema_transition(
            &basis,
            super::checkpoint_schema_versions::transition(SchemaVersionId(2)),
            Some(SchemaReconciliationPolicy::PreserveInformation),
            super::checkpoint_schema_versions::registry(2),
        )
        .unwrap();
    txn.push_batch(
        batch_create("main-b"),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let transitioned = txn.commit(execution, &runtime).unwrap();

    let live_schema_transition = transitioned.envelope().schema_transition.clone();
    let live_schema_continuation_descriptor = transitioned
        .envelope()
        .schema_continuation_descriptor
        .clone();
    let live_schema_reconciliation_descriptor = transitioned
        .envelope()
        .schema_reconciliation_descriptor
        .clone();
    let mut recovered = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(
            AspectSchemaFixture {
                schema_version_id: SchemaVersionId(2),
                ..AspectSchemaFixture::with_default_declared_aspects(
                    CascadeDeletePolicy::CascadeDeleteRelations,
                )
            }
            .build_registry(),
        )
        .build_durable(medium.attachment())
        .expect("valid native runtime configuration");
    let plan = recovered
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let live_recovery_authority_continuity = plan.authority_continuity.clone();

    let _outcome = recovered
        .durability_recovery()
        .recover(execution, plan.clone())
        .unwrap();
    let recovered_envelope = recovered
        .replay()
        .canonical_commit_envelope(execution, transitioned.commit.commit_id)
        .unwrap()
        .expect("recovered transitioned envelope");
    let recovered_diagnostics = recovered.publication().diagnostics();
    let recovery_authority_continuity_diagnostic = recovered_diagnostics
        .by_scope(DiagnosticsScope::History)
        .into_iter()
        .flat_map(|artifact| artifact.entries.iter())
        .find(|entry| entry.code == DiagnosticCode::DurableRecoveryAuthorityContinuityEvaluated)
        .expect("recovery authority continuity diagnostic");

    assert_eq!(
        live_schema_transition,
        recovered_envelope.schema_transition.clone()
    );
    assert_eq!(
        live_schema_continuation_descriptor,
        recovered_envelope.schema_continuation_descriptor.clone()
    );
    assert_eq!(
        live_schema_reconciliation_descriptor,
        recovered_envelope.schema_reconciliation_descriptor.clone()
    );
    assert_eq!(
        live_recovery_authority_continuity,
        plan.authority_continuity.clone()
    );
    assert!(recovered_diagnostics
        .by_scope(DiagnosticsScope::History)
        .into_iter()
        .flat_map(|artifact| artifact.entries.iter())
        .any(|entry| {
            entry.code == DiagnosticCode::DurableRecoveryAuthorityContinuityEvaluated
                && diagnostic_field_optional(entry, "verification_layer")
                    == Some(&RelationalDiagnosticValue::string("DigestParity"))
        }));

    let mismatched_registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(3),
            kind_name: "other.entity".to_string(),
            schema_id: SchemaId("other".to_string()),
            schema_version_id: SchemaVersionId(99),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .unwrap();
    let mut mismatched = RelationalRuntimeApi::builder()
        .schema_registry(mismatched_registry)
        .build()
        .expect("valid native runtime configuration");
    let error = mismatched
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = super::checkpoint_recovery_observation::failure(error);

    assert_eq!(error.class, RecoveryFailureClass::SchemaMismatch);
    assert!(matches!(
        error.authority_continuity_mismatch,
        Some(RecoveryAuthorityContinuityMismatch::SchemaRegistryShape { .. })
    ));
    assert!(matches!(
        error.authority_continuity_mismatch,
        Some(RecoveryAuthorityContinuityMismatch::SchemaRegistryShape {
            expected_primary_schema_version: SchemaVersionId(2),
            ..
        })
    ));
    assert_eq!(
        diagnostic_field(
            recovery_authority_continuity_diagnostic,
            "verification_layer"
        ),
        &RelationalDiagnosticValue::string("DigestParity")
    );
    assert!(!error.detail.is_empty());
}
