use super::super::support::*;

fn task_relation_runtime(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRuntime {
    stateful_bridge_task_relation_runtime(request_execution)
}

#[test]
fn compose_graph_supports_mixed_symbolic_create_and_existing_target_lifecycle() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_relation_runtime(request_execution)
        .workspace("tasks.graph-composition-mixed-existing")
        .expect("runtime should open a named workspace");
    let tasks: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.graph-composition-mixed-existing-tasks",
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
                    .schema_basis("tasks-graph-composition-mixed-existing-tasks")
            },
        )
        .expect("task live view should declare");
    let relations: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.graph-composition-mixed-existing-relations",
            |q| {
                q.from("TaskRelation")
                    .select([
                        crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts("kind", "value")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts("status", "value")
                            .unwrap(),
                    ])
                    .order_by(
                        crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                            .unwrap(),
                    )
                    .schema_basis("tasks-graph-composition-mixed-existing-relations")
            },
        )
        .expect("relation live view should declare");

    let update_seed = workspace
        .insert(request_execution, "TaskRelation", |relation| {
            relation
                .set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("rel-update"),
                )
                .set_aspect(
                    test_aspect_touch("kind.value"),
                    test_authored_string_aspect_value("depends_on"),
                )
                .set_aspect(
                    test_aspect_touch("status.value"),
                    test_authored_string_aspect_value("open"),
                )
        })
        .expect("update seed should execute");
    let delete_seed = workspace
        .insert(request_execution, "TaskRelation", |relation| {
            relation
                .set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("rel-delete"),
                )
                .set_aspect(
                    test_aspect_touch("kind.value"),
                    test_authored_string_aspect_value("blocks"),
                )
                .set_aspect(
                    test_aspect_touch("status.value"),
                    test_authored_string_aspect_value("stale"),
                )
        })
        .expect("delete seed should execute");
    let update_binding = workspace
        .bind_existing_relation(
            WorthQueryExistingRelationTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:rel-update").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                update_seed.deltas()[0].entity_identity.clone(),
            )
            .expect("update relation target should build")
            .in_target_collection("TaskRelation")
            .expect("update relation collection should build"),
        )
        .expect("update binding should build");
    let delete_binding = workspace
        .bind_existing_relation(
            WorthQueryExistingRelationTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:rel-update").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                delete_seed.deltas()[0].entity_identity.clone(),
            )
            .expect("delete relation target should build")
            .in_target_collection("TaskRelation")
            .expect("delete relation collection should build"),
        )
        .expect("delete binding should build");

    let receipt = workspace
        .compose_graph(request_execution, |graph| {
            let _ = graph.insert_entity("draft-task", "Task", |task| {
                task.set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("task-mixed-existing"),
                )
                .set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Mixed existing task"),
                )
            })?;
            graph.update_existing(update_binding, |relation| {
                relation.set_aspect(
                    test_aspect_touch("status.value"),
                    test_authored_string_aspect_value("closed"),
                )
            })?;
            graph.delete_existing(delete_binding, |delete| {
                delete.touches(test_aspect_touches(["kind.value", "status.value"]))
            })?;
            Ok(())
        })
        .expect("mixed symbolic and existing lifecycle program should execute");

    let task_rows = workspace.read(request_execution, &tasks);
    let relation_rows = workspace.read(request_execution, &relations);
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
        0
    );
    assert_eq!(program.component_count(), 3);
    assert_eq!(
        program.steps()[1].kind(),
        WorthQueryGraphCompositionProgramStepKind::ExistingTargetFollowupMutation
    );
    assert_eq!(program.steps()[1].declared_collection(), "TaskRelation");
    assert_eq!(
        program.steps()[2].kind(),
        WorthQueryGraphCompositionProgramStepKind::ExistingTargetRetirement
    );
    assert_eq!(program.steps()[2].declared_collection(), "TaskRelation");
    assert_eq!(lifecycle.entries().len(), 3);
    assert_eq!(
        lifecycle.entries()[0].outcome_kind(),
        WorthQueryGraphCompositionLifecycleOutcomeKind::Created
    );
    assert_eq!(
        lifecycle.entries()[1].outcome_kind(),
        WorthQueryGraphCompositionLifecycleOutcomeKind::UpdatedIdentityPreserved
    );
    assert_eq!(
        lifecycle.entries()[2].outcome_kind(),
        WorthQueryGraphCompositionLifecycleOutcomeKind::RetiredCurrentTruth
    );
    assert_eq!(
        lifecycle.counter_snapshot(),
        "created=1;updated_identity_preserved=1;retargeted_identity_preserved=0;retired_current_truth=1;superseded_with_lineage=0;deleted_if_uncommitted=0;denied_before_execution=0"
    );
    assert_eq!(composition_evidence.symbolic_resolution_count(), 0);
    assert_eq!(
        composition_evidence.counter_snapshot(),
        "components=3;symbolic_entities=1;symbolic_relations=0;symbolic_resolutions=0;affected_live_views=2;affected_derived_views=0;considered_computed_views=0;created=1;updated_identity_preserved=1;retargeted_identity_preserved=0;retired_current_truth=1;superseded_with_lineage=0;deleted_if_uncommitted=0;denied_before_execution=0"
    );
    assert_eq!(
        composition_evidence.lifecycle_counter_snapshot(),
        "created=1;updated_identity_preserved=1;retargeted_identity_preserved=0;retired_current_truth=1;superseded_with_lineage=0;deleted_if_uncommitted=0;denied_before_execution=0"
    );
    assert_eq!(task_rows.len(), 1);
    assert_eq!(relation_rows.len(), 1);
    assert_eq!(
        test_native_string_value(&relation_rows[0], "identity.id").as_deref(),
        Some("rel-update")
    );
    assert_eq!(
        test_native_string_value(&relation_rows[0], "status.value").as_deref(),
        Some("closed")
    );

    let support = workspace.public_authoritative_mutation_evidence_support();
    assert!(support
        .graph_composition_families()
        .iter()
        .any(|family| family == "mixed_existing_target_followup_mutation"));
    assert!(support
        .graph_composition_families()
        .iter()
        .any(|family| family == "mixed_existing_target_retirement"));

    match inspection {
        WorthQueryInspection::BatchWriteReceipt(inspection) => {
            let program = inspection
                .graph_composition_program()
                .expect("inspection should expose composition program");
            let lifecycle = inspection
                .graph_composition_lifecycle_outcomes()
                .expect("inspection should expose lifecycle outcomes");
            assert_eq!(
                program.steps()[1].kind(),
                WorthQueryGraphCompositionProgramStepKind::ExistingTargetFollowupMutation
            );
            assert_eq!(
                program.steps()[2].kind(),
                WorthQueryGraphCompositionProgramStepKind::ExistingTargetRetirement
            );
            assert_eq!(
                lifecycle.entries()[1].outcome_kind(),
                WorthQueryGraphCompositionLifecycleOutcomeKind::UpdatedIdentityPreserved
            );
            assert_eq!(
                lifecycle.entries()[2].outcome_kind(),
                WorthQueryGraphCompositionLifecycleOutcomeKind::RetiredCurrentTruth
            );
            assert_eq!(inspection.component_operations()[1].family(), "update");
            assert_eq!(inspection.component_operations()[2].family(), "delete");
            assert_eq!(
                inspection.component_operations()[1]
                    .existing_truth_binding_evidence()
                    .expect("existing-target update should retain binding evidence")
                    .family(),
                WorthQueryExistingTruthBindingFamily::DirectRelationIdentity
            );
            assert_eq!(
                inspection.component_operations()[2]
                    .existing_truth_binding_evidence()
                    .expect("existing-target delete should retain binding evidence")
                    .family(),
                WorthQueryExistingTruthBindingFamily::DirectRelationIdentity
            );
        }
        other => panic!("expected batch receipt inspection, got {other:?}"),
    }
}
