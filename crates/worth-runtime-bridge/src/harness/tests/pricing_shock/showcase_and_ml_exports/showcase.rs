use super::super::support::*;

#[test]
fn pricing_shock_showcase_artifact_explains_retained_commit_without_hidden_memory() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let scenario = generated_pricing_scenario();
    let bundle = capture_pricing_workload_certification_bundle(
        BridgeRuntimePolicy::development(),
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-showcase-artifact"),
        resource_request,
    );
    let artifact = bundle.showcase_artifact_json();
    let shock_commit = bundle
        .showcase_commit_explorer_json("commit:rubber-shock")
        .expect("shock commit should be explorable from the showcase artifact");
    let markdown = bundle.showcase_markdown_report();
    let completeness = bundle.bundle_completeness_evidence();

    assert_eq!(
        bundle.provenance.shock_commit,
        crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-shock")
    );
    assert_eq!(bundle.provenance.shock_multiplier_per_mille, 4000);
    assert_eq!(bundle.fanout.second_delivery_target_count, 100);
    assert_eq!(
        bundle.provenance.shock_delta_microunits,
        scenario.speculative_rubber_cost - scenario.main_rubber_cost
    );
    assert_ne!(
        bundle.matrix.reference.main_snapshot,
        bundle.matrix.reference.speculative_snapshot
    );
    assert_eq!(
        bundle.merge.merged_rubber_cost_cents,
        bundle.merge.speculative_rubber_cost_cents
    );
    assert_eq!(bundle.provenance.representative_sku, "scooter-001");
    assert_eq!(bundle.portfolio.product_count, 100);
    assert!(bundle.portfolio.positive_retail_delta_count > 0);
    assert_eq!(
        bundle.crisis.crisis_name,
        "energy-logistics-industrial-crunch"
    );
    assert!(!bundle.crisis.policy_pressure_family.is_empty());
    assert!(!bundle.strategy.recommended_strategy.is_empty());
    assert_eq!(
        bundle.strategy.promotion_strategy,
        "promote-speculative-strategy"
    );
    assert_eq!(bundle.simulation.branch_count, 10);
    assert_eq!(bundle.simulation.iterations_per_branch, 10);
    assert!(!bundle.simulation.ranked_materials_by_damage.is_empty());
    assert_eq!(
        bundle.trust_attacks.replay_policy_error_kind,
        crate::facade::BridgePolicyRejectionKind::ReplayPolicyConflict
    );
    assert_eq!(
        bundle
            .certification_counter_evidence()
            .trust_attack_classification_count,
        8
    );
    assert!(completeness.offline_sufficient);
    assert!(artifact["trust_proof"]["suite_27"].is_object());
    let trust_attacks = artifact["trust_attack_matrix"]
        .as_array()
        .expect("trust attack matrix should be an array");
    assert_eq!(trust_attacks.len(), 8);
    assert_eq!(
        artifact["demo_flow"][3]
            .as_str()
            .expect("demo flow entry should be string"),
        "measure portfolio blast radius"
    );
    assert_eq!(
        artifact["demo_artifact_family"]["showcase_digest"]
            .as_str()
            .expect("showcase digest should be exported as string"),
        bundle.digest()
    );
    assert_eq!(
        artifact["timeline"][2]["commit"]
            .as_str()
            .expect("timeline commit should be exported as string"),
        "commit:rubber-shock"
    );
    assert_eq!(
        artifact["timeline"][4]["snapshot"]
            .as_str()
            .expect("timeline snapshot should be exported as string"),
        bundle.merge.merged_snapshot.as_str()
    );
    assert_eq!(
        shock_commit["snapshot"]
            .as_str()
            .expect("shock commit snapshot should be exported as string"),
        "snapshot:pricing-shock"
    );
    assert_eq!(
        shock_commit["representative_retail_price_cents"]
            .as_i64()
            .expect("representative retail price should be exported as integer"),
        bundle.provenance.representative_retail_price_cents
    );
    assert!(markdown.contains("# Pricing Shock Showcase Report"));
    assert!(markdown.contains("commit:rubber-shock"));
    assert!(markdown.contains("scooter-001"));
    assert!(markdown.contains("Suite 27"));
    assert!(markdown.contains("Trust Attacks"));
    assert!(markdown.contains("Demo Flow"));
    assert!(markdown.contains("energy-logistics-industrial-crunch"));
    assert!(markdown.contains(&bundle.strategy.recommended_strategy));
    assert!(markdown.contains(
        bundle
            .simulation
            .ranked_materials_by_damage
            .first_material()
            .expect("simulation should rank at least one material")
    ));
}

