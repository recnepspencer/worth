use super::super::support::*;

fn task_edge_runtime(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRuntime {
    stateful_bridge_task_edge_runtime(request_execution)
}

#[test]
fn ordinary_batch_does_not_claim_graph_composition_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_edge_runtime(request_execution)
        .workspace("tasks.graph-composition-ordinary-batch")
        .expect("runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.graph-composition-ordinary-batch-tasks",
            |q| {
                q.from("Task")
                    .select([identity_id_field_key(), title_value_field_key()])
                    .order_by(title_value_field_key())
                    .schema_basis("tasks-graph-composition-ordinary-batch-tasks")
            },
        )
        .expect("task live view should declare");

    let receipt = workspace
        .batch(request_execution, |batch| {
            batch.insert("Task", |task| {
                task.set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("task-ordinary"),
                )
                .set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Ordinary task"),
                )
            })
        })
        .expect("ordinary batch should execute");
    let inspection = workspace
        .inspect(request_execution, &receipt)
        .expect("receipt should inspect");

    assert!(receipt.graph_composition_program().is_none());
    assert!(receipt.graph_composition_lifecycle_outcomes().is_none());
    assert!(receipt.graph_composition_evidence().is_none());
    match inspection {
        WorthQueryInspection::BatchWriteReceipt(inspection) => {
            assert!(inspection.graph_composition_program().is_none());
            assert!(inspection.graph_composition_lifecycle_outcomes().is_none());
            assert!(inspection.graph_composition_evidence().is_none())
        }
        other => panic!("expected batch receipt inspection, got {other:?}"),
    }
}

#[test]
fn reconstructed_graph_composition_receipt_without_breadth_fails_closed() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_edge_runtime(request_execution)
        .workspace("tasks.graph-composition-reconstructed-boundary")
        .expect("runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.graph-composition-reconstructed-boundary-tasks",
            |q| {
                q.from("Task")
                    .select([identity_id_field_key(), title_value_field_key()])
                    .order_by(title_value_field_key())
                    .schema_basis("tasks-graph-composition-reconstructed-boundary-tasks")
            },
        )
        .expect("task live view should declare");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.graph-composition-reconstructed-boundary-edges",
            |q| {
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
                    .schema_basis("tasks-graph-composition-reconstructed-boundary-edges")
            },
        )
        .expect("edge live view should declare");

    let composed = workspace
        .compose_graph(request_execution, |graph| {
            let draft = graph.insert_entity("draft-task", "Task", |task| {
                task.set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("task-reconstructed"),
                )
                .set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Reconstructed task"),
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
    let reconstructed =
        WorthQueryBatchWriteReceipt::from_write_receipts(composed.write_receipts().to_vec())
            .expect("reconstructing from component receipts should succeed");
    let inspection = workspace
        .inspect(request_execution, &reconstructed)
        .expect("reconstructed receipt should inspect");

    assert!(reconstructed.graph_composition_program().is_none());
    assert!(reconstructed
        .graph_composition_lifecycle_outcomes()
        .is_none());
    assert!(reconstructed.graph_composition_evidence().is_none());
    match inspection {
        WorthQueryInspection::BatchWriteReceipt(inspection) => {
            assert!(inspection.graph_composition_program().is_none());
            assert!(inspection.graph_composition_lifecycle_outcomes().is_none());
            assert!(inspection.graph_composition_evidence().is_none())
        }
        other => panic!("expected batch receipt inspection, got {other:?}"),
    }
}
