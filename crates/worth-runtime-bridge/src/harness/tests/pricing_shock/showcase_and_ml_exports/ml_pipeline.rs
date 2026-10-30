use super::super::support::*;

#[test]
fn pricing_shock_ml_pipeline_export_contains_full_traceable_simulation_artifacts() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = capture_pricing_workload_certification_bundle(
        BridgeRuntimePolicy::development(),
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-ml-export"),
        resource_request,
    );
    let export = bundle.ml_pipeline_export_json();
    let reference_comparison = bundle.reference_workload_comparison_evidence();

    assert_eq!(
        export["schema"]
            .as_str()
            .expect("ML export schema should be a string"),
        "worth-runtime-bridge.pricing-showcase.ml-pipeline.v1"
    );
    assert_eq!(
        export["bundle_digest"]
            .as_str()
            .expect("ML export bundle digest should be a string"),
        bundle.digest()
    );
    assert_eq!(bundle.simulation.branch_count, 10);
    assert_eq!(bundle.simulation.iterations_per_branch, 10);
    assert_eq!(
        export["simulation"]["material_summaries"]
            .as_array()
            .map(|array| array.len())
            .unwrap_or_default(),
        bundle.simulation.material_summaries.len()
    );
    assert_eq!(
        export["simulation"]["iteration_traces"]
            .as_array()
            .map(|array| array.len())
            .unwrap_or_default(),
        bundle.simulation.iteration_traces.len()
    );
    assert_eq!(
        export["simulation"]["ranked_materials_by_damage"][0]
            .as_str()
            .expect("ranked material should export as a string"),
        bundle
            .simulation
            .ranked_materials_by_damage
            .first_material()
            .expect("simulation should rank at least one material")
    );
    assert_eq!(
        bundle.provenance.shock_commit,
        crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-shock")
    );
    assert!(!bundle.matrix.reference.source_commit.as_str().is_empty());
    assert!(!bundle
        .matrix
        .reference
        .speculative_truth_branch
        .as_str()
        .is_empty());
    assert!(!bundle.aspect.aspect_registration_id.as_str().is_empty());
    assert!(!bundle
        .promotion
        .promotion_session_identity
        .as_str()
        .is_empty());
    assert!(!bundle.merge.bundle_digest.is_empty());
    assert!(bundle.provenance.shock_delta_microunits > 0);
    assert!(!bundle.hostile_failure.source_commit.as_str().is_empty());
    assert_eq!(
        bundle.trust_attacks.replay_policy_error_kind,
        crate::facade::BridgePolicyRejectionKind::ReplayPolicyConflict
    );
    assert_eq!(
        export["lineage_provenance"]["causality"]["bundle_digest"]
            .as_str()
            .expect("causality bundle digest should export as a string"),
        bundle.digest()
    );
    assert_eq!(
        export["lineage_provenance"]["causality"]["suite_25_causality_digest"]
            .as_str()
            .expect("suite 25 causality digest should export as a string"),
        bundle.suite_25_digest_evidence().causality_digest
    );
    assert!(export["lineage_provenance_edges"]
        .as_array()
        .is_some_and(|edges| !edges.is_empty()));
    let typed_edges = bundle.lineage_provenance_edges();
    assert!(typed_edges
        .iter()
        .any(|edge| edge.kind == "commit_to_snapshot"
            && edge.from == bundle.provenance.shock_commit.as_str()
            && edge.to == bundle.provenance.shock_snapshot.as_str()));
    assert!(typed_edges
        .iter()
        .any(|edge| edge.kind == "speculative_to_merged_snapshot"
            && edge.from == bundle.merge.speculative_snapshot.as_str()
            && edge.to == bundle.merge.merged_snapshot.as_str()));
    assert!(typed_edges
        .iter()
        .any(|edge| edge.kind == "bundle_to_causality_digest" && edge.from == bundle.digest()));
    assert!(reference_comparison.simulation_identifies_at_least_one_damaging_material);
    assert!(reference_comparison.trust_attack_matrix_is_typed);
    assert!(export["suite_27"].is_object());
}

