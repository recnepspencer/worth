use super::support::*;

#[test]
fn pricing_shock_standard_path_routes_and_evaluates_main() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let truth = RelationalPricing::new(execution, 100, 40, 56);
    let sink = RecordingSignalBridgeSink::default();
    let runtime = truth.runtime(sink.clone());

    let steel_route = runtime
        .route(truth.steel_commit.clone(), execution)
        .expect("steel pricing route should succeed");
    let steel_eval = runtime
        .evaluate_current(steel_route.target(), execution)
        .expect("steel route should prepare signal evaluation");
    let branch_eval = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_branch_head(
                TruthBranchIdentity::from_relational_branch_id("main"),
            ),
            execution,
        )
        .expect("main pricing branch-head evaluation should succeed");

    assert_eq!(
        steel_route.result().receipt().delivered_target_count(),
        2,
        "shared steel cost should fan out to multiple product price invalidations"
    );
    assert_eq!(
        steel_route.result().receipt().snapshot_identity().as_str(),
        truth.steel_snapshot().as_str()
    );
    assert_eq!(
        steel_eval.snapshot().snapshot_identity().as_str(),
        truth.steel_snapshot().as_str()
    );
    assert_eq!(
        branch_eval.snapshot_identity().as_str(),
        truth.main_snapshot().as_str()
    );
    let diagnostics = runtime.diagnostics();
    let delivered_targets = diagnostics
        .route_records()
        .last()
        .expect("steel route should produce a diagnostics record")
        .invalidation_targets()
        .iter()
        .map(|target| target.signal_scope().to_owned())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        delivered_targets,
        std::collections::BTreeSet::from([
            "price:bicycle".to_owned(),
            "price:wheelbarrow".to_owned()
        ])
    );
    assert_eq!(sink.deliveries().len(), 1);
}

#[test]
fn pricing_shock_split_screen_keeps_main_and_speculative_truth_isolated() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let scenario = generated_pricing_scenario();
    let truth = RelationalPricing::new(
        execution,
        scenario.main_steel_cost,
        scenario.main_rubber_cost,
        scenario.speculative_rubber_cost,
    );
    let sink = RecordingSignalBridgeSink::default();
    let runtime = truth.runtime(sink);
    let session = truth.begin_preview(
        &runtime,
        BridgePreviewSessionIdentity::from_stable_name("pricing:preview-compare"),
    );
    let (main_eval, speculative_eval) = session
        .compare_to_main(
            TruthBranchIdentity::from_relational_branch_id("main"),
            truth.rubber_read_packet(),
            execution,
        )
        .expect("both bound branches should evaluate");
    let live_main_route = runtime
        .route(truth.steel_commit.clone(), execution)
        .expect("main branch routing should remain live while speculation is open");

    let main_rubber_cost = read_cost(&main_eval, execution);
    let speculative_rubber_cost = read_cost(&speculative_eval, execution);

    assert_eq!(
        session.branches().relational().relational_branch_id(),
        Some("pricing-shock")
    );
    assert_eq!(session.branches().signal().as_str(), "signal:pricing-shock");
    assert_eq!(
        main_eval.snapshot_identity().as_str(),
        truth.main_snapshot().as_str()
    );
    assert_eq!(
        speculative_eval.snapshot_identity().as_str(),
        truth.shock_snapshot().as_str()
    );
    let mismatched = runtime.begin_preview(
        BridgePreviewSessionIdentity::from_stable_name("pricing:preview-mismatch"),
        session.branches().clone(),
        truth.main_snapshot().clone(),
    );
    let denial = mismatched
        .compare_to_main(
            TruthBranchIdentity::from_relational_branch_id("main"),
            truth.rubber_read_packet(),
            execution,
        )
        .err()
        .expect("a main snapshot cannot stand in for the preview branch");
    assert_eq!(
        denial.kind(),
        BridgeDeliveryErrorKind::HistoricalBranchMismatch
    );
    assert_eq!(main_rubber_cost, scenario.main_rubber_cost.to_string());
    assert_eq!(
        speculative_rubber_cost,
        scenario.speculative_rubber_cost.to_string()
    );
    assert_eq!(
        live_main_route
            .result()
            .receipt()
            .snapshot_identity()
            .as_str(),
        truth.steel_snapshot().as_str()
    );
    assert_eq!(
        live_main_route.result().receipt().delivered_target_count(),
        2
    );
}

#[test]
fn pricing_shock_generated_commit_attribution_exposes_stream_and_product_criteria() {
    let scenario = generated_pricing_scenario();
    let shock = scenario
        .commit_attributions
        .get("commit:rubber-shock")
        .expect("generated scenario should retain shock attribution");

    assert_eq!(
        shock.snapshot_identity,
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-shock")
    );
    assert_eq!(
        shock.branch_identity,
        crate::truth_identity_fixtures::truth_branch_fixture("pricing-shock")
    );
    assert_eq!(shock.material, PricingMaterial::Rubber);
    assert_eq!(shock.shock_multiplier_per_mille, 4000);
    assert_eq!(
        shock.shock_delta_microunits,
        scenario.speculative_rubber_cost - scenario.main_rubber_cost
    );
    assert_eq!(
        shock.material_attribution.current_value_microunits,
        scenario.main_rubber_cost
    );
    assert_eq!(shock.representative_product.sku, "scooter-001");
    assert!(shock.representative_product.material_cost_cents > 0);
    assert!(shock.representative_product.shipping_cost_cents > 0);
    assert!(shock
        .representative_product
        .material_contributions_cents
        .iter()
        .any(|(material, cents)| *material == PricingMaterial::Rubber && *cents > 0));
    assert_ne!(shock.material_attribution.external_factor_microunits, 0);
    assert!(
        shock.material_attribution.factor_delta_microunits != 0
            || shock.material_attribution.trend_delta_microunits != 0
            || shock.material_attribution.idiosyncratic_noise_microunits != 0
            || shock.material_attribution.jump_delta_microunits != 0
    );
}

