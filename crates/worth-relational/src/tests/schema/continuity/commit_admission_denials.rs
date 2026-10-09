use super::*;

#[test]
fn ordinary_commit_keeps_the_admitted_branch_root_schema_when_live_registry_drifts() {
    let mut runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(&runtime, "a");

    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::default()
        }
        .build_registry(),
    );

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("schema-drift").push(MutationIntent::Create(CreateIntent::Entity(
            crate::transactions::data::EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw("b"),
                fields: crate::tests::support::single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                    "b",
                ),
            },
        ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let committed = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("ambient registry drift cannot reinterpret an admitted branch root");

    assert_eq!(committed.envelope().schema_version, SchemaVersionId(1));
    assert_eq!(
        committed
            .envelope()
            .schema_authority
            .primary_schema_version_id,
        Some(SchemaVersionId(1))
    );
}

#[test]
fn declared_schema_transition_rejects_wrong_source_basis() {
    let mut runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(&runtime, "a");

    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::default()
        }
        .build_registry(),
    );

    let proposed_transition = ProposedSchemaTransition {
        source_schema_id: SchemaId("wrong".to_string()),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: SchemaId("test".to_string()),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("test".to_string()),
                SchemaVersionId(2),
                Some(KindId(1)),
                Arc::<str>::from("tag"),
            ),
            vec![SchemaStratum::StructuralShape],
            SchemaPublicationImpact::None,
            SchemaSubscriberImpact::None,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: field_key("tag"),
                required: false,
                default_expression: None,
            },
        )],
    };

    let mut txn = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime)
                .with_schema_transition(
                    proposed_transition,
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
        batch_create("b"),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let error = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();

    match error {
        crate::transactions::data::TransactionCommitError::Conflict { error, .. } => {
            assert!(matches!(
                error.class,
                ConflictClass::InvalidSchemaTransitionSourceBasis {
                    declared_schema_version: SchemaVersionId(1),
                    expected_schema_version: SchemaVersionId(1),
                    ..
                }
            ));
        }
        other => panic!("expected source-basis conflict, got {other:?}"),
    }
}

#[test]
fn declared_schema_transition_rejects_wrong_target_basis() {
    let mut runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(&runtime, "a");

    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::default()
        }
        .build_registry(),
    );

    let proposed_transition = ProposedSchemaTransition {
        source_schema_id: SchemaId("test".to_string()),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: SchemaId("test".to_string()),
        target_schema_version_id: SchemaVersionId(99),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("test".to_string()),
                SchemaVersionId(2),
                Some(KindId(1)),
                Arc::<str>::from("tag"),
            ),
            vec![SchemaStratum::StructuralShape],
            SchemaPublicationImpact::None,
            SchemaSubscriberImpact::None,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: field_key("tag"),
                required: false,
                default_expression: None,
            },
        )],
    };

    let mut txn = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime)
                .with_schema_transition(
                    proposed_transition,
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
        batch_create("b"),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let error = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();

    match error {
        crate::transactions::data::TransactionCommitError::Conflict { error, .. } => {
            assert!(matches!(
                error.class,
                ConflictClass::InvalidSchemaTransitionTargetBasis {
                    declared_schema_version: SchemaVersionId(99),
                    expected_schema_version: SchemaVersionId(2),
                    ..
                }
            ));
        }
        other => panic!("expected target-basis conflict, got {other:?}"),
    }
    let diagnostics = runtime.publication().diagnostics();
    let failure_artifact = diagnostics
        .by_scope(DiagnosticsScope::Schema)
        .into_iter()
        .find(|artifact| artifact.kind == DiagnosticsArtifactKind::Failure)
        .expect("schema continuity failure artifact");
    assert!(failure_artifact.entries.iter().any(|entry| {
        entry.message.contains("rejected schema diff atom 0")
            && diagnostic_object_field(diagnostic_field(entry, "detail"), "kind")
                == &RelationalDiagnosticValue::string("AddedField")
            && diagnostic_field(entry, "strata")
                == &RelationalDiagnosticValue::Array(vec![RelationalDiagnosticValue::string(
                    "StructuralShape",
                )])
    }));
}

#[test]
fn declared_schema_transition_requires_non_empty_runtime_basis() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .build();

    let proposed_transition = ProposedSchemaTransition {
        source_schema_id: SchemaId("empty".to_string()),
        source_schema_version_id: SchemaVersionId(0),
        target_schema_id: SchemaId("empty".to_string()),
        target_schema_version_id: SchemaVersionId(0),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Schema,
                SchemaId("empty".to_string()),
                SchemaVersionId(0),
                None,
                Arc::<str>::from("root"),
            ),
            vec![SchemaStratum::StructuralShape],
            SchemaPublicationImpact::None,
            SchemaSubscriberImpact::None,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::FreeText {
                detail: Arc::<str>::from("bootstrap"),
                declared_intent: FreeFormSchemaDiffIntent::Additive,
            },
        )],
    };

    let txn = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime)
                .with_schema_transition(
                    proposed_transition,
                    Some(SchemaReconciliationPolicy::PreserveInformation),
                );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    let error = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();

    match error {
        crate::transactions::data::TransactionCommitError::Conflict { error, .. } => {
            assert!(matches!(
                error.class,
                ConflictClass::MissingSchemaBasisForTransition { ref role }
                    if role == "runtime"
            ));
        }
        other => panic!("expected missing-runtime-basis conflict, got {other:?}"),
    }
}

#[test]
fn declared_type_continuity_denied_schema_transition_reports_specific_conflict_class() {
    let mut runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(&runtime, "a");

    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::default()
        }
        .build_registry(),
    );

    let proposed_transition = ProposedSchemaTransition {
        source_schema_id: SchemaId("test".to_string()),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: SchemaId("test".to_string()),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("test".to_string()),
                SchemaVersionId(2),
                Some(KindId(1)),
                Arc::<str>::from("tag"),
            ),
            vec![
                SchemaStratum::ValueDomain,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
            HistoricalInterpretationSensitivity::SensitiveToValueMeaning,
            SchemaDiffDetail::TypeChanged {
                field: field_key("tag"),
                from_type: Arc::<str>::from("string"),
                to_type: Arc::<str>::from("enum<tag>"),
            },
        )],
    };

    let mut txn = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime)
                .with_schema_transition(
                    proposed_transition,
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
        batch_create("b"),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let error = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();

    match error {
        crate::transactions::data::TransactionCommitError::Conflict { error, .. } => {
            assert!(matches!(
                error.class,
                ConflictClass::TypeContinuityDeniedSchemaTransition { .. }
            ));
        }
        other => panic!("expected type-continuity-denied conflict, got {other:?}"),
    }
}
