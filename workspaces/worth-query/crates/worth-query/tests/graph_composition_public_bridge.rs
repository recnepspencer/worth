use crate::support;
use worth_foundational::facade::{CanonicalFieldPath, FieldKey};
use worth_query::facade::runtime::{
    WorthQueryGraphCompositionLifecycleOutcomeKind, WorthQueryGraphCompositionProgramStepKind,
    WorthQueryInspection, WorthQueryLiveView, WorthQueryUnrefinedLiveShape,
};

use support::aspect_touch as touch;
use support::public_bridge_runtime::PublicBridgeRuntimeHarness;
use support::test_entity_identities::relational_test_entity_identity;

#[test]
fn graph_composition_public_bridge_executes_symbolic_followup_and_relation_retirement() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("public.graph-composition-lifecycle")
        .expect("runtime should open a named workspace");
    let tasks: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "public.graph-composition-lifecycle-tasks",
            |q| {
                q.from("Task")
                    .select([
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "identity", "id",
                        )
                        .unwrap(),
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "title", "value",
                        )
                        .unwrap(),
                    ])
                    .order_by(
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "title", "value",
                        )
                        .unwrap(),
                    )
                    .schema_basis("public-graph-composition-lifecycle-tasks")
            },
        )
        .expect("task live view should declare");
    let edges: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "public.graph-composition-lifecycle-edges",
            |q| {
                q.from("TaskEdge")
                    .select([
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "edge", "kind",
                        )
                        .unwrap(),
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "edge",
                            "source_identity",
                        )
                        .unwrap(),
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "edge",
                            "target_identity",
                        )
                        .unwrap(),
                    ])
                    .order_by(
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "edge", "kind",
                        )
                        .unwrap(),
                    )
                    .schema_basis("public-graph-composition-lifecycle-edges")
            },
        )
        .expect("edge live view should declare");

    let receipt = workspace
        .compose_graph(request_execution, |graph| {
            let draft = graph.insert_entity("draft-task", "Task", |task| {
                task.set_aspect(touch("identity.id"), authored_text("task-lifecycle"))
                    .set_aspect(touch("title.value"), authored_text("Draft task"))
            })?;
            let edge = graph.insert_symbolic_relation("draft-edge", "TaskEdge", |relation| {
                relation
                    .set_aspect(touch("edge.kind"), authored_text("depends_on"))
                    .symbolic_entity_identity(touch("edge.source_identity"), &draft)
                    .existing_entity_identity(
                        touch("edge.target_identity"),
                        relational_test_entity_identity("task-existing"),
                    )
            })?;
            graph.update_entity(&draft, |task| {
                task.set_aspect(touch("title.value"), authored_text("Published task"))
            })?;
            graph.delete_relation(&edge, |delete| {
                delete.touches([
                    touch("edge.kind"),
                    touch("edge.source_identity"),
                    touch("edge.target_identity"),
                ])
            })?;
            Ok(())
        })
        .expect("graph composition lifecycle should execute");
    let task_rows = workspace.read(request_execution, &tasks);
    let edge_rows = workspace.read(request_execution, &edges);
    let inspection = workspace
        .inspections()
        .expect("inspection lane should admit")
        .inspect(request_execution, &receipt)
        .expect("receipt should inspect");

    assert_eq!(receipt.write_receipts().len(), 4);
    assert_eq!(
        receipt
            .graph_composition_program()
            .expect("graph composition receipt should expose composition program")
            .steps()[2]
            .kind(),
        WorthQueryGraphCompositionProgramStepKind::SymbolicEntityFollowupMutation
    );
    assert_eq!(
        receipt
            .graph_composition_program()
            .expect("graph composition receipt should expose composition program")
            .steps()[3]
            .kind(),
        WorthQueryGraphCompositionProgramStepKind::SymbolicRelationRetirement
    );
    assert_eq!(
        receipt
            .graph_composition_lifecycle_outcomes()
            .expect("graph composition receipt should expose lifecycle outcomes")
            .entries()[2]
            .outcome_kind(),
        WorthQueryGraphCompositionLifecycleOutcomeKind::UpdatedIdentityPreserved
    );
    assert_eq!(
        receipt
            .graph_composition_lifecycle_outcomes()
            .expect("graph composition receipt should expose lifecycle outcomes")
            .entries()[3]
            .outcome_kind(),
        WorthQueryGraphCompositionLifecycleOutcomeKind::RetiredCurrentTruth
    );
    assert_eq!(receipt.graph_composition_resolution_map().len(), 3);
    assert_eq!(task_rows.len(), 1);
    assert_eq!(edge_rows.len(), 0);
    assert_eq!(
        task_rows[0].scalar_value_at(&field_path("title.value")),
        Some(&text("Published task"))
    );

    match inspection {
        WorthQueryInspection::BatchWriteReceipt(inspection) => {
            assert_eq!(inspection.component_operations().len(), 4);
            assert_eq!(
                inspection
                    .graph_composition_evidence()
                    .expect("inspection should expose graph composition evidence")
                    .affected_live_view_count(),
                2
            );
            assert_eq!(
                inspection
                    .graph_composition_resolution_map()
                    .entries()
                    .len(),
                3
            );
        }
        other => panic!("expected batch receipt inspection, got {other:?}"),
    }
}

fn authored_text(
    value: impl Into<String>,
) -> worth_query::facade::runtime::WorthQueryAuthoredAspectValue {
    worth_query::facade::runtime::WorthQueryAuthoredAspectValue::string(value)
}

fn text(value: impl Into<String>) -> worth_foundational::facade::AspectValue {
    worth_foundational::facade::AspectValue::String(value.into().into())
}

fn field_path(path: &str) -> CanonicalFieldPath {
    CanonicalFieldPath::new(
        path.split('.').map(|segment| {
            FieldKey::new(segment).expect("test field path segment should be valid")
        }),
    )
    .expect("test field path should be non-empty")
}
