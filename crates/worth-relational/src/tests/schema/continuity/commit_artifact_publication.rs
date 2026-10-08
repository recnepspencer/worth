use super::*;

#[test]
fn successful_commit_surfaces_descriptor_semantics_version_in_result_and_envelope() {
    let runtime = runtime_with_test_schema();

    let outcome = create_entity_outcome(&runtime, "a");

    assert_eq!(
        outcome.descriptor_semantics_version(),
        DescriptorSemanticsVersion::default()
    );
    assert_eq!(
        outcome.schema_summary().descriptor_semantics_version,
        DescriptorSemanticsVersion::default()
    );
    assert_eq!(
        outcome.envelope().descriptor_semantics_version,
        DescriptorSemanticsVersion::default()
    );
}

#[test]
fn explicit_schema_transition_is_lowered_into_canonical_commit_artifacts() {
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
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: field_key("tag"),
                required: false,
                default_expression: Some(Arc::<str>::from("null")),
            },
        )
        .with_boundary_visibility_proof(
            crate::schema::data::SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable,
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
        WorkerIntentBatch::new("schema-transition").push(MutationIntent::Create(
            CreateIntent::Entity(crate::transactions::data::EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw("b"),
                fields: crate::tests::support::single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                    "b",
                ),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    let transition = outcome.schema_transition_summary().unwrap();
    assert_eq!(transition.changed_atom_count, 1);
    assert_eq!(
        transition.continuation,
        SchemaContinuationClassification::ContinueWithVisibleBridge
    );
    assert!(outcome.envelope().schema_transition.is_some());
    assert!(outcome.envelope().schema_continuation_descriptor.is_some());
    assert!(outcome
        .envelope()
        .schema_reconciliation_descriptor
        .is_some());
    let populated_nested_bytes = outcome
        .envelope()
        .allocation_inventory()
        .authoritative_nested_bytes;
    let mut omitted_schema_authority = outcome.envelope().clone();
    omitted_schema_authority.schema_transition = None;
    omitted_schema_authority.schema_continuation_descriptor = None;
    omitted_schema_authority.schema_reconciliation_descriptor = None;
    assert!(
        populated_nested_bytes
            > omitted_schema_authority
                .allocation_inventory()
                .authoritative_nested_bytes
    );
    assert!(outcome.diagnostics().iter().any(|artifact| artifact.scope
        == DiagnosticsScope::Schema
        && artifact
            .entries
            .iter()
            .any(|entry| { entry.code == DiagnosticCode::SchemaTransitionTraced })));
    let detailed_trace = outcome
        .diagnostics()
        .iter()
        .find(|artifact| {
            artifact.scope == DiagnosticsScope::Schema
                && artifact.kind == DiagnosticsArtifactKind::DetailedTrace
        })
        .expect("schema detailed trace artifact");
    assert!(detailed_trace
        .entries
        .iter()
        .any(|entry| { entry.code == DiagnosticCode::SchemaLineageTraced }));
    assert!(detailed_trace
        .entries
        .iter()
        .any(|entry| { entry.code == DiagnosticCode::SchemaBridgeDescriptorConstructed }));
    assert!(detailed_trace
        .entries
        .iter()
        .any(|entry| { entry.code == DiagnosticCode::SchemaReconciliationResolved }));
    assert!(detailed_trace
        .entries
        .iter()
        .any(|entry| { entry.code == DiagnosticCode::SchemaInterpretationSensitivityClassified }));
    assert!(detailed_trace
        .entries
        .iter()
        .any(|entry| { entry.code == DiagnosticCode::SchemaDescriptorVersionSelected }));
    let diff_entry = detailed_trace
        .entries
        .iter()
        .find(|entry| entry.message.contains("schema diff atom 0"))
        .expect("per-diff schema trace entry");
    assert_eq!(
        diagnostic_field(diff_entry, "strata"),
        &RelationalDiagnosticValue::Array(vec![
            RelationalDiagnosticValue::string("StructuralShape"),
            RelationalDiagnosticValue::string("PublicationContract"),
        ])
    );
    let detail = diagnostic_field(diff_entry, "detail");
    assert_eq!(
        diagnostic_object_field(detail, "kind"),
        &RelationalDiagnosticValue::string("AddedField")
    );
    assert_eq!(
        diagnostic_object_field(detail, "field_path"),
        &RelationalDiagnosticValue::FieldPath(CanonicalFieldPath::single(field_key("tag")))
    );
}

#[test]
fn schema_certification_transition_is_explained_and_counted() {
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
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: field_key("tag"),
                required: false,
                default_expression: Some(Arc::<str>::from("null")),
            },
        )
        .with_boundary_visibility_proof(
            crate::schema::data::SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable,
        )],
    };

    runtime.performance_access().reset_counters();
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
        WorkerIntentBatch::new("schema-transition-certified").push(MutationIntent::Create(
            CreateIntent::Entity(crate::transactions::data::EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw("b"),
                fields: crate::tests::support::single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                    "b",
                ),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    let diagnostics = outcome.diagnostics();
    let detailed_trace = diagnostics
        .iter()
        .find(|artifact| {
            artifact.scope == DiagnosticsScope::Schema
                && artifact.kind == DiagnosticsArtifactKind::DetailedTrace
        })
        .expect("schema detailed trace artifact");
    assert!(detailed_trace
        .entries
        .iter()
        .any(|entry| entry.code == DiagnosticCode::SchemaTransitionClassified));
    assert!(detailed_trace
        .entries
        .iter()
        .any(|entry| entry.code == DiagnosticCode::SchemaBridgeDescriptorConstructed));
    assert!(detailed_trace
        .entries
        .iter()
        .any(|entry| entry.code == DiagnosticCode::SchemaReconciliationResolved));
    assert!(detailed_trace
        .entries
        .iter()
        .any(|entry| { entry.code == DiagnosticCode::SchemaInterpretationSensitivityClassified }));
    assert!(detailed_trace
        .entries
        .iter()
        .any(|entry| entry.code == DiagnosticCode::SchemaDescriptorVersionSelected));

    let counters = runtime.performance_access().counters();
    assert_eq!(counters.schema_transition_atoms_inspected, 1);
    assert_eq!(counters.schema_changed_subtrees_inspected, 1);
    assert_eq!(counters.schema_bridge_descriptors_built, 1);
    assert_eq!(counters.schema_transition_continue_visible_bridge_count, 1);
    assert_eq!(counters.schema_reconciliation_preserve_information_count, 1);
    assert_eq!(
        counters.schema_historical_interpretation_sensitive_boundaries,
        0
    );
}
