use super::*;

#[test]
fn durability_contract_failure_schema_mismatch_is_explicit() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "main-a");
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
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
        .build();
    let error = recovered.durability_recovery().recover(plan).unwrap_err();

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
    let runtime = runtime_with_test_schema();
    create_entity_outcome(&runtime, "main-a");
    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    plan.descriptor_semantics_version = DescriptorSemanticsVersion(99);

    let mut recovered = runtime_with_test_schema();
    let error = recovered.durability_recovery().recover(plan).unwrap_err();

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
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "main-a");

    let normal = runtime
        .durability()
        .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);
    let audit = runtime
        .durability()
        .recovery_plan(RecoveryVerificationMode::AuditRecoveryVerification);

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
fn durability_recovery_plan_reports_descriptor_version_mismatch_before_recovery() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "main-a");
    runtime.performance_access().reset_counters();
    let store = runtime
        .durability()
        .recovery_plan(
            crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .store
        .unwrap();
    let segment_path = store
        .segments
        .last()
        .expect("persisted segment after commit")
        .path
        .clone();
    let mut file =
        crate::durability::log::native_file_codec::read_segment_file(&segment_path).unwrap();
    file.entries[0]
        .envelope_mut_for_test()
        .descriptor_semantics_version = DescriptorSemanticsVersion(99);
    crate::durability::log::native_file_codec::write_segment_file(&segment_path, &file).unwrap();

    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );

    assert_eq!(
        plan.descriptor_semantics_version,
        DescriptorSemanticsVersion(99)
    );
    assert_eq!(
        plan.authority_continuity.descriptor_version_parity,
        RecoveryAuthorityParity::Drift
    );
    assert!(matches!(
        plan.authority_continuity.first_mismatch,
        Some(
            RecoveryAuthorityContinuityMismatch::DescriptorSemanticsVersion {
                expected: DescriptorSemanticsVersion(1),
                found: DescriptorSemanticsVersion(99),
            }
        )
    ));
    assert_eq!(
        plan.authority_continuity.verification_outcome,
        RecoveryVerificationOutcome::Rejected {
            layer: ReplayVerificationLayer::DigestParity,
            detail: "descriptor semantics version mismatch".to_string(),
        }
    );
    let counters = runtime.performance_access().counters();
    assert!(counters.descriptor_version_mismatches_encountered >= 1);
    assert!(counters.replay_digest_parity_checks >= 1);
}

#[test]
fn durability_contract_failure_descriptor_canonical_basis_version_mismatch_is_explicit() {
    let mut runtime = persisted_runtime_with_test_schema();
    let _ = create_entity_outcome(&runtime, "main-a");

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
        batch_create("transitioned"),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(
        &runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();

    let segment_path = runtime
        .durability()
        .recovery_plan(
            crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .store
        .unwrap()
        .segments
        .last()
        .expect("persisted segment after transition")
        .path
        .clone();
    let mut file =
        crate::durability::log::native_file_codec::read_segment_file(&segment_path).unwrap();
    if let Some(descriptor) = file.entries[1]
        .envelope_mut_for_test()
        .schema_continuation_descriptor
        .as_mut()
    {
        descriptor.bridge.canonical_basis_version = DescriptorCanonicalBasisVersion(99);
    }
    crate::durability::log::native_file_codec::write_segment_file(&segment_path, &file).unwrap();

    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );

    assert_eq!(
        plan.authority_continuity.descriptor_version_parity,
        RecoveryAuthorityParity::Drift
    );
    assert!(matches!(
        plan.authority_continuity.first_mismatch,
        Some(
            RecoveryAuthorityContinuityMismatch::DescriptorCanonicalBasisVersion {
                expected: DescriptorCanonicalBasisVersion(1),
                found: DescriptorCanonicalBasisVersion(99),
            }
        )
    ));
    assert_eq!(
        plan.authority_continuity.verification_outcome,
        RecoveryVerificationOutcome::Rejected {
            layer: ReplayVerificationLayer::DigestParity,
            detail: "descriptor canonical basis version mismatch".to_string(),
        }
    );
}

#[test]
fn durability_recovery_emits_authority_continuity_diagnostic_before_execution() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "main-a");
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );

    let mut recovered = persisted_runtime_with_test_schema();
    let _ = recovered.durability_recovery().recover(plan).unwrap();

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
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "main-a");
    runtime.performance_access().reset_counters();
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );

    let mut recovered = persisted_runtime_with_test_schema();
    let _ = recovered.durability_recovery().recover(plan).unwrap();

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
