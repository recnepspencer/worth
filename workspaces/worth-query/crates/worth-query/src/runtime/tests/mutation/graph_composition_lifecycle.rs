use super::super::support::*;

fn task_edge_runtime(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRuntime {
    stateful_bridge_task_edge_runtime(request_execution)
}

#[test]
fn compose_graph_supports_symbolic_entity_followup_and_relation_retirement() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_edge_runtime(request_execution)
        .workspace("tasks.graph-composition-lifecycle")
        .expect("runtime should open a named workspace");
    let tasks: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.graph-composition-lifecycle-tasks",
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
                    .schema_basis("tasks-graph-composition-lifecycle-tasks")
            },
        )
        .expect("task live view should declare");
    let edges: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.graph-composition-lifecycle-edges",
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
                    .schema_basis("tasks-graph-composition-lifecycle-edges")
            },
        )
        .expect("edge live view should declare");

    let receipt = workspace
        .compose_graph(request_execution, |graph| {
            let draft = graph.insert_entity("draft-task", "Task", |task| {
                task.set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("task-lifecycle"),
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
            graph.update_entity(&draft, |task| {
                task.set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Published task"),
                )
            })?;
            graph.delete_relation(&edge, |delete| {
                delete.touches(test_aspect_touches([
                    "edge.kind",
                    "edge.source_identity",
                    "edge.target_identity",
                ]))
            })?;
            Ok(())
        })
        .expect("graph composition lifecycle should execute");
    let task_rows = workspace.read(request_execution, &tasks);
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

    assert_eq!(receipt.write_receipts().len(), 4);
    assert_eq!(receipt.batch_mutation_evidence().component_count(), 4);
    assert_eq!(receipt.batch_mutation_evidence().resolved_target_count(), 4);
    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .symbolic_resolution_count(),
        3
    );
    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .symbolic_target_reference_count(),
        2
    );
    assert_eq!(receipt.graph_composition_breadth().component_count(), 4);
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
    assert_eq!(program.component_count(), 4);
    assert_eq!(
        program.steps()[2].kind(),
        WorthQueryGraphCompositionProgramStepKind::SymbolicEntityFollowupMutation
    );
    assert_eq!(program.steps()[2].declared_symbol(), Some("draft-task"));
    assert_eq!(
        program.steps()[3].kind(),
        WorthQueryGraphCompositionProgramStepKind::SymbolicRelationRetirement
    );
    assert_eq!(program.steps()[3].declared_symbol(), Some("draft-edge"));
    assert_eq!(lifecycle.entries().len(), 4);
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
        lifecycle.entries()[3].outcome_kind(),
        WorthQueryGraphCompositionLifecycleOutcomeKind::RetiredCurrentTruth
    );
    assert_eq!(
        lifecycle.counter_snapshot(),
        "created=2;updated_identity_preserved=1;retargeted_identity_preserved=0;retired_current_truth=1;superseded_with_lineage=0;deleted_if_uncommitted=0;denied_before_execution=0"
    );
    assert_eq!(receipt.graph_composition_resolution_map().len(), 3);
    assert_eq!(composition_evidence.symbolic_resolution_count(), 3);
    assert_eq!(composition_evidence.affected_live_view_count(), 2);
    assert_eq!(composition_evidence.affected_derived_view_count(), 0);
    assert_eq!(composition_evidence.considered_computed_view_count(), 0);
    assert_eq!(
        composition_evidence.counter_snapshot(),
        "components=4;symbolic_entities=1;symbolic_relations=1;symbolic_resolutions=3;affected_live_views=2;affected_derived_views=0;considered_computed_views=0;created=2;updated_identity_preserved=1;retargeted_identity_preserved=0;retired_current_truth=1;superseded_with_lineage=0;deleted_if_uncommitted=0;denied_before_execution=0"
    );
    assert_eq!(
        composition_evidence.lifecycle_counter_snapshot(),
        "created=2;updated_identity_preserved=1;retargeted_identity_preserved=0;retired_current_truth=1;superseded_with_lineage=0;deleted_if_uncommitted=0;denied_before_execution=0"
    );
    assert_eq!(task_rows.len(), 1);
    assert_eq!(edge_rows.len(), 0);
    assert_eq!(
        test_native_string_value(&task_rows[0], "title.value").as_deref(),
        Some("Published task")
    );

    let support = workspace.public_authoritative_mutation_evidence_support();
    assert!(support
        .graph_composition_families()
        .iter()
        .any(|family| family == "same_batch_symbolic_entity_followup_mutation"));
    assert!(support
        .graph_composition_families()
        .iter()
        .any(|family| family == "same_batch_symbolic_relation_retirement"));

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
            assert_eq!(
                program.steps()[2].kind(),
                WorthQueryGraphCompositionProgramStepKind::SymbolicEntityFollowupMutation
            );
            assert_eq!(
                program.steps()[3].kind(),
                WorthQueryGraphCompositionProgramStepKind::SymbolicRelationRetirement
            );
            assert_eq!(
                lifecycle.entries()[2].outcome_kind(),
                WorthQueryGraphCompositionLifecycleOutcomeKind::UpdatedIdentityPreserved
            );
            assert_eq!(
                lifecycle.entries()[3].outcome_kind(),
                WorthQueryGraphCompositionLifecycleOutcomeKind::RetiredCurrentTruth
            );
            assert_eq!(inspection.component_operations().len(), 4);
            assert_eq!(inspection.batch_mutation_evidence().component_count(), 4);
            assert_eq!(
                inspection.batch_mutation_evidence().resolved_target_count(),
                4
            );
            assert_eq!(
                inspection
                    .batch_mutation_evidence()
                    .symbolic_resolution_count(),
                3
            );
            assert_eq!(
                inspection
                    .batch_mutation_evidence()
                    .symbolic_target_reference_count(),
                2
            );
            assert_eq!(inspection.graph_composition_breadth().component_count(), 4);
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
            assert_eq!(inspection.component_operations()[2].family(), "update");
            assert_eq!(inspection.component_operations()[3].family(), "delete");
            assert_eq!(inspection.graph_composition_resolution_map().len(), 3);
            assert_eq!(composition_evidence.symbolic_resolution_count(), 3);
            assert_eq!(composition_evidence.affected_live_view_count(), 2);
            assert_eq!(composition_evidence.affected_derived_view_count(), 0);
            assert_eq!(composition_evidence.considered_computed_view_count(), 0);
            assert_eq!(
                composition_evidence.counter_snapshot(),
                "components=4;symbolic_entities=1;symbolic_relations=1;symbolic_resolutions=3;affected_live_views=2;affected_derived_views=0;considered_computed_views=0;created=2;updated_identity_preserved=1;retargeted_identity_preserved=0;retired_current_truth=1;superseded_with_lineage=0;deleted_if_uncommitted=0;denied_before_execution=0"
            );
            assert_eq!(
                composition_evidence.lifecycle_counter_snapshot(),
                "created=2;updated_identity_preserved=1;retargeted_identity_preserved=0;retired_current_truth=1;superseded_with_lineage=0;deleted_if_uncommitted=0;denied_before_execution=0"
            );
            assert_eq!(
                inspection.component_operations()[2]
                    .symbolic_target_reference_evidence()
                    .expect("symbolic entity update should retain target evidence")
                    .symbol()
                    .as_str(),
                "draft-task"
            );
            assert_eq!(
                inspection.component_operations()[3]
                    .symbolic_target_reference_evidence()
                    .expect("symbolic relation delete should retain target evidence")
                    .symbol()
                    .as_str(),
                "draft-edge"
            );
        }
        other => panic!("expected batch receipt inspection, got {other:?}"),
    }
}
