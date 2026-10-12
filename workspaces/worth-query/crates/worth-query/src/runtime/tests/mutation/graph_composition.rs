use super::super::support::*;

fn task_edge_runtime(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRuntime {
    stateful_bridge_task_edge_runtime(request_execution)
}

#[test]
fn compose_graph_preserves_symbolic_resolution_and_mixed_edge_meaning() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_edge_runtime(request_execution)
        .workspace("tasks.graph-composition")
        .expect("runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.graph-composition-tasks", |q| {
            q.from("Task")
                .select([identity_id_field_key(), title_value_field_key()])
                .order_by(title_value_field_key())
                .schema_basis("tasks-graph-composition-tasks")
        })
        .expect("task live view should declare");
    let edges: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.graph-composition-edges", |q| {
            q.from("TaskEdge")
                .select([
                    edge_kind_field_key(),
                    crate::authoring::AspectFieldKey::from_authoring_parts(
                        "edge",
                        "source_identity",
                    )
                    .unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts(
                        "edge",
                        "target_identity",
                    )
                    .unwrap(),
                ])
                .order_by(edge_kind_field_key())
                .schema_basis("tasks-graph-composition-edges")
        })
        .expect("edge live view should declare");

    let receipt = workspace
        .compose_graph(request_execution, |graph| {
            let draft = graph.insert_entity("draft-task", "Task", |task| {
                task.set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("task-draft"),
                )
                .set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Draft task"),
                )
            })?;
            graph.insert_relation("TaskEdge", |edge| {
                edge.set_aspect(
                    test_aspect_touch("edge.kind"),
                    test_authored_string_aspect_value("depends_on"),
                )
                .symbolic_entity_identity(test_aspect_touch("edge.source_identity"), &draft)
                .existing_entity_identity(
                    test_aspect_touch("edge.target_identity"),
                    test_entity_identity("task-existing"),
                )
            })?;
            Ok(())
        })
        .expect("graph composition should execute");
    let draft_identity = receipt.write_receipts()[0].deltas()[0]
        .entity_identity
        .clone();
    let edge_rows = workspace.read(request_execution, &edges);
    let inspection = workspace
        .inspect(request_execution, &receipt)
        .expect("receipt should inspect");

    assert_eq!(receipt.write_receipts().len(), 2);
    assert_eq!(receipt.batch_mutation_evidence().component_count(), 2);
    assert_eq!(receipt.batch_mutation_evidence().resolved_target_count(), 2);
    assert_eq!(receipt.graph_composition_breadth().component_count(), 2);
    assert_eq!(
        receipt
            .graph_composition_breadth()
            .symbolic_entity_declaration_count(),
        1
    );
    assert_eq!(
        receipt
            .graph_composition_breadth()
            .symbolic_relation_declaration_count(),
        1
    );
    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .symbolic_resolution_count(),
        1
    );
    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .symbolic_target_reference_count(),
        0
    );
    let program = receipt
        .graph_composition_program()
        .expect("graph composition receipt should expose composition program");
    let lifecycle = receipt
        .graph_composition_lifecycle_outcomes()
        .expect("graph composition receipt should expose lifecycle outcomes");
    let resolution_map = receipt.graph_composition_resolution_map();
    let composition_evidence = receipt
        .graph_composition_evidence()
        .expect("graph composition receipt should expose composition evidence");
    assert_eq!(program.component_count(), 2);
    assert_eq!(
        program.steps()[0].kind(),
        WorthQueryGraphCompositionProgramStepKind::SymbolicEntityDeclaration
    );
    assert_eq!(program.steps()[0].declared_symbol(), Some("draft-task"));
    assert_eq!(program.steps()[0].declared_collection(), "Task");
    assert_eq!(
        program.steps()[1].kind(),
        WorthQueryGraphCompositionProgramStepKind::RelationDeclaration
    );
    assert_eq!(program.steps()[1].declared_symbol(), None);
    assert_eq!(program.steps()[1].declared_collection(), "TaskEdge");
    assert_eq!(lifecycle.entries().len(), 2);
    assert_eq!(
        lifecycle.entries()[0].outcome_kind(),
        WorthQueryGraphCompositionLifecycleOutcomeKind::Created
    );
    assert_eq!(
        lifecycle.entries()[1].outcome_kind(),
        WorthQueryGraphCompositionLifecycleOutcomeKind::Created
    );
    assert_eq!(
        lifecycle.counter_snapshot(),
        "created=2;updated_identity_preserved=0;retargeted_identity_preserved=0;retired_current_truth=0;superseded_with_lineage=0;deleted_if_uncommitted=0;denied_before_execution=0"
    );
    assert_eq!(resolution_map.len(), 1);
    assert_eq!(resolution_map.entries()[0].component_index(), 1);
    assert_eq!(
        resolution_map.entries()[0].aspect_touch(),
        Some(&test_aspect_touch("edge.source_identity"))
    );
    assert_eq!(resolution_map.entries()[0].symbol().as_str(), "draft-task");
    assert_eq!(
        resolution_map.entries()[0].resolved_entity_identity(),
        &draft_identity
    );
    assert_eq!(composition_evidence.symbolic_resolution_count(), 1);
    assert_eq!(composition_evidence.affected_live_view_count(), 2);
    assert_eq!(composition_evidence.affected_derived_view_count(), 0);
    assert_eq!(composition_evidence.considered_computed_view_count(), 0);
    assert!(composition_evidence
        .counter_snapshot()
        .contains("symbolic_entities=1"));
    assert_eq!(
        composition_evidence.lifecycle_counter_snapshot(),
        "created=2;updated_identity_preserved=0;retargeted_identity_preserved=0;retired_current_truth=0;superseded_with_lineage=0;deleted_if_uncommitted=0;denied_before_execution=0"
    );
    assert!(
        !composition_evidence.graph_composition_digest().is_empty()
            && !composition_evidence
                .graph_symbolic_resolution_digest()
                .is_empty()
    );
    assert_eq!(edge_rows.len(), 1);
    let expected_source = test_native_entity_ref_value(&draft_identity);
    let expected_target = test_native_entity_ref_value(&test_entity_identity("task-existing"));
    assert_eq!(
        test_native_scalar_value(&edge_rows[0], "edge.source_identity"),
        Some(&expected_source)
    );
    assert_eq!(
        test_native_scalar_value(&edge_rows[0], "edge.target_identity"),
        Some(&expected_target)
    );

    let support = workspace.public_authoritative_mutation_evidence_support();
    assert!(support
        .symbolic_aspect_reference_families()
        .iter()
        .any(|family| family == "same_batch_declared_entity_identity"));
    assert!(support
        .graph_composition_families()
        .iter()
        .any(|family| family == "mixed_existing_and_symbolic_entity_identity_edges"));
    match inspection {
        WorthQueryInspection::BatchWriteReceipt(inspection) => {
            let program = inspection
                .graph_composition_program()
                .expect("inspection should expose composition program");
            let lifecycle = inspection
                .graph_composition_lifecycle_outcomes()
                .expect("inspection should expose lifecycle outcomes");
            let composition_evidence = inspection
                .graph_composition_evidence()
                .expect("inspection should expose composition evidence");
            assert_eq!(program.component_count(), 2);
            assert_eq!(
                program.steps()[0].kind(),
                WorthQueryGraphCompositionProgramStepKind::SymbolicEntityDeclaration
            );
            assert_eq!(program.steps()[0].declared_symbol(), Some("draft-task"));
            assert_eq!(
                program.steps()[1].kind(),
                WorthQueryGraphCompositionProgramStepKind::RelationDeclaration
            );
            assert_eq!(
                lifecycle.entries()[0].outcome_kind(),
                WorthQueryGraphCompositionLifecycleOutcomeKind::Created
            );
            assert_eq!(
                lifecycle.entries()[1].outcome_kind(),
                WorthQueryGraphCompositionLifecycleOutcomeKind::Created
            );
            assert_eq!(inspection.batch_mutation_evidence().component_count(), 2);
            assert_eq!(inspection.graph_composition_breadth().component_count(), 2);
            assert_eq!(
                inspection
                    .graph_composition_breadth()
                    .symbolic_entity_declaration_count(),
                1
            );
            assert_eq!(
                inspection
                    .graph_composition_breadth()
                    .symbolic_relation_declaration_count(),
                1
            );
            assert_eq!(
                inspection
                    .batch_mutation_evidence()
                    .symbolic_resolution_count(),
                1
            );
            assert_eq!(
                inspection
                    .batch_mutation_evidence()
                    .symbolic_target_reference_count(),
                0
            );
            assert_eq!(inspection.component_operations().len(), 2);
            assert_eq!(inspection.component_operations()[0].family(), "insert");
            assert_eq!(inspection.component_operations()[1].family(), "insert");
            assert_eq!(composition_evidence.symbolic_resolution_count(), 1);
            assert!(composition_evidence
                .counter_snapshot()
                .contains("affected_live_views=2"));
            assert_eq!(
                composition_evidence.lifecycle_counter_snapshot(),
                "created=2;updated_identity_preserved=0;retargeted_identity_preserved=0;retired_current_truth=0;superseded_with_lineage=0;deleted_if_uncommitted=0;denied_before_execution=0"
            );
            assert_eq!(
                inspection
                    .graph_composition_resolution_map()
                    .entries()
                    .len(),
                1
            );
            assert_eq!(
                inspection.component_operations()[1].declared_collection(),
                Some("TaskEdge")
            );
            assert_eq!(
                inspection.component_operations()[1].target_collection(),
                Some("TaskEdge")
            );
            assert_eq!(
                inspection.component_operations()[1].admitted_touched_aspects(),
                test_aspect_touches(["edge", "edge.source_identity"]).as_slice()
            );
            assert_eq!(
                inspection.component_operations()[1]
                    .symbolic_aspect_resolution_evidence()
                    .len(),
                1
            );
            assert_eq!(
                inspection.component_operations()[1].symbolic_aspect_resolution_evidence()[0]
                    .aspect_touch(),
                &test_aspect_touch("edge.source_identity")
            );
            assert!(inspection.component_operations()[1]
                .existing_truth_binding_evidence()
                .is_none());
        }
        other => panic!("expected batch receipt inspection, got {other:?}"),
    }
}
