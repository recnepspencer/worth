use super::support::*;

#[test]
fn pricing_shock_repricing_signal_is_delta_driven_not_always_on() {
    let scenario = generated_pricing_scenario();
    let product_count = scenario.main_portfolio.len();
    let main_repricing_count = scenario
        .main_portfolio
        .iter()
        .filter(|entry| entry.repricing_triggered)
        .count();
    let shock_repricing_count = scenario
        .speculative_portfolio
        .iter()
        .filter(|entry| entry.repricing_triggered)
        .count();
    let increased_shock_pressure_count = scenario
        .main_portfolio
        .iter()
        .zip(scenario.speculative_portfolio.iter())
        .filter(|(main_entry, shock_entry)| {
            shock_entry.landed_cost_delta_cents > main_entry.landed_cost_delta_cents
        })
        .count();

    assert!(main_repricing_count < product_count);
    assert!(shock_repricing_count <= product_count);
    assert!(increased_shock_pressure_count > 0);
    assert!(scenario
        .main_portfolio
        .iter()
        .all(|entry| entry.repricing_threshold_cents > 0));
    assert!(scenario
        .speculative_portfolio
        .iter()
        .all(|entry| entry.repricing_threshold_cents > 0));
    assert!(scenario
        .main_portfolio
        .iter()
        .any(|entry| !entry.repricing_triggered));
    assert!(scenario.main_portfolio.iter().all(|entry| {
        entry.repricing_triggered
            == ((entry.repricing_threshold_cents > 0
                && entry.landed_cost_delta_cents >= entry.repricing_threshold_cents)
                || entry.margin_floor_breached)
    }));
    assert!(scenario.speculative_portfolio.iter().all(|entry| {
        entry.repricing_triggered
            == ((entry.repricing_threshold_cents > 0
                && entry.landed_cost_delta_cents >= entry.repricing_threshold_cents)
                || entry.margin_floor_breached)
    }));
}

#[test]
fn pricing_shock_suites_25_through_27_emit_canonical_machine_checkable_artifacts() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = capture_pricing_workload_certification_bundle(
        BridgeRuntimePolicy::development(),
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-workload-suites"),
        resource_request,
    );
    let suite_25 = bundle.suite_25_digest_evidence();
    let suite_26 = bundle.suite_26_digest_evidence();
    let diagnostics_entrypoints = bundle.diagnostics_entrypoint_evidence();
    let completeness = bundle.bundle_completeness_evidence();
    let counters = bundle.certification_counter_evidence();

    assert_ne!(
        suite_25.reference_workload_bundle_digest,
        suite_26.reference_workload_failure_bundle_digest
    );
    assert_ne!(suite_26.failure_digest, suite_26.replay_failure_digest);
    assert_eq!(counters.offline_bundle_insufficiency_count, 0);
    assert!(diagnostics_entrypoints.all_entrypoints_available());
    assert!(diagnostics_entrypoints.routing);
    assert!(diagnostics_entrypoints.branch_isolation);
    assert!(diagnostics_entrypoints.policy);
    assert!(diagnostics_entrypoints.source);
    assert!(diagnostics_entrypoints.preview);
    assert!(diagnostics_entrypoints.merge);
    assert!(diagnostics_entrypoints.writeback);
    assert!(diagnostics_entrypoints.residue);
    assert!(diagnostics_entrypoints.historical_provenance);
    assert!(diagnostics_entrypoints.portfolio);
    assert!(diagnostics_entrypoints.crisis);
    assert!(diagnostics_entrypoints.strategy);
    assert!(diagnostics_entrypoints.simulation);
    assert!(diagnostics_entrypoints.trust_attacks);
    assert!(completeness.has_routing_artifact);
    assert!(completeness.has_branch_comparison_artifact);
    assert!(completeness.has_policy_artifact);
    assert!(completeness.has_source_artifact);
    assert!(completeness.has_preview_artifact);
    assert!(completeness.has_merge_artifact);
    assert!(completeness.has_writeback_artifact);
    assert!(completeness.has_residue_artifact);
    assert!(completeness.has_historical_provenance_artifact);
    assert!(completeness.has_portfolio_artifact);
    assert!(completeness.has_crisis_artifact);
    assert!(completeness.has_strategy_artifact);
    assert!(completeness.has_simulation_artifact);
    assert!(completeness.has_trust_attack_artifact);
    assert!(completeness.offline_sufficient);
    assert_eq!(completeness.insufficiency_count, 0);
}

#[test]
fn pricing_shock_can_emit_ml_pipeline_export_file_when_requested() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let Some(path) = std::env::var_os("WORTH_PRICING_SHOWCASE_EXPORT_PATH") else {
        return;
    };
    let bundle = capture_pricing_workload_certification_bundle(
        BridgeRuntimePolicy::development(),
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-ml-export-file"),
        resource_request,
    );
    std::fs::write(&path, bundle.ml_pipeline_export_pretty_json())
        .expect("ml pipeline export file should write");
}

#[test]
fn pricing_shock_restart_replay_preserves_canonical_truth_across_rebuild() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let restart =
        capture_pricing_restart_replay_bundle(BridgeRuntimePolicy::development(), resource_request);
    let replay = capture_pricing_certification_matrix(
        BridgeRuntimePolicy::development(),
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-restart-parity"),
        resource_request,
    )
    .replay;

    assert_eq!(restart.source_commit, replay.source_commit);
    assert_eq!(restart.source_snapshot, replay.source_snapshot);
    assert_eq!(restart.route_identity, replay.route_identity);
    assert_eq!(restart.invalidation_identity, replay.invalidation_identity);
}

#[test]
fn pricing_shock_restart_replay_rejects_route_drift_after_truth_change() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let restart_failure = capture_pricing_restart_failure_bundle(resource_request);

    assert_eq!(
        restart_failure.error_kind,
        BridgeReplayErrorKind::RouteMismatch
    );
    assert_eq!(restart_failure.replay_mismatch_count, 1);
}

#[test]
fn pricing_shock_missing_snapshot_fails_with_typed_delivery_record() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(pricing_patch(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-missing-snapshot"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch:steel-missing-snapshot"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-missing"),
        ),
        "steel",
    ));

    let sink = RecordingSignalBridgeSink::default();
    let runtime = build_pricing_runtime(source, sink);
    let failure = capture_pricing_missing_snapshot_failure_bundle(&runtime, resource_request);

    assert!(
        matches!(failure.error_kind, BridgeDeliveryErrorKind::SnapshotAcquisitionFailure(ref cause) if cause.kind() == crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure)
    );
    assert!(
        matches!(failure.failure_class, BridgeFailureClass::Delivery(BridgeDeliveryErrorKind::SnapshotAcquisitionFailure(ref cause)) if cause.kind() == crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure)
    );
    assert_eq!(
        failure.source_commit,
        crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-missing-snapshot")
    );
    assert_eq!(
        failure.source_snapshot,
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-missing")
    );
}
