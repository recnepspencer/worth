use super::*;

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_discard_bundle(
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingDiscardBundle {
    let scenario = generated_pricing_scenario();
    let source = pricing_reference_source();
    let runtime = build_pricing_runtime(source.clone(), RecordingSignalBridgeSink::default());
    let discard_session_identity =
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-discard-churn");
    let session = runtime
        .speculate(BridgeSpeculativeSessionRequest::new(
            discard_session_identity.clone(),
            pricing_preview_declaration(),
            4,
            2,
            2,
        ))
        .expect("discard churn preview should activate");
    let comparison = session.compare_to_main();

    source.insert_committed_patch(pricing_patch(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main-live"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch:steel-main-live"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main-live"),
        ),
        "steel",
    ));
    source.insert_snapshot(scenario.live_main_snapshot);

    let live_main_route = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main-live"),
            execution,
        )
        .expect("main branch should keep routing during speculative churn");
    let speculative_eval = runtime
        .evaluate(
            comparison
                .speculative_evaluation_request()
                .with_read_packet(pricing_component_read_packet("rubber")),
            execution,
        )
        .expect("speculative branch should still see shock pricing");

    let discarded = session
        .discard(vec![
            BridgePreviewResidueClass::PreviewExecutionRetained,
            BridgePreviewResidueClass::ReplayRetainedNonAuthoritative,
            BridgePreviewResidueClass::TemporaryDiagnosticsResidue,
        ])
        .expect("discard should succeed with zero authoritative residue");

    let post_discard_main_eval = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_branch_head(
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
            )
            .with_read_packet(pricing_component_read_packet("steel")),
            execution,
        )
        .expect("main branch should still evaluate after discard");
    let replay_bundle = runtime
        .replay_preview_bundle(&discard_session_identity)
        .expect("discard replay bundle should be retained");

    PricingDiscardBundle {
        live_main_snapshot: live_main_route
            .result()
            .receipt()
            .snapshot_identity()
            .clone(),
        speculative_rubber_cost_cents: read_single_money_cents(&speculative_eval),
        post_discard_main_snapshot: post_discard_main_eval.snapshot_identity().clone(),
        post_discard_main_steel_cost_cents: read_single_money_cents(&post_discard_main_eval),
        lifecycle_state: discarded.session().lifecycle_state_kind(),
        discard_record_count: runtime.diagnostics().preview_discard_records().len(),
        promotion_record_count: runtime.diagnostics().preview_promotion_records().len(),
        replay_outcome: replay_bundle.lifecycle_outcome(),
        has_discard_record: replay_bundle.preview_discard_record().is_some(),
        has_promotion_record: replay_bundle.preview_promotion_record().is_some(),
    }
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_promotion_bundle(
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingPromotionBundle {
    let scenario = generated_pricing_scenario();
    let source = pricing_reference_source();
    let runtime = build_pricing_runtime(source.clone(), RecordingSignalBridgeSink::default());
    let promotion_session_identity =
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-promote-churn");
    let session = runtime
        .speculate(BridgeSpeculativeSessionRequest::new(
            promotion_session_identity.clone(),
            pricing_preview_declaration(),
            4,
            2,
            2,
        ))
        .expect("promotion churn preview should activate");
    let comparison = session.compare_to_main();

    source.insert_committed_patch(pricing_patch(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-main-interleaved"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch:rubber-main-interleaved"),
            crate::truth_identity_fixtures::truth_snapshot_fixture(
                "snapshot:pricing-main-interleaved",
            ),
        ),
        "rubber",
    ));
    source.insert_snapshot(scenario.interleaved_main_snapshot);

    let main_eval = runtime
        .evaluate(
            comparison
                .main_evaluation_request(crate::truth_identity_fixtures::truth_branch_fixture(
                    "main",
                ))
                .with_read_packet(pricing_component_read_packet("rubber")),
            execution,
        )
        .expect("interleaved main branch should remain independently readable");
    let speculative_eval = runtime
        .evaluate(
            comparison
                .speculative_evaluation_request()
                .with_read_packet(pricing_component_read_packet("rubber")),
            execution,
        )
        .expect("speculative branch should keep its isolated shock view");

    let promoted = session
        .promote()
        .expect("promotion should succeed after interleaved main churn");
    let replay_bundle = runtime
        .replay_preview_bundle(&promotion_session_identity)
        .expect("promotion replay bundle should be retained");
    let promotion_record = replay_bundle
        .preview_promotion_record()
        .expect("promotion replay bundle should retain the promotion record");

    PricingPromotionBundle {
        main_snapshot: main_eval.snapshot_identity().clone(),
        speculative_snapshot: speculative_eval.snapshot_identity().clone(),
        main_rubber_cost_cents: read_single_money_cents(&main_eval),
        speculative_rubber_cost_cents: read_single_money_cents(&speculative_eval),
        lifecycle_state: promoted.session().lifecycle_state_kind(),
        promotion_session_identity: BridgePreviewSessionIdentity::admit_bridge_owned(
            promotion_record.preview_session_identity(),
        ),
        authoritative_commit_boundary_digest: promotion_record
            .authoritative_commit_boundary_digest()
            .to_owned(),
        authoritative_artifact_digest: promotion_record.authoritative_artifact_digest().to_owned(),
        replay_outcome: replay_bundle.lifecycle_outcome(),
        has_promotion_explanation: matches!(
            runtime.diagnostics().explain_session(
                &BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-promote-churn",)
            ),
            Some(crate::facade::BridgeStandardSessionExplanation::PreviewPromotion(_))
        ),
    }
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_fanout_bundle(
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingFanoutBundle {
    let scenario = generated_pricing_scenario();
    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(pricing_patch(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-fanout-a"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch:steel-fanout-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-fanout-a"),
        ),
        "steel",
    ));
    source.insert_snapshot(scenario.fanout_first_snapshot);

    let sink = RecordingSignalBridgeSink::default();
    let runtime = build_high_fanout_pricing_runtime(source.clone(), sink.clone(), 100);

    let first_route = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-fanout-a"),
            execution,
        )
        .expect("first steel fanout route should succeed");

    source.insert_committed_patch(pricing_patch(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-fanout-b"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch:steel-fanout-b"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-fanout-b"),
        ),
        "steel",
    ));
    source.insert_snapshot(scenario.fanout_second_snapshot);

    let second_route = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-fanout-b"),
            execution,
        )
        .expect("second steel fanout route should succeed");
    let second_eval = runtime
        .evaluate_current(second_route.target(), execution)
        .expect("second steel fanout route should prepare evaluation");
    let branch_eval = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_branch_head(
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
            )
            .with_read_packet(pricing_component_read_packet("steel")),
            execution,
        )
        .expect("main branch should evaluate after repeated steel churn");

    let route_records = runtime.diagnostics().route_records();
    let last_record = route_records
        .last()
        .expect("repeated steel churn should retain the last route record");
    let mut last_targets = last_record
        .invalidation_targets()
        .iter()
        .map(|target| target.signal_scope().to_owned())
        .collect::<Vec<_>>();
    last_targets.sort();

    PricingFanoutBundle {
        total_deliveries: sink.deliveries().len(),
        first_delivery_target_count: first_route.result().receipt().delivered_target_count(),
        second_delivery_target_count: second_route.result().receipt().delivered_target_count(),
        second_source_commit: crate::truth_identity_fixtures::truth_commit_fixture(
            "commit:steel-fanout-b",
        ),
        second_snapshot: second_eval.snapshot().snapshot_identity().clone(),
        branch_snapshot: branch_eval.snapshot_identity().clone(),
        branch_steel_cost_cents: read_single_money_cents(&branch_eval),
        retained_target_count: last_targets.len(),
        first_target: last_targets.first().cloned().unwrap_or_default(),
        last_target: last_targets.last().cloned().unwrap_or_default(),
    }
}