#[test]
fn pricing_shock_showcase_timeline_is_lineage_coherent() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = capture_pricing_workload_certification_bundle(
        BridgeRuntimePolicy::development(),
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-showcase-timeline"),
        resource_request,
    );
    let artifact = bundle.showcase_artifact_json();
    let timeline = artifact["timeline"]
        .as_array()
        .expect("showcase artifact should expose timeline as an array");
    let edges = bundle.lineage_provenance_edges();

    assert_eq!(
        timeline.len(),
        5,
        "showcase timeline should keep the five canonical phases"
    );

    let has_edge =
        |from: &str, to: &str| edges.iter().any(|edge| edge.from == from && edge.to == to);

    let main_basis = &timeline[0];
    let speculative_shock = &timeline[2];
    let merged_authority = &timeline[4];

    assert_eq!(
        main_basis["commit"]
            .as_str()
            .expect("timeline commit should export as a string"),
        bundle.matrix.reference.source_commit.as_str()
    );
    assert_eq!(
        main_basis["snapshot"]
            .as_str()
            .expect("timeline snapshot should export as a string"),
        bundle.matrix.reference.main_snapshot.as_str()
    );
    assert_eq!(
        speculative_shock["commit"]
            .as_str()
            .expect("timeline commit should export as a string"),
        bundle.provenance.shock_commit.as_str()
    );
    assert_eq!(
        speculative_shock["snapshot"]
            .as_str()
            .expect("timeline snapshot should export as a string"),
        bundle.provenance.shock_snapshot.as_str()
    );
    assert_eq!(
        merged_authority["snapshot"]
            .as_str()
            .expect("timeline snapshot should export as a string"),
        bundle.merge.merged_snapshot.as_str()
    );

    assert!(has_edge(
        bundle.matrix.reference.source_commit.as_str(),
        bundle.matrix.reference.main_snapshot.as_str()
    ));
    assert!(has_edge(
        bundle.provenance.shock_commit.as_str(),
        bundle.provenance.shock_snapshot.as_str()
    ));
    assert!(has_edge(
        bundle.merge.speculative_snapshot.as_str(),
        bundle.merge.merged_snapshot.as_str()
    ));
    assert!(has_edge(
        bundle.merge.main_premerge_snapshot.as_str(),
        bundle.merge.merged_snapshot.as_str()
    ));
}

#[test]
fn pricing_shock_showcase_trust_attack_matrix_is_bundle_derived() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = capture_pricing_workload_certification_bundle(
        BridgeRuntimePolicy::development(),
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-showcase-trust-derived"),
        resource_request,
    );
    let artifact = bundle.showcase_artifact_json();
    let trust_attacks = artifact["trust_attack_matrix"]
        .as_array()
        .expect("showcase artifact should expose trust_attack_matrix as an array");

    assert_eq!(
        trust_attacks.len(),
        bundle.counter_snapshot_json()["trust_attack_classification_count"]
            .as_u64()
            .expect("counter snapshot should expose trust attack count") as usize
    );
    assert!(trust_attacks.iter().all(|entry| {
        entry["attack"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
            && entry["classification"]
                .as_str()
                .is_some_and(|value| !value.is_empty())
            && entry["result"]
                .as_str()
                .is_some_and(|value| !value.is_empty())
    }));
    assert_eq!(
        bundle.hostile_failure.failure_class,
        BridgeFailureClass::Delivery(BridgeDeliveryErrorKind::SnapshotAcquisitionFailure)
    );
    assert_eq!(
        bundle.restart_failure.error_kind,
        BridgeReplayErrorKind::RouteMismatch
    );
    assert_eq!(
        bundle.writeback.rejection_error_kind,
        BridgeWritebackErrorKind::MergeAuthorityRejected
    );
    assert_eq!(
        bundle.trust_attacks.replay_policy_error_kind,
        crate::facade::BridgePolicyRejectionKind::ReplayPolicyConflict
    );
    assert_eq!(
        bundle.trust_attacks.route_policy_error_kind,
        crate::facade::BridgeRouteErrorKind::RoutePolicyMismatch
    );
    assert_eq!(
        bundle.trust_attacks.merge_denial_class,
        crate::facade::BridgeMergeDenialClass::TopologyRewireGate
    );
    assert!(!bundle.simulation.ranked_materials_by_damage.is_empty());
}