#[test]
fn pricing_shock_ml_pipeline_export_lineage_graph_is_well_formed() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = capture_pricing_workload_certification_bundle(
        BridgeRuntimePolicy::development(),
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-ml-graph"),
        resource_request,
    );
    let export = bundle.ml_pipeline_export_json();
    let edges = export["lineage_provenance_edges"]
        .as_array()
        .expect("ml export should expose lineage_provenance_edges as an array");

    let valid_nodes = std::collections::HashSet::from([
        bundle.matrix.reference.source_commit.as_str().to_owned(),
        bundle.matrix.reference.main_snapshot.as_str().to_owned(),
        bundle.provenance.main_commit.as_str().to_owned(),
        bundle.provenance.main_snapshot.as_str().to_owned(),
        bundle.provenance.shock_commit.as_str().to_owned(),
        bundle.provenance.shock_snapshot.as_str().to_owned(),
        bundle
            .matrix
            .reference
            .speculative_snapshot
            .as_str()
            .to_owned(),
        bundle.aspect.source_commit.as_str().to_owned(),
        bundle.matrix.replay.route_identity.as_str().to_owned(),
        bundle
            .matrix
            .replay
            .invalidation_identity
            .as_str()
            .to_owned(),
        bundle.aspect.aspect_registration_id.as_str().to_owned(),
        bundle.aspect.invalidation_target.clone(),
        bundle
            .promotion
            .promotion_session_identity
            .as_str()
            .to_owned(),
        bundle
            .promotion
            .authoritative_commit_boundary_digest
            .clone(),
        bundle.promotion.authoritative_artifact_digest.clone(),
        bundle.fanout.second_source_commit.as_str().to_owned(),
        bundle.fanout.second_snapshot.as_str().to_owned(),
        bundle.restart_replay.source_commit.as_str().to_owned(),
        bundle.restart_replay.route_identity.as_str().to_owned(),
        format!("{:?}", bundle.writeback.family_kind),
        bundle.writeback.commit_replay_semantic_digest.clone(),
        bundle.merge.main_premerge_snapshot.as_str().to_owned(),
        bundle.merge.speculative_snapshot.as_str().to_owned(),
        bundle.merge.merged_snapshot.as_str().to_owned(),
        bundle.merge.bundle_digest.clone(),
        bundle.merge.canonical_replay_digest.clone(),
        bundle.hostile_failure.source_commit.as_str().to_owned(),
        bundle.hostile_failure.source_snapshot.as_str().to_owned(),
        bundle.digest(),
        bundle.suite_25_digest_evidence().causality_digest,
    ]);

    let mut seen_edges = std::collections::HashSet::new();
    let mut seen_edge_kinds = std::collections::HashSet::new();

    for edge in edges {
        let from = edge["from"]
            .as_str()
            .expect("lineage edge should expose string `from`");
        let to = edge["to"]
            .as_str()
            .expect("lineage edge should expose string `to`");
        let kind = edge["kind"]
            .as_str()
            .expect("lineage edge should expose string `kind`");
        let surface = edge["surface"]
            .as_str()
            .expect("lineage edge should expose string `surface`");

        assert!(!from.is_empty() && !to.is_empty() && !kind.is_empty() && !surface.is_empty());
        assert!(valid_nodes.contains(from));
        assert!(valid_nodes.contains(to));
        assert!(seen_edges.insert((
            from.to_owned(),
            to.to_owned(),
            kind.to_owned(),
            surface.to_owned()
        )));
        seen_edge_kinds.insert(kind.to_owned());
    }

    for required_kind in [
        "commit_to_snapshot",
        "commit_to_route",
        "route_to_invalidation",
        "aspect_to_target",
        "speculative_to_merged_snapshot",
        "bundle_to_causality_digest",
    ] {
        assert!(seen_edge_kinds.contains(required_kind));
    }
}

