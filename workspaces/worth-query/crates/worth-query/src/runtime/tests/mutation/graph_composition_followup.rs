use super::super::support::*;

fn task_edge_runtime(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRuntime {
    stateful_bridge_task_edge_runtime(request_execution)
}

#[test]
fn compose_graph_supports_symbolic_relation_followup_mutation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_edge_runtime(request_execution)
        .workspace("tasks.graph-composition-relation-followup")
        .expect("runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.graph-composition-relation-followup-tasks",
            |q| {
                q.from("Task")
                    .select([
                        crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                            .unwrap(),
                    ])
                    .order_by(
                        crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                            .unwrap(),
                    )
                    .schema_basis("tasks-graph-composition-relation-followup-tasks")
            },
        )
        .expect("task live view should declare");
    let edges: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.graph-composition-relation-followup-edges",
            |q| {
                q.from("TaskEdge")
                    .select([
                        crate::authoring::AspectFieldKey::from_authoring_parts("edge", "kind")
                            .unwrap(),
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
                    .order_by(
                        crate::authoring::AspectFieldKey::from_authoring_parts("edge", "kind")
                            .unwrap(),
                    )
                    .schema_basis("tasks-graph-composition-relation-followup-edges")
            },
        )
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
            let edge = graph.insert_symbolic_relation("draft-edge", "TaskEdge", |relation| {
                relation
                    .set_aspect(
                        test_aspect_touch("edge.kind"),
                        test_authored_string_aspect_value("depends_on"),
                    )
                    .symbolic_entity_identity(test_aspect_touch("edge.source_identity"), &draft)
                    .existing_entity_identity(
                        test_aspect_touch("edge.target_identity"),
                        test_entity_identity("task-existing"),
                    )
            })?;
            graph.update_relation(&edge, |relation| {
                relation.set_aspect(
                    test_aspect_touch("edge.kind"),
                    test_authored_string_aspect_value("blocks"),
                )
            })?;
            Ok(())
        })
        .expect("graph composition with relation followup should execute");
    let edge_rows = workspace.read(request_execution, &edges);
    let inspection = workspace
        .inspect(request_execution, &receipt)
        .expect("receipt should inspect");
    let program = receipt
        .graph_composition_program()
        .expect("graph composition receipt should expose composition program");
    let lifecycle = receipt
        .graph_composition_lifecycle_outcomes()
        .expect("graph composition receipt should expose lifecycle outcomes");
    let composition_evidence = receipt
        .graph_composition_evidence()
        .expect("graph composition receipt should expose composition evidence");

    assert_eq!(receipt.write_receipts().len(), 3);
    assert_eq!(receipt.batch_mutation_evidence().component_count(), 3);
    assert_eq!(receipt.batch_mutation_evidence().resolved_target_count(), 3);
    assert_eq!(receipt.graph_composition_breadth().component_count(), 3);
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
    assert_eq!(program.component_count(), 3);
    assert_eq!(
        program.steps()[0].kind(),
        WorthQueryGraphCompositionProgramStepKind::SymbolicEntityDeclaration
    );
    assert_eq!(
        program.steps()[1].kind(),
        WorthQueryGraphCompositionProgramStepKind::SymbolicRelationDeclaration
    );
    assert_eq!(program.steps()[1].declared_symbol(), Some("draft-edge"));
    assert_eq!(
        program.steps()[2].kind(),
        WorthQueryGraphCompositionProgramStepKind::SymbolicRelationFollowupMutation
    );
    assert_eq!(program.steps()[2].declared_symbol(), Some("draft-edge"));
    assert_eq!(lifecycle.entries().len(), 3);
    assert_eq!(
        lifecycle.entries()[0].outcome_kind(),
        WorthQueryGraphCompositionLifecycleOutcomeKind::Created
    );
    assert_eq!(
        lifecycle.entries()[1].outcome_kind(),
        WorthQueryGraphCompositionLifecycleOutcomeKind::Created
    );
    assert_eq!(
        lifecycle.entries()[2].outcome_kind(),
        WorthQueryGraphCompositionLifecycleOutcomeKind::UpdatedIdentityPreserved
    );
    assert_eq!(
        lifecycle.counter_snapshot(),
        "created=2;updated_identity_preserved=1;retargeted_identity_preserved=0;retired_current_truth=0;superseded_with_lineage=0;deleted_if_uncommitted=0;denied_before_execution=0"
    );
    assert_eq!(composition_evidence.symbolic_resolution_count(), 2);
    assert_eq!(
        composition_evidence.counter_snapshot(),
        "components=3;symbolic_entities=1;symbolic_relations=1;symbolic_resolutions=2;affected_live_views=2;affected_derived_views=0;considered_computed_views=0;created=2;updated_identity_preserved=1;retargeted_identity_preserved=0;retired_current_truth=0;superseded_with_lineage=0;deleted_if_uncommitted=0;denied_before_execution=0"
    );
    assert_eq!(
        composition_evidence.lifecycle_counter_snapshot(),
        "created=2;updated_identity_preserved=1;retargeted_identity_preserved=0;retired_current_truth=0;superseded_with_lineage=0;deleted_if_uncommitted=0;denied_before_execution=0"
    );
    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .symbolic_resolution_count(),
        2
    );
    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .symbolic_target_reference_count(),
        1
    );
    assert_eq!(receipt.graph_composition_resolution_map().len(), 2);
    assert_eq!(edge_rows.len(), 1);
    assert_eq!(
        test_native_string_value(&edge_rows[0], "edge.kind").as_deref(),
        Some("blocks")
    );

    let support = workspace.public_authoritative_mutation_evidence_support();
    assert!(support
        .graph_composition_families()
        .iter()
        .any(|family| family == "same_batch_symbolic_relation_followup_mutation"));
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
            assert_eq!(program.component_count(), 3);
            assert_eq!(
                program.steps()[1].kind(),
                WorthQueryGraphCompositionProgramStepKind::SymbolicRelationDeclaration
            );
            assert_eq!(
                program.steps()[2].kind(),
                WorthQueryGraphCompositionProgramStepKind::SymbolicRelationFollowupMutation
            );
            assert_eq!(
                lifecycle.entries()[2].outcome_kind(),
                WorthQueryGraphCompositionLifecycleOutcomeKind::UpdatedIdentityPreserved
            );
            assert_eq!(inspection.batch_mutation_evidence().component_count(), 3);
            assert_eq!(inspection.graph_composition_breadth().component_count(), 3);
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
                2
            );
            assert_eq!(
                inspection
                    .batch_mutation_evidence()
                    .symbolic_target_reference_count(),
                1
            );
            assert_eq!(inspection.component_operations().len(), 3);
            assert_eq!(inspection.component_operations()[0].family(), "insert");
            assert_eq!(inspection.component_operations()[1].family(), "insert");
            assert_eq!(inspection.component_operations()[2].family(), "update");
            assert_eq!(composition_evidence.symbolic_resolution_count(), 2);
            assert_eq!(
                composition_evidence.counter_snapshot(),
                "components=3;symbolic_entities=1;symbolic_relations=1;symbolic_resolutions=2;affected_live_views=2;affected_derived_views=0;considered_computed_views=0;created=2;updated_identity_preserved=1;retargeted_identity_preserved=0;retired_current_truth=0;superseded_with_lineage=0;deleted_if_uncommitted=0;denied_before_execution=0"
            );
            assert_eq!(
                composition_evidence.lifecycle_counter_snapshot(),
                "created=2;updated_identity_preserved=1;retargeted_identity_preserved=0;retired_current_truth=0;superseded_with_lineage=0;deleted_if_uncommitted=0;denied_before_execution=0"
            );
            assert_eq!(
                inspection.component_operations()[1]
                    .symbolic_aspect_resolution_evidence()
                    .len(),
                1
            );
            assert_eq!(
                inspection.component_operations()[2]
                    .symbolic_target_reference_evidence()
                    .expect("relation followup should retain symbolic relation evidence")
                    .symbol()
                    .as_str(),
                "draft-edge"
            );
            assert_eq!(
                inspection
                    .graph_composition_resolution_map()
                    .entries()
                    .len(),
                2
            );
        }
        other => panic!("expected batch receipt inspection, got {other:?}"),
    }
}
