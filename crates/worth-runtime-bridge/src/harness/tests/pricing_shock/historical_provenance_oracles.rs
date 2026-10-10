use super::support::*;

#[test]
fn pricing_shock_historical_commit_reads_bridge_visible_provenance_from_truth() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let scenario = generated_pricing_scenario();
    let runtime = build_pricing_runtime(
        pricing_reference_source(),
        RecordingSignalBridgeSink::default(),
    );

    let historical = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_historical_commit(
                crate::truth_identity_fixtures::truth_branch_fixture("pricing-shock"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-shock"),
            )
            .with_read_packet(pricing_provenance_read_packet("rubber")),
            execution,
        )
        .expect("historical pricing shock provenance should materialize");
    let provenance_texts = read_pricing_provenance_aspect_text_packet(&historical);
    let shock = scenario
        .commit_attributions
        .get("commit:rubber-shock")
        .expect("generated scenario should retain shock attribution");

    assert_eq!(
        historical.snapshot_identity().as_str(),
        "snapshot:pricing-shock"
    );
    assert_eq!(
        provenance_texts.regime_text(),
        format!("{:?}", shock.material_attribution.regime)
    );
    assert_eq!(
        provenance_texts.external_factor_text(),
        shock
            .material_attribution
            .external_factor_microunits
            .to_string()
    );
    assert_eq!(
        provenance_texts.factor_delta_text(),
        shock
            .material_attribution
            .factor_delta_microunits
            .to_string()
    );
    assert_eq!(
        provenance_texts.trend_delta_text(),
        shock
            .material_attribution
            .trend_delta_microunits
            .to_string()
    );
    assert_eq!(
        provenance_texts.jump_delta_text(),
        shock.material_attribution.jump_delta_microunits.to_string()
    );
    assert_eq!(
        provenance_texts.shock_delta_text(),
        shock.shock_delta_microunits.to_string()
    );
    assert_eq!(
        provenance_texts.shock_multiplier_text(),
        shock.shock_multiplier_per_mille.to_string()
    );
}

#[test]
fn pricing_shock_historical_provenance_corruption_is_detectable_against_independent_oracle() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let scenario = generated_pricing_scenario();
    let runtime = build_pricing_runtime(
        pricing_reference_source_with_corrupted_shock_provenance("shock-delta", "999999"),
        RecordingSignalBridgeSink::default(),
    );

    let provenance_eval = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_historical_commit(
                crate::truth_identity_fixtures::truth_branch_fixture("pricing-shock"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-shock"),
            )
            .with_read_packet(pricing_provenance_read_packet("rubber")),
            execution,
        )
        .expect("corrupted historical provenance should still materialize as truth");
    let cost_eval = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_historical_commit(
                crate::truth_identity_fixtures::truth_branch_fixture("pricing-shock"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-shock"),
            )
            .with_read_packet(pricing_component_read_packet("rubber")),
            execution,
        )
        .expect("historical component cost should still materialize");
    let provenance_texts = read_pricing_provenance_aspect_text_packet(&provenance_eval);
    let shock = scenario
        .commit_attributions
        .get("commit:rubber-shock")
        .expect("generated scenario should retain shock attribution");

    assert_eq!(
        provenance_eval.snapshot_identity().as_str(),
        "snapshot:pricing-shock"
    );
    assert_eq!(
        read_single_money_cents(&cost_eval),
        scenario.speculative_rubber_cost
    );
    assert_eq!(
        provenance_texts.regime_text(),
        format!("{:?}", shock.material_attribution.regime)
    );
    assert_eq!(
        provenance_texts.external_factor_text(),
        shock
            .material_attribution
            .external_factor_microunits
            .to_string()
    );
    assert_ne!(
        provenance_texts.shock_delta_text(),
        shock.shock_delta_microunits.to_string()
    );
    assert_eq!(provenance_texts.shock_delta_text(), "999999");
    assert_eq!(
        provenance_texts.shock_multiplier_text(),
        shock.shock_multiplier_per_mille.to_string()
    );
}

#[test]
fn pricing_shock_provenance_mutation_sweep_is_detectable_against_independent_oracle() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let scenario = generated_pricing_scenario();
    let shock = scenario
        .commit_attributions
        .get("commit:rubber-shock")
        .expect("generated scenario should retain shock attribution");

    for (field, expected_aspect_value_text, corrupted_aspect_value_text) in [
        (
            "external-factor",
            shock
                .material_attribution
                .external_factor_microunits
                .to_string(),
            "444444".to_owned(),
        ),
        (
            "factor-delta",
            shock
                .material_attribution
                .factor_delta_microunits
                .to_string(),
            "555555".to_owned(),
        ),
        (
            "trend-delta",
            shock
                .material_attribution
                .trend_delta_microunits
                .to_string(),
            "666666".to_owned(),
        ),
        (
            "jump-delta",
            shock.material_attribution.jump_delta_microunits.to_string(),
            "777777".to_owned(),
        ),
        (
            "shock-delta",
            shock.shock_delta_microunits.to_string(),
            "888888".to_owned(),
        ),
        (
            "shock-multiplier",
            shock.shock_multiplier_per_mille.to_string(),
            "999999".to_owned(),
        ),
    ] {
        let runtime = build_pricing_runtime(
            pricing_reference_source_with_corrupted_shock_provenance(
                field,
                corrupted_aspect_value_text.clone(),
            ),
            RecordingSignalBridgeSink::default(),
        );
        let historical = runtime
            .evaluate(
                BridgeTruthViewEvaluationRequest::for_historical_commit(
                    crate::truth_identity_fixtures::truth_branch_fixture("pricing-shock"),
                    crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-shock"),
                )
                .with_read_packet(pricing_provenance_read_packet("rubber")),
                execution,
            )
            .expect("corrupted provenance field should still materialize");
        let provenance_texts = read_pricing_provenance_aspect_text_packet(&historical);

        assert_eq!(
            historical.snapshot_identity().as_str(),
            "snapshot:pricing-shock"
        );
        assert_eq!(
            provenance_texts.field_text(field),
            corrupted_aspect_value_text
        );
        assert_ne!(
            provenance_texts.field_text(field),
            expected_aspect_value_text
        );
    }
}
