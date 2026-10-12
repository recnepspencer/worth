use super::super::support::*;

fn face_loop_runtime(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRuntime {
    stateful_bridge_runtime_with_collections(
        request_execution,
        &[
            "Loop",
            "HalfEdge",
            "FaceLoopRelation",
            "LoopHalfEdgeRelation",
        ],
    )
}

#[test]
fn compose_graph_supports_face_inner_loop_insertion_with_full_resolution_map() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = face_loop_runtime(request_execution)
        .workspace("topology.graph-composition-face-inner-loop")
        .expect("workspace should open");
    let loops: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "topology.face-inner-loop-loops", |q| {
            q.from("Loop")
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
                .schema_basis("topology-face-inner-loop-loops")
        })
        .expect("loop live view should declare");
    let half_edges: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "topology.face-inner-loop-half-edges",
            |q| {
                q.from("HalfEdge")
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
                    .schema_basis("topology-face-inner-loop-half-edges")
            },
        )
        .expect("half-edge live view should declare");
    let face_loops: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "topology.face-inner-loop-face-loops",
            |q| {
                q.from("FaceLoopRelation")
                    .select([
                        crate::authoring::AspectFieldKey::from_authoring_parts("face", "id")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts("loop", "id")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts("role", "value")
                            .unwrap(),
                    ])
                    .order_by(
                        crate::authoring::AspectFieldKey::from_authoring_parts("face", "id")
                            .unwrap(),
                    )
                    .schema_basis("topology-face-inner-loop-face-loops")
            },
        )
        .expect("face-loop live view should declare");
    let loop_edges: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "topology.face-inner-loop-loop-edges",
            |q| {
                q.from("LoopHalfEdgeRelation")
                    .select([
                        crate::authoring::AspectFieldKey::from_authoring_parts("loop", "id")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts("half_edge", "id")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts(
                            "position", "ordinal",
                        )
                        .unwrap(),
                    ])
                    .order_by(
                        crate::authoring::AspectFieldKey::from_authoring_parts(
                            "position", "ordinal",
                        )
                        .unwrap(),
                    )
                    .schema_basis("topology-face-inner-loop-loop-edges")
            },
        )
        .expect("loop-edge live view should declare");

    let receipt = workspace
        .compose_graph(request_execution, |graph| {
            let inner_loop = graph.insert_entity("draft-loop", "Loop", |loop_entity| {
                loop_entity
                    .set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value("loop-inner-1"),
                    )
                    .set_aspect(
                        test_aspect_touch("kind.value"),
                        test_authored_string_aspect_value("inner"),
                    )
            })?;
            let first_half_edge =
                graph.insert_entity("draft-half-edge-a", "HalfEdge", |half_edge| {
                    half_edge
                        .set_aspect(
                            test_aspect_touch("identity.id"),
                            test_authored_string_aspect_value("he-inner-1"),
                        )
                        .set_aspect(
                            test_aspect_touch("kind.value"),
                            test_authored_string_aspect_value("half_edge"),
                        )
                })?;
            let second_half_edge =
                graph.insert_entity("draft-half-edge-b", "HalfEdge", |half_edge| {
                    half_edge
                        .set_aspect(
                            test_aspect_touch("identity.id"),
                            test_authored_string_aspect_value("he-inner-2"),
                        )
                        .set_aspect(
                            test_aspect_touch("kind.value"),
                            test_authored_string_aspect_value("half_edge"),
                        )
                })?;
            graph.insert_relation("FaceLoopRelation", |relation| {
                relation
                    .existing_entity_identity(
                        test_aspect_touch("face.id"),
                        test_entity_identity("face-1"),
                    )
                    .symbolic_entity_identity(test_aspect_touch("loop.id"), &inner_loop)
                    .set_aspect(
                        test_aspect_touch("role.value"),
                        test_authored_string_aspect_value("inner"),
                    )
            })?;
            graph.insert_relation("LoopHalfEdgeRelation", |relation| {
                relation
                    .symbolic_entity_identity(test_aspect_touch("loop.id"), &inner_loop)
                    .symbolic_entity_identity(test_aspect_touch("half_edge.id"), &first_half_edge)
                    .set_aspect(
                        test_aspect_touch("position.ordinal"),
                        test_authored_string_aspect_value("0"),
                    )
            })?;
            graph.insert_relation("LoopHalfEdgeRelation", |relation| {
                relation
                    .symbolic_entity_identity(test_aspect_touch("loop.id"), &inner_loop)
                    .symbolic_entity_identity(test_aspect_touch("half_edge.id"), &second_half_edge)
                    .set_aspect(
                        test_aspect_touch("position.ordinal"),
                        test_authored_string_aspect_value("1"),
                    )
            })?;
            Ok(())
        })
        .expect("face inner loop insertion should execute");
    let loop_identity = receipt.write_receipts()[0].deltas()[0]
        .entity_identity
        .clone();
    let first_half_edge_identity = receipt.write_receipts()[1].deltas()[0]
        .entity_identity
        .clone();
    let second_half_edge_identity = receipt.write_receipts()[2].deltas()[0]
        .entity_identity
        .clone();

    let program = receipt
        .graph_composition_program()
        .expect("graph composition receipt should expose program");
    let lifecycle = receipt
        .graph_composition_lifecycle_outcomes()
        .expect("graph composition receipt should expose lifecycle");
    let evidence = receipt
        .graph_composition_evidence()
        .expect("graph composition receipt should expose evidence");
    let resolution_map = receipt.graph_composition_resolution_map();

    assert_eq!(program.component_count(), 6);
    assert_eq!(receipt.graph_composition_breadth().component_count(), 6);
    assert_eq!(
        receipt
            .graph_composition_breadth()
            .symbolic_entity_declaration_count(),
        3
    );
    assert_eq!(
        receipt
            .graph_composition_breadth()
            .symbolic_relation_declaration_count(),
        3
    );
    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .symbolic_resolution_count(),
        5
    );
    assert_eq!(evidence.symbolic_resolution_count(), 5);
    assert_eq!(
        lifecycle.counter_snapshot(),
        "created=6;updated_identity_preserved=0;retargeted_identity_preserved=0;retired_current_truth=0;superseded_with_lineage=0;deleted_if_uncommitted=0;denied_before_execution=0"
    );
    assert_eq!(
        evidence.counter_snapshot(),
        "components=6;symbolic_entities=3;symbolic_relations=3;symbolic_resolutions=5;affected_live_views=4;affected_derived_views=0;considered_computed_views=0;created=6;updated_identity_preserved=0;retargeted_identity_preserved=0;retired_current_truth=0;superseded_with_lineage=0;deleted_if_uncommitted=0;denied_before_execution=0"
    );
    assert_eq!(resolution_map.len(), 5);
    assert_graph_composition_resolution_snapshot(
        resolution_map,
        &[
            (
                3,
                Some("loop.id".to_string()),
                "draft-loop".to_string(),
                loop_identity.terminal_projection_for_reporting(),
            ),
            (
                4,
                Some("loop.id".to_string()),
                "draft-loop".to_string(),
                loop_identity.terminal_projection_for_reporting(),
            ),
            (
                4,
                Some("half_edge.id".to_string()),
                "draft-half-edge-a".to_string(),
                first_half_edge_identity.terminal_projection_for_reporting(),
            ),
            (
                5,
                Some("loop.id".to_string()),
                "draft-loop".to_string(),
                loop_identity.terminal_projection_for_reporting(),
            ),
            (
                5,
                Some("half_edge.id".to_string()),
                "draft-half-edge-b".to_string(),
                second_half_edge_identity.terminal_projection_for_reporting(),
            ),
        ],
    );
    assert_eq!(
        program.steps()[0].kind(),
        WorthQueryGraphCompositionProgramStepKind::SymbolicEntityDeclaration
    );
    assert_eq!(
        program.steps()[3].kind(),
        WorthQueryGraphCompositionProgramStepKind::RelationDeclaration
    );
    let loop_rows = workspace.read(request_execution, &loops);
    let mut half_edge_rows = workspace.read(request_execution, &half_edges);
    let face_loop_rows = workspace.read(request_execution, &face_loops);
    let loop_edge_rows = workspace.read(request_execution, &loop_edges);
    half_edge_rows.sort_by(|left, right| {
        test_native_string_value(left, "identity.id")
            .cmp(&test_native_string_value(right, "identity.id"))
    });
    assert_eq!(loop_rows.len(), 1);
    assert_eq!(half_edge_rows.len(), 2);
    assert_eq!(
        test_native_string_value(&loop_rows[0], "identity.id").as_deref(),
        Some("loop-inner-1")
    );
    assert_eq!(
        test_native_string_value(&half_edge_rows[0], "identity.id").as_deref(),
        Some("he-inner-1")
    );
    assert_eq!(
        test_native_string_value(&half_edge_rows[1], "identity.id").as_deref(),
        Some("he-inner-2")
    );
    assert_eq!(face_loop_rows.len(), 1);
    let expected_loop = test_native_entity_ref_value(&loop_identity);
    let expected_face = test_native_entity_ref_value(&test_entity_identity("face-1"));
    assert_eq!(
        test_native_scalar_value(&face_loop_rows[0], "loop.id"),
        Some(&expected_loop)
    );
    assert_eq!(
        test_native_scalar_value(&face_loop_rows[0], "face.id"),
        Some(&expected_face)
    );
    assert_eq!(loop_edge_rows.len(), 2);
    let loop_edge_half_edge_ids = loop_edge_rows
        .iter()
        .filter_map(|row| test_native_scalar_value(row, "half_edge.id").cloned())
        .collect::<Vec<_>>();
    assert!(
        loop_edge_half_edge_ids.contains(&test_native_entity_ref_value(&first_half_edge_identity))
    );
    assert!(
        loop_edge_half_edge_ids.contains(&test_native_entity_ref_value(&second_half_edge_identity))
    );

    match workspace
        .inspect(request_execution, &receipt)
        .expect("receipt should inspect")
    {
        WorthQueryInspection::BatchWriteReceipt(inspection) => {
            assert_eq!(
                inspection
                    .graph_composition_evidence()
                    .expect("inspection should expose graph composition evidence")
                    .symbolic_resolution_count(),
                5
            );
            assert_eq!(
                inspection
                    .graph_composition_resolution_map()
                    .entries()
                    .len(),
                5
            );
            assert_graph_composition_resolution_maps_match(
                inspection.graph_composition_resolution_map(),
                resolution_map,
            );
            assert_eq!(
                inspection.component_operations()[3]
                    .symbolic_aspect_resolution_evidence()
                    .len(),
                1
            );
            assert_eq!(
                inspection.component_operations()[4]
                    .symbolic_aspect_resolution_evidence()
                    .len(),
                2
            );
            assert_eq!(
                inspection.component_operations()[5]
                    .symbolic_aspect_resolution_evidence()
                    .len(),
                2
            );
            assert_eq!(
                inspection.component_operations()[4].symbolic_aspect_resolution_evidence()[1]
                    .resolved_entity_identity(),
                &first_half_edge_identity
            );
            assert_eq!(
                inspection.component_operations()[5].symbolic_aspect_resolution_evidence()[1]
                    .resolved_entity_identity(),
                &second_half_edge_identity
            );
        }
        other => panic!("expected batch receipt inspection, got {other:?}"),
    }
}
