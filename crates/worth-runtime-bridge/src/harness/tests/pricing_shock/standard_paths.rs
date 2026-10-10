use super::support::*;

#[test]
fn pricing_shock_standard_path_routes_evaluates_and_keeps_speculation_local() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(pricing_patch(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch:steel-main"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main"),
        ),
        "steel",
    ));
    source.insert_committed_patch(pricing_patch(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-main"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch:rubber-main"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main"),
        ),
        "rubber",
    ));
    source.insert_snapshot(pricing_snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main"),
        "100",
        "40",
    ));

    let sink = RecordingSignalBridgeSink::default();
    let runtime = build_pricing_runtime(source.clone(), sink.clone());

    let steel_route = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main"),
            execution,
        )
        .expect("steel pricing route should succeed");
    let steel_eval = runtime
        .evaluate_current(steel_route.target(), execution)
        .expect("steel route should prepare signal evaluation");
    let branch_eval = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_branch_head(
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
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
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main").as_str()
    );
    assert_eq!(
        steel_eval.snapshot().snapshot_identity().as_str(),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main").as_str()
    );
    assert_eq!(
        branch_eval.snapshot_identity().as_str(),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main").as_str()
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

    let discarded = runtime
        .speculate(BridgeSpeculativeSessionRequest::new(
            BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-discard"),
            pricing_preview_declaration(),
            4,
            2,
            2,
        ))
        .expect("pricing shock preview should activate")
        .discard(vec![
            BridgePreviewResidueClass::PreviewExecutionRetained,
            BridgePreviewResidueClass::ReplayRetainedNonAuthoritative,
            BridgePreviewResidueClass::TemporaryDiagnosticsResidue,
        ])
        .expect("pricing shock discard should succeed");

    let promoted = runtime
        .speculate(BridgeSpeculativeSessionRequest::new(
            BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-promote"),
            pricing_preview_declaration(),
            4,
            2,
            2,
        ))
        .expect("pricing shock promotion preview should activate")
        .promote()
        .expect("pricing shock promotion should succeed");

    assert_eq!(
        runtime.diagnostics().preview_discard_records().len(),
        1,
        "discard should stay isolated and queryable"
    );
    assert_eq!(
        runtime.diagnostics().preview_promotion_records().len(),
        1,
        "promotion should stay isolated and queryable"
    );
    assert!(matches!(
        runtime
            .diagnostics()
            .explain_session(&BridgePreviewSessionIdentity::admit_bridge_owned(
                "pricing:preview-promote"
            )),
        Some(crate::facade::BridgeStandardSessionExplanation::PreviewPromotion(_))
    ));
    assert_eq!(
        discarded.session().session_identity().as_str(),
        "pricing:preview-discard"
    );
    assert_eq!(
        promoted.session().session_identity().as_str(),
        "pricing:preview-promote"
    );
}

#[test]
fn pricing_shock_split_screen_keeps_main_and_speculative_truth_isolated() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let scenario = generated_pricing_scenario();
    let source = pricing_reference_source();
    let sink = RecordingSignalBridgeSink::default();
    let runtime = build_pricing_runtime(source, sink);
    let comparison = runtime
        .speculate(BridgeSpeculativeSessionRequest::new(
            BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-compare"),
            pricing_preview_declaration(),
            4,
            2,
            2,
        ))
        .expect("pricing shock comparison preview should activate")
        .compare_to_main();

    let rubber_read = pricing_component_read_packet("rubber");
    let main_eval = runtime
        .evaluate(
            comparison
                .main_evaluation_request(crate::truth_identity_fixtures::truth_branch_fixture(
                    "main",
                ))
                .with_read_packet(rubber_read.clone()),
            execution,
        )
        .expect("main branch should evaluate against its retained snapshot");
    let speculative_eval = runtime
        .evaluate(
            comparison
                .speculative_evaluation_request()
                .with_read_packet(rubber_read),
            execution,
        )
        .expect("speculative branch should evaluate against its isolated snapshot");
    let live_main_route = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main"),
            execution,
        )
        .expect("main branch routing should remain live while speculation is open");

    let main_rubber_cost = read_single_aspect_value_text(&main_eval);
    let speculative_rubber_cost = read_single_aspect_value_text(&speculative_eval);

    assert_eq!(
        comparison.truth_branch_identity().as_str(),
        crate::truth_identity_fixtures::truth_branch_fixture("pricing-shock").as_str()
    );
    assert_eq!(
        comparison.signal_branch_identity().as_str(),
        "signal:pricing-shock"
    );
    assert_eq!(
        main_eval.snapshot_identity().as_str(),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main").as_str()
    );
    assert_eq!(
        speculative_eval.snapshot_identity().as_str(),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-shock").as_str()
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
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main").as_str()
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
