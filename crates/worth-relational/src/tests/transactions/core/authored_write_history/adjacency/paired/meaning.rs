//! The exact roots carry installed relation and endpoint meaning, not just kind IDs.
use super::*;
use crate::schema::data::*;

#[test]
fn paired_membership_refuses_incompatible_installed_relation_or_endpoint_meaning() {
    for endpoint in [false, true] {
        let host = crate::test_execution_policy::serial_request();
        let execution = ExecutionRequest::serial(&host);
        let mut runtime = runtime_with_test_schema();
        let a = create_entity(&execution, &runtime, "source");
        let b = create_entity(&execution, &runtime, "target");
        let edge = create_relation_outcome(&execution, &runtime, a, b, "edge");
        let source = test_owner_main_basis(&runtime).unwrap();
        let old = runtime
            .read_truth()
            .project_snapshot(&edge.snapshot)
            .unwrap();
        let schema = AspectSchemaFixture::with_default_declared_aspects(
            CascadeDeletePolicy::CascadeDeleteRelations,
        );
        let old_registry = schema.build_registry();
        let mut entity = old_registry.entity_registration(KindId(1)).unwrap().clone();
        let mut relation = old_registry
            .relation_registration(KindId(2))
            .unwrap()
            .clone();
        // Native requires one authoritative schema basis across ALL kinds.
        // Advance that real basis coherently before changing either meaning.
        entity.schema_version_id = SchemaVersionId(2);
        relation.schema_version_id = SchemaVersionId(2);
        if endpoint {
            entity.kind_name.push_str("-changed");
        } else {
            relation.kind_name.push_str("-changed");
        }
        let registry = RelationalSchemaRegistry::new()
            .register_entity_kind(entity)
            .unwrap()
            .register_relation_kind(relation)
            .unwrap();
        runtime.set_schema_registry_for_test(registry);
        let transition = ProposedSchemaTransition {
            source_schema_id: SchemaId("test".into()),
            source_schema_version_id: SchemaVersionId(1),
            target_schema_id: SchemaId("test".into()),
            target_schema_version_id: SchemaVersionId(2),
            diff_atoms: vec![SchemaDiffAtom::new(
                SchemaElementRef::new(
                    SchemaElementKind::ProjectionContract,
                    SchemaId("test".into()),
                    SchemaVersionId(2),
                    Some(KindId(2)),
                    "adjacency-meaning",
                ),
                vec![SchemaStratum::PublicationContract],
                SchemaPublicationImpact::ObservableSurfaceChanged,
                SchemaSubscriberImpact::ConsumableSurfaceChanged,
                HistoricalInterpretationSensitivity::NotSensitive,
                SchemaDiffDetail::ProjectionContractChanged {
                    projection_name: "adjacency-meaning".into(),
                },
            )
            .with_boundary_visibility_proof(
                SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable,
            )],
        };
        let input = test_owner_transaction_validation_input_for_main(&runtime)
            .with_schema_transition(
                transition,
                Some(SchemaReconciliationPolicy::PreserveInformation),
            );
        let mut transaction = runtime
            .begin_branch_transaction(input.basis(), input.intent().clone())
            .unwrap();
        transaction
            .push_batch(
                WorkerIntentBatch::new("install-meaning").push(MutationIntent::Create(
                    CreateIntent::Entity(EntitySpec {
                        partition_id: crate::identity::data::PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: crate::symbols::data::ClientKey::raw("meaning-transition"),
                        fields: single_string_aspect_field_patch(
                            aspect_key("name"),
                            field_key("name"),
                            "transition",
                        ),
                    }),
                )),
                execution.allocation_policy(),
            )
            .unwrap();
        let installed = transaction.commit(&execution, &runtime).unwrap();
        let current = runtime
            .read_truth()
            .project_snapshot(&installed.snapshot)
            .unwrap();
        assert!(
            current
                .compare_adjacency_support(
                    &execution,
                    &current,
                    a,
                    KindId(2),
                    ReadDirection::Outgoing
                )
                .unwrap(),
            "the target really has an installed, usable relation and endpoint meaning"
        );
        assert_eq!(
            old.compare_adjacency_support(
                &execution,
                &current,
                a,
                KindId(2),
                ReadDirection::Outgoing
            ),
            Err(Denial::IncompatibleMeaning)
        );
        let target = test_owner_main_basis(&runtime).unwrap();
        assert!(
            matches!(runtime.authored_write_history_cursor(
            execution, &source, selected(&runtime, &target), &[key(a, Direction::Outgoing)]
        ), Err(RelationalAuthoredWriteDenial::IncompleteAdjacencyHistory {
            commit, ordinal, relation: None, kind: None, reason: "unknown-touch-graph"
        }) if commit == installed.commit.commit_id && ordinal > 0),
            "an actual schema-transition publication has unknown touches and must refuse globally"
        );
        for outcome in [&edge, &installed] {
            release_test_commit_snapshot(&runtime, outcome);
        }
    }
}
