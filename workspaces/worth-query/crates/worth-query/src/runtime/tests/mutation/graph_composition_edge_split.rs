use super::super::support::*;

fn edge_split_verified_profile() -> WorthQueryRuntimeSupportProfile {
    bridge_verified_direct_relation_profile("update_existing_verified")
}

fn edge_split_runtime(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    binding: &WorthQueryExistingTruthTargetBinding,
) -> WorthQueryRuntime {
    bridge_runtime_with_support_and_existing_truth_verification(
        request_execution,
        edge_split_verified_profile(),
        TestExistingTruthVerificationAdapter::default()
            .with_value(
                binding,
                "source.id",
                test_native_entity_ref_value(&test_entity_identity("vertex-a")),
            )
            .with_value(
                binding,
                "target.id",
                test_native_entity_ref_value(&test_entity_identity("vertex-b")),
            ),
    )
}

fn declare_edge_split_live_views(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
) {
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "topology.edge-split-vertices", |q| {
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
                .schema_basis("topology-edge-split-vertices")
        })
        .expect("vertex live view should declare");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "topology.edge-split-edges", |q| {
            q.from("Edge")
                .select([
                    crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                        .unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts("kind", "value")
                        .unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts("source", "id").unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts("target", "id").unwrap(),
                ])
                .order_by(
                    crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                        .unwrap(),
                )
                .schema_basis("topology-edge-split-edges")
        })
        .expect("edge live view should declare");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "topology.edge-split-adjacencies", |q| {
            q.from("VertexEdgeAdjacency")
                .select([
                    crate::authoring::AspectFieldKey::from_authoring_parts("vertex", "id").unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts("edge", "id").unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts("role", "value")
                        .unwrap(),
                ])
                .order_by(
                    crate::authoring::AspectFieldKey::from_authoring_parts("role", "value")
                        .unwrap(),
                )
                .schema_basis("topology-edge-split-adjacencies")
        })
        .expect("adjacency live view should declare");
}

fn compose_verified_edge_split(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
    binding: WorthQueryExistingTruthTargetBinding,
) -> WorthQueryBatchWriteReceipt {
    workspace
        .compose_graph(request_execution, |graph| {
            let split_vertex = graph.insert_entity("split-vertex", "Vertex", |vertex| {
                vertex
                    .set_aspect(test_aspect_touch("identity.id"), test_authored_string_aspect_value("vertex-split"))
                    .set_aspect(test_aspect_touch("kind.value"), test_authored_string_aspect_value("split"))
            })?;
            let left_edge = graph.insert_entity("edge-left", "Edge", |edge| {
                edge.set_aspect(test_aspect_touch("identity.id"), test_authored_string_aspect_value("edge-left"))
                    .set_aspect(test_aspect_touch("kind.value"), test_authored_string_aspect_value("edge"))
                    .existing_entity_identity(test_aspect_touch("source.id"), test_entity_identity("vertex-a"))
                    .symbolic_entity_identity(test_aspect_touch("target.id"), split_vertex.reference().clone())
            })?;
            let right_edge = graph.insert_entity("edge-right", "Edge", |edge| {
                edge.set_aspect(test_aspect_touch("identity.id"), test_authored_string_aspect_value("edge-right"))
                    .set_aspect(test_aspect_touch("kind.value"), test_authored_string_aspect_value("edge"))
                    .symbolic_entity_identity(test_aspect_touch("source.id"), split_vertex.reference().clone())
                    .existing_entity_identity(test_aspect_touch("target.id"), test_entity_identity("vertex-b"))
            })?;
            graph.insert_relation("VertexEdgeAdjacency", |relation| {
                relation
                    .existing_entity_identity(test_aspect_touch("vertex.id"), test_entity_identity("vertex-a"))
                    .symbolic_entity_identity(test_aspect_touch("edge.id"), &left_edge)
                    .set_aspect(test_aspect_touch("role.value"), test_authored_string_aspect_value("source"))
            })?;
            graph.insert_relation("VertexEdgeAdjacency", |relation| {
                relation
                    .symbolic_entity_identity(test_aspect_touch("vertex.id"), &split_vertex)
                    .symbolic_entity_identity(test_aspect_touch("edge.id"), &left_edge)
                    .set_aspect(test_aspect_touch("role.value"), test_authored_string_aspect_value("split-left"))
            })?;
            graph.insert_relation("VertexEdgeAdjacency", |relation| {
                relation
                    .symbolic_entity_identity(test_aspect_touch("vertex.id"), &split_vertex)
                    .symbolic_entity_identity(test_aspect_touch("edge.id"), &right_edge)
                    .set_aspect(test_aspect_touch("role.value"), test_authored_string_aspect_value("split-right"))
            })?;
            graph.insert_relation("VertexEdgeAdjacency", |relation| {
                relation
                    .existing_entity_identity(test_aspect_touch("vertex.id"), test_entity_identity("vertex-b"))
                    .symbolic_entity_identity(test_aspect_touch("edge.id"), &right_edge)
                    .set_aspect(test_aspect_touch("role.value"), test_authored_string_aspect_value("target"))
            })?;
            graph.supersede_existing_verified(
                binding,
                |verify| {
                    verify
                        .existing_entity_identity(test_aspect_touch("source.id"), test_entity_identity("vertex-a"))
                        .existing_entity_identity(test_aspect_touch("target.id"), test_entity_identity("vertex-b"))
                },
                |edge| {
                    edge.continuity_split_successors(
                        crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_prior_authority(crate::runtime::WorthQueryContinuityPriorAuthorityLabel::new(
                            "authority:edge-main",
                        )
                        .expect("continuity prior authority label")).expect("continuity prior authority identity"),
                        [
                            crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(crate::runtime::WorthQueryContinuitySuccessorAuthorityLabel::new(
                                "authority:edge-left",
                            )
                            .expect("continuity successor authority label")).expect("continuity successor authority identity"),
                            crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(crate::runtime::WorthQueryContinuitySuccessorAuthorityLabel::new(
                                "authority:edge-right",
                            )
                            .expect("continuity successor authority label")).expect("continuity successor authority identity"),
                        ],
                    )
                    .set_aspect(test_aspect_touch("kind.value"), test_authored_string_aspect_value("split-parent"))
                },
            )?;
            Ok(())
        })
        .expect("verified edge split should execute")
}

mod edge_split;
