use super::support::*;

#[test]
fn pricing_shock_reference_matrix_preserves_semantic_truth_across_diagnostics_profiles() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let scenario = generated_pricing_scenario();
    let truth = RelationalPricing::reference(resource_request, &scenario, None);
    let baseline = capture_pricing_certification_matrix(
        BridgeRuntimePolicy::development(),
        &truth,
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-baseline"),
        resource_request,
    );
    let forensic = capture_pricing_certification_matrix(
        BridgeRuntimePolicy::forensic(),
        &truth,
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-forensic"),
        resource_request,
    );

    assert_eq!(baseline.reference, forensic.reference);
    assert_eq!(baseline.replay, forensic.replay);
}

#[test]
fn pricing_shock_route_replay_preserves_canonical_main_branch_truth() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let scenario = generated_pricing_scenario();
    let truth = RelationalPricing::reference(resource_request, &scenario, None);
    let replay = capture_pricing_certification_matrix(
        BridgeRuntimePolicy::development(),
        &truth,
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-replay-control"),
        resource_request,
    )
    .replay;

    assert_eq!(replay.source_commit, truth.steel_commit.clone());
    assert_eq!(replay.source_snapshot, truth.steel_snapshot().clone());
    assert!(!replay.route_identity.as_str().is_empty());
    assert!(!replay.invalidation_identity.as_str().is_empty());
}

#[test]
fn pricing_shock_duplicate_commit_identity_with_conflicting_route_meaning_is_detectable() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = RelationalPricing::new(execution, 100, 40, 56);
    let runtime = truth.fault_runtime(
        PricingFault::PatchItems(
            truth.steel_commit.clone(),
            vec![truth.cost_item(PricingMaterial::Rubber)],
        ),
        BridgeRuntimePolicy::development(),
    );
    let route = runtime
        .route(truth.steel_commit.clone(), execution)
        .expect("conflicting duplicate commit identity should still route as retained truth");
    assert_eq!(
        route.result().result_summary().source_commit(),
        &truth.steel_commit
    );
    assert_eq!(
        route.result().receipt().snapshot_identity(),
        truth.steel_snapshot()
    );
    assert_eq!(route.result().receipt().delivered_target_count(), 1);
    assert_eq!(
        route.result().artifact().invalidation_targets().targets()[0].signal_scope(),
        "price:scooter"
    );
}

#[test]
fn pricing_shock_duplicate_conflicting_commit_identity_permutation_sweep_is_detectable() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = RelationalPricing::new(execution, 100, 40, 56);
    for (label, commit, snapshot, materials, expected_targets) in [
        (
            "steel changed to rubber",
            &truth.steel_commit,
            truth.steel_snapshot(),
            vec![PricingMaterial::Rubber],
            vec!["price:scooter"],
        ),
        (
            "rubber changed to steel",
            &truth.rubber_commit,
            truth.main_snapshot(),
            vec![PricingMaterial::Steel],
            vec!["price:bicycle", "price:wheelbarrow"],
        ),
        (
            "steel changed to both",
            &truth.steel_commit,
            truth.steel_snapshot(),
            vec![PricingMaterial::Steel, PricingMaterial::Rubber],
            vec!["price:bicycle", "price:scooter", "price:wheelbarrow"],
        ),
    ] {
        let items = materials
            .into_iter()
            .map(|material| truth.cost_item(material))
            .collect();
        let runtime = truth.fault_runtime(
            PricingFault::PatchItems(commit.clone(), items),
            BridgeRuntimePolicy::development(),
        );
        let route = runtime
            .route(commit.clone(), execution)
            .unwrap_or_else(|_| panic!("{label} should still route as retained truth"));
        assert_eq!(route.result().result_summary().source_commit(), commit);
        assert_eq!(route.result().receipt().snapshot_identity(), snapshot);

        assert_eq!(
            route.result().receipt().delivered_target_count(),
            expected_targets.len(),
            "{label} should deliver the expected target width"
        );

        let mut actual_targets = route
            .result()
            .artifact()
            .invalidation_targets()
            .targets()
            .iter()
            .map(|target| target.signal_scope().to_owned())
            .collect::<Vec<_>>();
        actual_targets.sort();

        let mut expected_targets = expected_targets
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        expected_targets.sort();

        assert_eq!(
            actual_targets, expected_targets,
            "{label} should surface the conflicting retained meaning"
        );
    }
}

#[test]
fn pricing_shock_non_commuting_route_history_attack_fails_closed_on_replay() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = RelationalPricing::new(execution, 100, 40, 56);
    let original_runtime = truth.runtime(RecordingSignalBridgeSink::default());
    original_runtime
        .route(truth.steel_commit.clone(), execution)
        .expect("original steel route should succeed before replay attack");
    let canonical_record = original_runtime
        .diagnostics()
        .last_canonical_route_record()
        .expect("original runtime should expose canonical route record");

    let restarted_runtime = truth.fault_runtime(
        PricingFault::PatchItems(
            truth.steel_commit.clone(),
            vec![truth.cost_item(PricingMaterial::Rubber)],
        ),
        BridgeRuntimePolicy::development(),
    );
    let error = restarted_runtime
        .replay_canonical_record(&canonical_record, execution)
        .expect_err("replay should reject non-commuting route history drift");

    assert!(!error.to_string().is_empty());
    let failure_record = restarted_runtime
        .diagnostics()
        .last_failure_record()
        .expect("replay failure should retain diagnostics");
    assert_eq!(failure_record.counters().route_replay_mismatch_count(), 1);
}

#[test]
fn pricing_shock_non_commuting_route_history_permutation_sweep_fails_closed() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = RelationalPricing::new(execution, 100, 40, 56);
    for (label, commit, materials) in [
        (
            "steel changed to rubber",
            &truth.steel_commit,
            vec![PricingMaterial::Rubber],
        ),
        (
            "rubber changed to steel",
            &truth.rubber_commit,
            vec![PricingMaterial::Steel],
        ),
        (
            "steel changed to both",
            &truth.steel_commit,
            vec![PricingMaterial::Steel, PricingMaterial::Rubber],
        ),
    ] {
        let original_runtime = truth.runtime(RecordingSignalBridgeSink::default());
        original_runtime
            .route(commit.clone(), execution)
            .unwrap_or_else(|_| panic!("{label} should route canonically before replay attack"));
        let canonical_record = original_runtime
            .diagnostics()
            .last_canonical_route_record()
            .unwrap_or_else(|| panic!("{label} should retain a canonical route record"));

        let items = materials
            .into_iter()
            .map(|material| truth.cost_item(material))
            .collect();
        let restarted_runtime = truth.fault_runtime(
            PricingFault::PatchItems(commit.clone(), items),
            BridgeRuntimePolicy::development(),
        );
        let error = restarted_runtime
            .replay_canonical_record(&canonical_record, execution)
            .err()
            .unwrap_or_else(|| panic!("{label} should fail closed under non-commuting replay"));

        assert_eq!(
            error.kind(),
            BridgeReplayErrorKind::RouteMismatch,
            "{label} should classify as a replay route mismatch"
        );

        let failure_record = restarted_runtime
            .diagnostics()
            .last_failure_record()
            .unwrap_or_else(|| panic!("{label} should retain a failure record"));
        assert_eq!(
            failure_record.counters().route_replay_mismatch_count(),
            1,
            "{label} should increment replay mismatch exactly once"
        );
    }
}
