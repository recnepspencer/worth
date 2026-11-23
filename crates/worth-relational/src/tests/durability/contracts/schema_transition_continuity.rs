use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn durable_recovery_and_schema_mismatch_test() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = runtime_with_test_schema();
    let _baseline = create_entity_outcome(execution, &runtime, "main-a");

    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::with_default_declared_aspects(
                CascadeDeletePolicy::CascadeDeleteRelations,
            )
        }
        .build_registry(),
    );
    let mut txn = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime)
                .with_schema_transition(
                    schema_transition_for_subscriber_impact(
                        SchemaVersionId(2),
                        SchemaSubscriberImpact::ConsumableSurfaceChanged,
                    ),
                    Some(SchemaReconciliationPolicy::PreserveInformation),
                );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    txn.push_batch(
        batch_create("main-b"),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let transitioned = txn.commit(execution, &runtime).unwrap();

    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let live_schema_transition = transitioned.envelope().schema_transition.clone();
    let live_schema_continuation_descriptor = transitioned
        .envelope()
        .schema_continuation_descriptor
        .clone();
    let live_schema_reconciliation_descriptor = transitioned
        .envelope()
        .schema_reconciliation_descriptor
        .clone();
    let live_recovery_authority_continuity = plan.authority_continuity.clone();

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
        .build()
        .expect("valid native runtime configuration");
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
    let recovered_counters = recovered.performance_access().counters();
    assert!(recovered_counters.replay_digest_parity_checks >= 1);

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
    let error = crate::tests::support::expect_durability_failure(error);

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
