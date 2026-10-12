use super::*;

#[test]
fn compose_graph_supports_verified_edge_split_with_lineage_summary() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let binding = geometry_relation_binding("authority:edge-main", "Edge:1", "Edge");
    let expected_basis_binding_digest = WorthQueryMutationEvidenceDigest::source_identity(
        "continuity-basis-binding",
        binding.binding_evidence_identity(),
    );
    let expected_edge_identity = binding.resolved_relation_identity().clone();
    let runtime = edge_split_runtime(request_execution, &binding);
    let mut workspace = runtime
        .workspace("topology.graph-composition-edge-split")
        .expect("workspace should open");
    declare_edge_split_live_views(request_execution, &mut workspace);

    let receipt = compose_verified_edge_split(request_execution, &mut workspace, binding);

    let split_vertex_identity = receipt.write_receipts()[0].deltas()[0]
        .entity_identity
        .clone();
    let left_edge_identity = receipt.write_receipts()[1].deltas()[0]
        .entity_identity
        .clone();
    let right_edge_identity = receipt.write_receipts()[2].deltas()[0]
        .entity_identity
        .clone();

    let program = receipt
        .graph_composition_program()
        .expect("edge split receipt should expose program");
    let lifecycle = receipt
        .graph_composition_lifecycle_outcomes()
        .expect("edge split receipt should expose lifecycle");
    let evidence = receipt
        .graph_composition_evidence()
        .expect("edge split receipt should expose evidence");
    let assumptions = receipt
        .graph_composition_assumption_summary()
        .expect("verified edge split should expose assumption summary");
    let lineage = receipt
        .graph_composition_lineage_summary()
        .expect("edge split should expose lineage summary");
    let resolution_map = receipt.graph_composition_resolution_map();
    let continuity = receipt.write_receipts()[7]
        .continuity_mutation_evidence()
        .expect("verified supersession component should retain continuity evidence");

    assert_eq!(program.component_count(), 8);
    assert_eq!(receipt.graph_composition_breadth().component_count(), 8);
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
        4
    );
    assert_eq!(
        receipt.write_receipts()[7].terminal_target_collection_projection(),
        Some("Edge")
    );
    assert_eq!(
        continuity
            .basis_binding_digest()
            .expect("continuity should retain typed binding basis digest")
            .as_str(),
        expected_basis_binding_digest.as_str()
    );
    assert_eq!(
        continuity.resolved_target_entity_identity(),
        Some(&expected_edge_identity)
    );
    assert_eq!(
        program.steps()[7].kind(),
        WorthQueryGraphCompositionProgramStepKind::ExistingTargetVerifiedSupersession
    );
    assert_eq!(program.steps()[7].declared_collection(), "Edge");
    assert_eq!(
        lifecycle.entries()[7].outcome_kind(),
        WorthQueryGraphCompositionLifecycleOutcomeKind::SupersededWithLineage
    );
    assert_eq!(
        lifecycle.counter_snapshot(),
        "created=7;updated_identity_preserved=0;retargeted_identity_preserved=0;retired_current_truth=0;superseded_with_lineage=1;deleted_if_uncommitted=0;denied_before_execution=0"
    );
    assert_eq!(
        evidence.counter_snapshot(),
        "components=8;symbolic_entities=3;symbolic_relations=4;symbolic_resolutions=8;affected_live_views=3;affected_derived_views=0;considered_computed_views=0;created=7;updated_identity_preserved=0;retargeted_identity_preserved=0;retired_current_truth=0;superseded_with_lineage=1;deleted_if_uncommitted=0;denied_before_execution=0"
    );
    assert_eq!(evidence.symbolic_resolution_count(), 8);
    assert_eq!(
        assumptions.counter_snapshot(),
        "verified_steps=1;target_bindings=1;asserted_aspects=2;distinct_asserted_aspect_touches=2;cleared_assertions=0"
    );
    assert_eq!(
        lineage.counter_snapshot(),
        "continuity_entries=1;single_successors=0;split_successors=1;merge_successors=0;rejections=0"
    );
    assert_eq!(lineage.entries().len(), 1);
    assert_eq!(lineage.entries()[0].component_index(), 7);
    assert_eq!(
        lineage.entries()[0].family(),
        WorthQueryContinuityMutationFamily::SplitExistingTarget
    );
    assert_eq!(
        lineage.entries()[0].outcome_class(),
        WorthQueryContinuityOutcomeClass::ContinuesAsSplitSuccessors
    );
    assert_eq!(
        lineage.entries()[0].prior_authoritative_identity().as_str(),
        "authority:edge-main"
    );
    assert_eq!(
        lineage.entries()[0]
            .successor_authoritative_identities()
            .iter()
            .map(|identity| identity.as_str())
            .collect::<Vec<_>>(),
        vec!["authority:edge-left", "authority:edge-right"]
    );
    assert_eq!(
        lineage.entries()[0]
            .target_collection()
            .map(|collection| collection.as_str()),
        Some("Edge")
    );
    assert_eq!(
        lineage.entries()[0].lineage_digest(),
        continuity.lineage_digest()
    );
    assert_eq!(
        lineage.entries()[0].continuity_resolution_digest(),
        continuity.continuity_resolution_digest()
    );
    assert_eq!(
        evidence
            .lineage_summary()
            .expect("graph composition evidence should retain lineage summary")
            .aggregate_continuity_resolution_digest(),
        lineage.aggregate_continuity_resolution_digest()
    );
    assert_eq!(
        evidence
            .lineage_summary()
            .expect("graph composition evidence should retain lineage summary")
            .lineage_summary_digest(),
        lineage.lineage_summary_digest()
    );
    assert_eq!(resolution_map.len(), 8);
    assert_graph_composition_resolution_snapshot(
        resolution_map,
        &[
            (
                1,
                Some("target.id".to_string()),
                "split-vertex".to_string(),
                split_vertex_identity.terminal_projection_for_reporting(),
            ),
            (
                2,
                Some("source.id".to_string()),
                "split-vertex".to_string(),
                split_vertex_identity.terminal_projection_for_reporting(),
            ),
            (
                3,
                Some("edge.id".to_string()),
                "edge-left".to_string(),
                left_edge_identity.terminal_projection_for_reporting(),
            ),
            (
                4,
                Some("vertex.id".to_string()),
                "split-vertex".to_string(),
                split_vertex_identity.terminal_projection_for_reporting(),
            ),
            (
                4,
                Some("edge.id".to_string()),
                "edge-left".to_string(),
                left_edge_identity.terminal_projection_for_reporting(),
            ),
            (
                5,
                Some("vertex.id".to_string()),
                "split-vertex".to_string(),
                split_vertex_identity.terminal_projection_for_reporting(),
            ),
            (
                5,
                Some("edge.id".to_string()),
                "edge-right".to_string(),
                right_edge_identity.terminal_projection_for_reporting(),
            ),
            (
                6,
                Some("edge.id".to_string()),
                "edge-right".to_string(),
                right_edge_identity.terminal_projection_for_reporting(),
            ),
        ],
    );

    match workspace
        .inspect(request_execution, &receipt)
        .expect("receipt should inspect")
    {
        WorthQueryInspection::BatchWriteReceipt(inspection) => {
            let inspection_lineage = inspection
                .graph_composition_lineage_summary()
                .expect("inspection should expose lineage summary");
            assert_eq!(
                inspection
                    .graph_composition_program()
                    .expect("inspection should expose program")
                    .steps()[7]
                    .kind(),
                WorthQueryGraphCompositionProgramStepKind::ExistingTargetVerifiedSupersession
            );
            assert_eq!(
                inspection.component_operations()[7].target_collection(),
                Some("Edge")
            );
            assert_eq!(
                inspection
                    .graph_composition_lifecycle_outcomes()
                    .expect("inspection should expose lifecycle")
                    .entries()[7]
                    .outcome_kind(),
                WorthQueryGraphCompositionLifecycleOutcomeKind::SupersededWithLineage
            );
            assert_eq!(
                inspection
                    .graph_composition_assumption_summary()
                    .expect("inspection should expose assumption summary")
                    .assumption_summary_digest(),
                assumptions.assumption_summary_digest()
            );
            assert_eq!(
                inspection_lineage.lineage_summary_digest(),
                lineage.lineage_summary_digest()
            );
            assert_eq!(
                inspection_lineage.entries()[0]
                    .successor_authoritative_identities()
                    .iter()
                    .map(|identity| identity.as_str())
                    .collect::<Vec<_>>(),
                vec!["authority:edge-left", "authority:edge-right"]
            );
            assert_eq!(
                inspection_lineage.entries()[0]
                    .target_collection()
                    .map(|collection| collection.as_str()),
                Some("Edge")
            );
            assert_eq!(
                inspection_lineage.entries()[0].continuity_resolution_digest(),
                continuity.continuity_resolution_digest()
            );
            assert_eq!(
                inspection_lineage.aggregate_continuity_resolution_digest(),
                lineage.aggregate_continuity_resolution_digest()
            );
            assert_graph_composition_resolution_maps_match(
                inspection.graph_composition_resolution_map(),
                resolution_map,
            );
            assert_eq!(
                inspection.component_operations()[7]
                    .continuity_mutation_evidence()
                    .expect("inspection should retain continuity evidence")
                    .outcome_class(),
                WorthQueryContinuityOutcomeClass::ContinuesAsSplitSuccessors
            );
        }
        other => panic!("expected batch receipt inspection, got {other:?}"),
    }
}
