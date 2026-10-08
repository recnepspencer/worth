use super::*;

#[test]
fn durable_recovery_and_schema_mismatch_test() {
    let mut runtime = persisted_runtime_with_test_schema();
    let _baseline = create_entity_outcome(&runtime, "main-a");

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
    let transitioned = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
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
        .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
        .durable_store_layout(DurableStoreLayout {
            root_path: unique_test_store_path("worth-relational-durable-recovery-schema-match"),
            segment_commit_capacity: 2,
        })
        .build();
    let _outcome = recovered
        .durability_recovery()
        .recover(plan.clone())
        .unwrap();
    let recovered_envelope = recovered
        .replay()
        .canonical_commit_envelope(transitioned.commit.commit_id)
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
        .build();
    let error = mismatched.durability_recovery().recover(plan).unwrap_err();

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

#[test]
fn durability_contract_failure_aspect_plan_mismatch_is_explicit() {
    let runtime =
        persisted_runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    create_entity_outcome(&runtime, "main-a");
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let expected_registry =
        declared_aspect_schema_registry(CascadeDeletePolicy::CascadeDeleteRelations);
    let mismatched_registry = AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(
                crate::tests::support::aspect_key("display_name"),
                crate::tests::support::field_key("name"),
            ),
            lifecycle_aspect(),
        ],
        relation_aspects: vec![
            relation_field_aspect(
                crate::tests::support::aspect_key("label"),
                crate::tests::support::field_key("label"),
            ),
            lifecycle_aspect(),
            relation_source_aspect(),
            relation_target_aspect(),
        ],
        ..AspectSchemaFixture::with_default_declared_aspects(
            CascadeDeletePolicy::CascadeDeleteRelations,
        )
    }
    .build_registry();
    let expected_revision = expected_registry
        .entity_aspect_declaration_trace(KindId(1))
        .unwrap()
        .plan_revision;
    let mismatched_revision = mismatched_registry
        .entity_aspect_declaration_trace(KindId(1))
        .unwrap()
        .plan_revision;
    let mut recovered = RelationalRuntimeApi::builder()
        .schema_registry(mismatched_registry)
        .build();
    let error = recovered.durability_recovery().recover(plan).unwrap_err();

    assert_ne!(expected_revision, mismatched_revision);
    assert_eq!(error.class, RecoveryFailureClass::SchemaMismatch);
    assert!(matches!(
        error.authority_continuity_mismatch,
        Some(RecoveryAuthorityContinuityMismatch::EntityAspectPlanRevision {
            kind_id: KindId(1),
            expected_revision: expected,
            found_revision: found,
            ..
        }) if expected == expected_revision.0 && found == mismatched_revision.0
    ));
}

#[test]
fn durability_contract_failure_relation_integrity_plan_mismatch_is_explicit() {
    let base_registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: crate::schema::data::RelationIntegrityDeclarations::new(
                    Vec::new(),
                    vec![crate::schema::data::CardinalityContractDeclaration {
                        contract_id: "source_max_one".into(),
                        source_max: Some(1),
                        source_min: None,
                        target_max: None,
                        target_min: None,
                        pair_max: None,
                        pair_min: None,
                        pair_min_semantics: crate::schema::data::PairMinimumSemantics::ObservedDirectedPairs,
                        minimum_enforcement:
                            crate::schema::data::MinimumCardinalityEnforcement::CertificationBoundary,
                    }],
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ),
            })
        })
        .unwrap();
    let store_layout = DurableStoreLayout {
        root_path: unique_test_store_path("worth-relational-relation-integrity-mismatch"),
        segment_commit_capacity: 2,
    };
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(base_registry)
        .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
        .durable_store_layout(store_layout.clone())
        .build();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    create_relation_outcome(&runtime, source, target, "guarded");
    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );

    let mismatched_registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: crate::schema::data::RelationIntegrityDeclarations::new(
                    Vec::new(),
                    vec![crate::schema::data::CardinalityContractDeclaration {
                        contract_id: "source_max_two".into(),
                        source_max: Some(2),
                        source_min: None,
                        target_max: None,
                        target_min: None,
                        pair_max: None,
                        pair_min: None,
                        pair_min_semantics: crate::schema::data::PairMinimumSemantics::ObservedDirectedPairs,
                        minimum_enforcement:
                            crate::schema::data::MinimumCardinalityEnforcement::CertificationBoundary,
                    }],
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ),
            })
        })
        .unwrap();
    let mut recovered = RelationalRuntimeApi::builder()
        .schema_registry(mismatched_registry)
        .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
        .durable_store_layout(store_layout)
        .build();
    let error = recovered.durability_recovery().recover(plan).unwrap_err();

    assert_eq!(error.class, RecoveryFailureClass::SchemaMismatch);
    assert!(matches!(
        error.authority_continuity_mismatch,
        Some(RecoveryAuthorityContinuityMismatch::RelationIntegrityPlanRevision {
            kind_id: KindId(2),
            contract_family: RelationIntegrityContractFamily::Cardinality,
            ref expected_contract_ids,
            ref found_contract_ids,
            ..
        }) if expected_contract_ids == &vec![crate::schema::data::ContractId::from("source_max_one")]
            && found_contract_ids == &vec![crate::schema::data::ContractId::from("source_max_two")]
    ));
}