#[test]
fn pricing_shock_ml_pipeline_export_simulation_summaries_match_iteration_traces() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = capture_pricing_workload_certification_bundle(
        BridgeRuntimePolicy::development(),
        BridgePreviewSessionIdentity::admit_bridge_owned(
            "pricing:preview-ml-simulation-consistency",
        ),
        resource_request,
    );
    let export = bundle.ml_pipeline_export_json();
    assert_eq!(
        export["simulation"]["material_summaries"]
            .as_array()
            .expect("ml export should expose material_summaries as an array")
            .len(),
        bundle.simulation.material_summaries.len()
    );
    assert_eq!(
        export["simulation"]["iteration_traces"]
            .as_array()
            .expect("ml export should expose iteration_traces as an array")
            .len(),
        bundle.simulation.iteration_traces.len()
    );
    assert_eq!(
        export["simulation"]["branch_count"]
            .as_u64()
            .expect("branch count should export as an integer") as usize,
        bundle.simulation.branch_count
    );
    assert_eq!(
        export["simulation"]["iterations_per_branch"]
            .as_u64()
            .expect("iteration count should export as an integer") as usize,
        bundle.simulation.iterations_per_branch
    );

    let mut ranked_from_summaries = Vec::new();

    for material_summary in &bundle.simulation.material_summaries {
        let matching_traces = bundle
            .simulation
            .iteration_traces
            .iter()
            .filter(|trace| trace.material == material_summary.material)
            .collect::<Vec<_>>();
        assert_eq!(
            matching_traces.len(),
            bundle.simulation.branch_count * bundle.simulation.iterations_per_branch
        );

        let total_retail_delta = matching_traces
            .iter()
            .map(|trace| trace.total_retail_delta_cents)
            .sum::<i64>();
        let total_shipping_delta = matching_traces
            .iter()
            .map(|trace| trace.shipping_delta_cents)
            .sum::<i64>();
        let total_material_delta = matching_traces
            .iter()
            .map(|trace| trace.material_delta_cents)
            .sum::<i64>();
        let total_breach_count = matching_traces
            .iter()
            .map(|trace| trace.margin_floor_breach_count as i64)
            .sum::<i64>();
        let total_repricing_count = matching_traces
            .iter()
            .map(|trace| trace.repricing_count as i64)
            .sum::<i64>();
        let total_iterations = matching_traces.len() as i64;

        let expected_mean_total = total_retail_delta / total_iterations;
        let expected_mean_shipping = total_shipping_delta / total_iterations;
        let expected_mean_material = total_material_delta / total_iterations;
        let expected_mean_breach = total_breach_count / total_iterations;
        let expected_mean_repricing = total_repricing_count / total_iterations;
        let expected_damage_score =
            expected_mean_total + (expected_mean_breach * 50) + expected_mean_shipping.abs() / 10;

        assert_eq!(
            material_summary.mean_total_retail_delta_cents,
            expected_mean_total
        );
        assert_eq!(
            material_summary.mean_shipping_delta_cents,
            expected_mean_shipping
        );
        assert_eq!(
            material_summary.mean_material_delta_cents,
            expected_mean_material
        );
        assert_eq!(
            material_summary.mean_margin_floor_breach_count,
            expected_mean_breach
        );
        assert_eq!(
            material_summary.mean_repricing_count,
            expected_mean_repricing
        );
        assert_eq!(material_summary.damage_score, expected_damage_score);

        let mut branch_totals = std::collections::BTreeMap::<String, i64>::new();
        for trace in matching_traces {
            *branch_totals
                .entry(trace.branch_identity.clone())
                .or_default() += trace.total_retail_delta_cents;
        }
        let (expected_worst_branch_identity, expected_worst_branch_total) = branch_totals
            .into_iter()
            .max_by_key(|(_, delta)| *delta)
            .expect("branch totals should not be empty");
        assert_eq!(
            material_summary.worst_branch_identity,
            expected_worst_branch_identity
        );
        assert_eq!(
            material_summary.worst_branch_mean_total_delta_cents,
            expected_worst_branch_total / bundle.simulation.iterations_per_branch as i64
        );

        ranked_from_summaries.push((
            material_summary.material.clone(),
            expected_damage_score,
            expected_mean_total,
        ));
    }

    ranked_from_summaries
        .sort_by(|left, right| right.1.cmp(&left.1).then_with(|| right.2.cmp(&left.2)));
    let expected_ranked_materials = ranked_from_summaries
        .into_iter()
        .map(|(material, _, _)| material)
        .collect::<Vec<_>>();
    assert_eq!(
        bundle
            .simulation
            .ranked_materials_by_damage
            .material_names()
            .map(str::to_owned)
            .collect::<Vec<_>>(),
        expected_ranked_materials
    );
    let exported_ranked_materials = export["simulation"]["ranked_materials_by_damage"]
        .as_array()
        .expect("ml export should expose ranked_materials_by_damage as an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("ranked_materials_by_damage should contain strings")
                .to_owned()
        })
        .collect::<Vec<_>>();

    assert_eq!(
        exported_ranked_materials,
        bundle
            .simulation
            .ranked_materials_by_damage
            .material_names()
            .map(str::to_owned)
            .collect::<Vec<_>>()
    );
}
