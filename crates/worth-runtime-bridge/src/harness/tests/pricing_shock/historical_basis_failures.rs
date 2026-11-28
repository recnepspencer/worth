use super::support::*;

#[test]
fn pricing_shock_conflicting_historical_basis_is_detectable_against_independent_oracle() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let scenario = generated_pricing_scenario();
    let truth = RelationalPricing::reference(execution, &scenario, None);
    let runtime = truth.fault_runtime(
        PricingFault::CommitSnapshot(truth.shock_commit.clone(), truth.main_snapshot().clone()),
        BridgeRuntimePolicy::development(),
    );

    let historical_cost = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_historical_commit(
                TruthBranchIdentity::from_relational_branch_id("pricing-shock"),
                truth.shock_commit.clone(),
            )
            .with_read_packet(truth.rubber_read_packet()),
            execution,
        )
        .expect("conflicting historical basis should still materialize as retained truth");
    let historical_provenance = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_historical_commit(
                TruthBranchIdentity::from_relational_branch_id("pricing-shock"),
                truth.shock_commit.clone(),
            )
            .with_read_packet(truth.rubber_provenance_packet()),
            execution,
        )
        .expect("conflicting historical basis should materialize provenance packet");
    let provenance_texts = read_pricing_provenance_aspect_text_packet(&historical_provenance);

    assert_eq!(
        historical_cost.snapshot_identity().canonical_label(),
        truth.main_snapshot().canonical_label()
    );
    assert_eq!(
        historical_provenance.snapshot_identity().canonical_label(),
        truth.main_snapshot().canonical_label()
    );
    assert_eq!(
        read_cost(&historical_cost, execution)
            .parse::<i64>()
            .expect("native pricing cost"),
        scenario.main_rubber_cost
    );
    assert_ne!(
        read_cost(&historical_cost, execution)
            .parse::<i64>()
            .expect("native pricing cost"),
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

    let scenario = generated_pricing_scenario();
    let truth = RelationalPricing::reference(execution, &scenario, None);
    for (branch, other) in [("pricing-shock", "main"), ("main", "pricing-shock")] {
        let branch = TruthBranchIdentity::from_relational_branch_id(branch);
        let runtime = truth.fault_runtime(
            PricingFault::SwapHead(
                branch.clone(),
                TruthBranchIdentity::from_relational_branch_id(other),
            ),
            BridgeRuntimePolicy::development(),
        );
        let error = runtime
            .evaluate(
                BridgeTruthViewEvaluationRequest::for_branch_head(branch)
                    .with_read_packet(truth.rubber_read_packet()),
                execution,
            )
            .err()
            .expect("a branch-head envelope from another branch should fail closed");
        assert!(!error.to_string().is_empty());
    }

    let missing_snapshot_runtime = truth.fault_runtime(
        PricingFault::UnavailableSnapshot(truth.shock_snapshot().clone()),
        BridgeRuntimePolicy::development(),
    );
    let error = missing_snapshot_runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_branch_head(
                TruthBranchIdentity::from_relational_branch_id("pricing-shock"),
            )
            .with_read_packet(truth.rubber_read_packet()),
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

    let truth = RelationalPricing::reference(execution, &scenario, None);

    for (selector_branch, expected_snapshot, other_snapshot, unexpected_cost) in [
        (
            "main",
            truth.main_snapshot(),
            truth.shock_snapshot(),
            scenario.main_rubber_cost,
        ),
        (
            "pricing-shock",
            truth.shock_snapshot(),
            truth.main_snapshot(),
            scenario.speculative_rubber_cost,
        ),
    ] {
        let runtime = truth.fault_runtime(
            PricingFault::SwapSnapshotContents(expected_snapshot.clone(), other_snapshot.clone()),
            BridgeRuntimePolicy::development(),
        );
        let evaluation = runtime
            .evaluate(
                BridgeTruthViewEvaluationRequest::for_branch_head(
                    TruthBranchIdentity::from_relational_branch_id(selector_branch),
                )
                .with_read_packet(truth.rubber_read_packet()),
                execution,
            )
            .unwrap_or_else(|_| {
                panic!(
                    "{selector_branch} should still materialize the overwritten retained snapshot"
                )
            });

        assert_eq!(
            evaluation.snapshot_identity(),
            expected_snapshot,
            "{selector_branch} should expose the conflicting retained snapshot identity"
        );
        assert_ne!(
            read_cost(&evaluation, execution).parse::<i64>().expect("native pricing cost"),
            unexpected_cost,
            "{selector_branch} should diverge from the independent oracle for the original branch meaning"
        );
    }
}

#[test]
fn pricing_shock_branch_head_missing_commit_sweep_fails_closed() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let scenario = generated_pricing_scenario();
    let truth = RelationalPricing::reference(execution, &scenario, None);
    for (branch, missing_commit) in [
        ("main", "commit:missing-main"),
        ("pricing-shock", "commit:missing-speculative"),
    ] {
        let runtime = truth.fault_runtime(
            PricingFault::MissingHead(
                TruthBranchIdentity::from_relational_branch_id(branch),
                missing_commit.to_owned(),
            ),
            BridgeRuntimePolicy::development(),
        );
        let error = runtime
            .evaluate(
                BridgeTruthViewEvaluationRequest::for_branch_head(
                    TruthBranchIdentity::from_relational_branch_id(branch),
                )
                .with_read_packet(truth.rubber_read_packet()),
                execution,
            )
            .err()
            .unwrap_or_else(|| {
                panic!("{branch} should fail closed when branch head commit is missing")
            });

        let error_text = error.to_string();
        assert!(
            error_text.contains(missing_commit),
            "{branch} should mention the missing retained branch-head commit"
        );
    }
}
