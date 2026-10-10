use super::support::*;

#[test]
fn pricing_shock_conflicting_historical_basis_is_detectable_against_independent_oracle() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let scenario = generated_pricing_scenario();
    let runtime = build_pricing_runtime(
        pricing_reference_source_with_conflicting_shock_snapshot(),
        RecordingSignalBridgeSink::default(),
    );

    let historical_cost = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_historical_commit(
                crate::truth_identity_fixtures::truth_branch_fixture("pricing-shock"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-shock"),
            )
            .with_read_packet(pricing_component_read_packet("rubber")),
            execution,
        )
        .expect("conflicting historical basis should still materialize as retained truth");
    let historical_provenance = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_historical_commit(
                crate::truth_identity_fixtures::truth_branch_fixture("pricing-shock"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-shock"),
            )
            .with_read_packet(pricing_provenance_read_packet("rubber")),
            execution,
        )
        .expect("conflicting historical basis should materialize provenance packet");
    let provenance_texts = read_pricing_provenance_aspect_text_packet(&historical_provenance);

    assert_eq!(
        historical_cost.snapshot_identity().as_str(),
        "snapshot:pricing-main"
    );
    assert_eq!(
        historical_provenance.snapshot_identity().as_str(),
        "snapshot:pricing-main"
    );
    assert_eq!(
        read_single_money_cents(&historical_cost),
        scenario.main_rubber_cost
    );
    assert_ne!(
        read_single_money_cents(&historical_cost),
        scenario.speculative_rubber_cost
    );
    assert_ne!(
        provenance_texts.shock_delta_text(),
        scenario
            .commit_attributions
            .get("commit:rubber-shock")
            .expect("generated scenario should retain shock attribution")
            .shock_delta_microunits
            .to_string()
    );
}

#[test]
fn pricing_shock_branch_head_and_snapshot_basis_mutation_sweep_is_detectable() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    for (label, source, branch) in [
        (
            "speculative-branch-head-points-at-main",
            pricing_reference_source_with_branch_head_pointing_to(
                "pricing-shock",
                "commit:rubber-main",
            ),
            "pricing-shock",
        ),
        (
            "main-branch-head-points-at-speculative",
            pricing_reference_source_with_branch_head_pointing_to("main", "commit:rubber-shock"),
            "main",
        ),
    ] {
        let runtime = build_pricing_runtime(source, RecordingSignalBridgeSink::default());
        let error = runtime
            .evaluate(
                BridgeTruthViewEvaluationRequest::for_branch_head(
                    crate::truth_identity_fixtures::truth_branch_fixture(branch),
                )
                .with_read_packet(pricing_component_read_packet("rubber")),
                execution,
            )
            .err()
            .unwrap_or_else(|| panic!("{label} should fail closed under branch-head mutation"));
        assert!(!error.to_string().is_empty());
    }

    let missing_snapshot_runtime = build_pricing_runtime(
        pricing_reference_source_with_missing_branch_head_snapshot(
            "pricing-shock",
            "commit:rubber-shock-missing-snapshot",
            "snapshot:pricing-shock-missing",
            "rubber",
        ),
        RecordingSignalBridgeSink::default(),
    );
    let error = missing_snapshot_runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_branch_head(
                crate::truth_identity_fixtures::truth_branch_fixture("pricing-shock"),
            )
            .with_read_packet(pricing_component_read_packet("rubber")),
            execution,
        )
        .err()
        .expect("missing branch-head snapshot basis should fail closed");

    assert!(!error.to_string().is_empty());
}

#[test]
fn pricing_shock_snapshot_identity_conflict_sweep_is_detectable_against_independent_oracle() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let scenario = generated_pricing_scenario();

    for (label, source, selector_branch, expected_snapshot, unexpected_cost) in [
        (
            "main-snapshot-overwritten-with-speculative-meaning",
            pricing_reference_source_with_conflicting_snapshot_identity(snapshot_with_identity(
                &scenario.speculative_snapshot,
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main"),
            )),
            "main",
            "snapshot:pricing-main",
            scenario.main_rubber_cost,
        ),
        (
            "speculative-snapshot-overwritten-with-main-meaning",
            pricing_reference_source_with_conflicting_snapshot_identity(snapshot_with_identity(
                &scenario.main_snapshot,
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-shock"),
            )),
            "pricing-shock",
            "snapshot:pricing-shock",
            scenario.speculative_rubber_cost,
        ),
    ] {
        let runtime = build_pricing_runtime(source, RecordingSignalBridgeSink::default());
        let evaluation = runtime
            .evaluate(
                BridgeTruthViewEvaluationRequest::for_branch_head(
                    crate::truth_identity_fixtures::truth_branch_fixture(selector_branch),
                )
                .with_read_packet(pricing_component_read_packet("rubber")),
                execution,
            )
            .unwrap_or_else(|_| {
                panic!("{label} should still materialize the overwritten retained snapshot")
            });

        assert_eq!(
            evaluation.snapshot_identity().as_str(),
            expected_snapshot,
            "{label} should expose the conflicting retained snapshot identity"
        );
        assert_ne!(
            read_single_money_cents(&evaluation),
            unexpected_cost,
            "{label} should diverge from the independent oracle for the original branch meaning"
        );
    }
}

#[test]
fn pricing_shock_branch_head_missing_commit_sweep_fails_closed() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    for (label, source, branch, missing_commit) in [
        (
            "main-branch-head-missing-envelope",
            pricing_reference_source_with_missing_branch_head_commit("main", "commit:missing-main"),
            "main",
            "commit:missing-main",
        ),
        (
            "speculative-branch-head-missing-envelope",
            pricing_reference_source_with_missing_branch_head_commit(
                "pricing-shock",
                "commit:missing-speculative",
            ),
            "pricing-shock",
            "commit:missing-speculative",
        ),
    ] {
        let runtime = build_pricing_runtime(source, RecordingSignalBridgeSink::default());
        let error = runtime
            .evaluate(
                BridgeTruthViewEvaluationRequest::for_branch_head(
                    crate::truth_identity_fixtures::truth_branch_fixture(branch),
                )
                .with_read_packet(pricing_component_read_packet("rubber")),
                execution,
            )
            .err()
            .unwrap_or_else(|| {
                panic!("{label} should fail closed when branch head commit is missing")
            });

        let error_text = error.to_string();
        assert!(
            error_text.contains(missing_commit),
            "{label} should mention the missing retained branch-head commit"
        );
    }
}