#[test]
fn historical_commit_reads_earlier_price_after_main_advances_twice() {
    let serial = crate::snapshot::test_serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let mut truth = RelationalPricing::new(execution, 100, 40, 56);
    let main = TruthBranchIdentity::from_relational_branch_id("main");
    let historical = BridgeTruthViewEvaluationRequest::for_historical_commit(
        main.clone(),
        truth.rubber_commit.clone(),
    )
    .with_read_packet(truth.rubber_read_packet());
    let source = SourceDeclaration::new(
        SourceDeclarationIdentity::from_stable_name("historical-rubber"),
        historical.selector().clone(),
        BridgeSourceCapabilitySet::new(vec![
            BridgeSourceCapability::HistoricalRead,
            BridgeSourceCapability::SnapshotRead,
        ]),
    );
    let runtime = truth.runtime_with_historical_source(source.clone());
    truth.publish_main_cost(execution, PricingMaterial::Rubber, 52);
    let planned = runtime
        .plan_truth_view_packet(
            historical.declaration(),
            historical.read_packet(),
            execution,
        )
        .unwrap();
    truth.publish_main_cost(execution, PricingMaterial::Rubber, 63);

    let observation = runtime
        .materialize_truth_view_observation(planned, execution)
        .expect("the plan retains the earlier committed root");
    let record = runtime.canonicalize_historical_evaluation_record(&observation);
    assert_eq!(
        record.decision_log().materialization_path(),
        crate::facade::BridgeHistoricalMaterializationPath::PinnedCommitObservation,
    );
    let earlier = crate::facade::BridgeTruthViewEvaluation::new(observation, record.clone());
    let latest = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_branch_head(main)
                .with_read_packet(truth.rubber_read_packet()),
            execution,
        )
        .unwrap();
    assert_eq!(read_cost(&earlier, execution), "40");
    assert_eq!(read_cost(&latest, execution), "63");
    let contract = runtime
        .admit_source(source)
        .expect("Native supports historical reads");
    let source_read = runtime
        .materialize_source_packet(&contract, truth.rubber_read_packet(), execution)
        .expect("source materialization must also open the earlier root");
    let canonical_source = runtime.canonicalize_historical_evaluation_record(&source_read);
    let source_eval = crate::facade::BridgeTruthViewEvaluation::new(source_read, canonical_source);
    assert_eq!(read_cost(&source_eval, execution), "40");

    assert_eq!(
        runtime
            .replay_canonical_historical_evaluation_record(&record, execution)
            .unwrap()
            .snapshot_identity(),
        earlier.snapshot_identity(),
    );
}

#[test]
fn historical_commit_refuses_unknown_commit_and_cancelled_materialization() {
    use crate::facade::{BridgeExecutionDenial, RelationalBridgeSourceErrorTag};
    use worth_relational::facade::change_source::RelationalCommitSelectionDenial;
    let serial = crate::snapshot::test_serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let truth = RelationalPricing::new(execution, 100, 40, 56);
    let runtime = truth.runtime(RecordingSignalBridgeSink::default());
    let main = TruthBranchIdentity::from_relational_branch_id("main");
    let unknown = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_historical_commit(
                main.clone(),
                TruthCommitIdentity::from_relational_commit_id(u64::MAX),
            ),
            execution,
        )
        .err()
        .expect("unknown commit must refuse");
    assert!(matches!(
        unknown.kind(),
        BridgeDeliveryErrorKind::HistoricalTruthViewUnavailable(source)
            if matches!(source.kind(), RelationalBridgeSourceErrorTag::CommitSelection(
                RelationalCommitSelectionDenial::RequestedCommitUnavailable { .. }
            ))
    ));

    let request =
        BridgeTruthViewEvaluationRequest::for_historical_commit(main, truth.rubber_commit.clone());
    let planned = runtime
        .plan_truth_view_packet(request.declaration(), request.read_packet(), execution)
        .unwrap();
    let cancel = worth_execution::CancellationSource::new();
    let stopped = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(cancel.token(), None);
    cancel.cancel();
    let stopped = worth_execution::ExecutionRequest::serial(&stopped);
    let error = runtime
        .materialize_truth_view_observation(planned, stopped)
        .err()
        .expect("cancelled plan must not materialize");
    assert_eq!(
        error.kind(),
        BridgeDeliveryErrorKind::ExecutionDenied(BridgeExecutionDenial::Cancelled)
    );
}
