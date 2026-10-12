use super::super::support::*;

#[test]
fn compose_graph_without_lineage_steps_fails_closed_on_lineage_summary() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_vertex_runtime(request_execution)
        .workspace("topology.graph-composition-no-lineage-summary")
        .expect("workspace should open");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "topology.graph-composition-no-lineage-summary-vertices",
            |q| {
                q.from("Vertex")
                    .select([
                        crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts("kind", "value")
                            .unwrap(),
                    ])
                    .order_by(
                        crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                            .unwrap(),
                    )
                    .schema_basis("topology-graph-composition-no-lineage-summary-vertices")
            },
        )
        .expect("vertex live view should declare");

    let receipt = workspace
        .compose_graph(request_execution, |graph| {
            graph.insert_entity("vertex-only", "Vertex", |vertex| {
                vertex
                    .set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value("vertex-only"),
                    )
                    .set_aspect(
                        test_aspect_touch("kind.value"),
                        test_authored_string_aspect_value("plain"),
                    )
            })?;
            Ok(())
        })
        .expect("simple composition should execute");

    assert!(receipt.graph_composition_lineage_summary().is_none());
    match workspace
        .inspect(request_execution, &receipt)
        .expect("receipt should inspect")
    {
        WorthQueryInspection::BatchWriteReceipt(inspection) => {
            assert!(inspection.graph_composition_lineage_summary().is_none());
        }
        other => panic!("expected batch receipt inspection, got {other:?}"),
    }
}
