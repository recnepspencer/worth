use super::*;

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_reference_bundle(
    runtime: &RuntimeBridge,
    preview_session_identity: BridgePreviewSessionIdentity,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingReferenceBundle {
    let route = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main"),
            execution,
        )
        .expect("pricing reference route should succeed");
    let route_record = runtime
        .diagnostics()
        .route_record_for_source_commit(route.result().result_summary().source_commit())
        .expect("pricing reference route should retain its route record");
    let main_eval = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_branch_head(
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
            )
            .with_read_packet(pricing_component_read_packet("rubber")),
            execution,
        )
        .expect("pricing main evaluation should succeed");
    let comparison = runtime
        .speculate(BridgeSpeculativeSessionRequest::new(
            preview_session_identity,
            pricing_preview_declaration(),
            4,
            2,
            2,
        ))
        .expect("pricing reference preview should activate")
        .compare_to_main();
    let speculative_eval = runtime
        .evaluate(
            comparison
                .speculative_evaluation_request()
                .with_read_packet(pricing_component_read_packet("rubber")),
            execution,
        )
        .expect("pricing speculative evaluation should succeed");

    PricingReferenceBundle {
        source_branch: route_record.source_branch().clone(),
        source_commit: route_record.source_commit().clone(),
        route_snapshot: route.result().receipt().snapshot_identity().clone(),
        delivered_target_count: route.result().receipt().delivered_target_count(),
        route_entry_count: route_record.entries().len(),
        evaluation_record_identity: main_eval.record().record_identity().clone(),
        evaluation_selector_identity: main_eval
            .record()
            .decision_log()
            .selector_identity()
            .clone(),
        main_snapshot: main_eval.snapshot_identity().clone(),
        main_rubber_cost_cents: read_single_money_cents(&main_eval),
        speculative_truth_branch: comparison.truth_branch_identity().clone(),
        speculative_signal_branch: comparison.signal_branch_identity().clone(),
        speculative_snapshot: speculative_eval.snapshot_identity().clone(),
        speculative_rubber_cost_cents: read_single_money_cents(&speculative_eval),
    }
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_aspect_bundle(
    policy: BridgeRuntimePolicy,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingAspectBundle {
    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(pricing_patch(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-aspect"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch:steel-aspect"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-aspect"),
        ),
        "steel",
    ));
    source.insert_snapshot(pricing_aspect_snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-aspect"),
        "145",
        "40",
    ));

    let runtime =
        build_pricing_runtime_with_aspects(source, RecordingSignalBridgeSink::default(), policy);
    let route = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-aspect"),
            execution,
        )
        .expect("aspect-aware pricing route should succeed");

    let route_record = runtime
        .diagnostics()
        .route_record_for_source_commit(route.result().result_summary().source_commit())
        .expect("aspect-aware pricing route should retain a route record");
    let explanation = runtime
        .diagnostics()
        .explain_route(route_record.route_identity())
        .expect("aspect-aware pricing route should be explainable");
    let entry = &explanation.route_entries()[0];

    PricingAspectBundle {
        route_identity: explanation.route_identity().clone(),
        snapshot: explanation.snapshot_identity().clone(),
        source_branch: route_record.source_branch().clone(),
        source_commit: route_record.source_commit().clone(),
        truth_surface_kind: entry.truth_surface_kind(),
        fine_grained_match_status: entry.fine_grained_match_status(),
        aspect_registration_id: entry
            .aspect_registration_id()
            .expect("aspect-aware route entry should retain the aspect registration id")
            .clone(),
        subscription_slice_kind: entry
            .subscription_slice_kind()
            .expect("aspect-aware route entry should retain the subscription slice kind")
            .clone(),
        target_canonical_basis: entry.target_canonical_basis().to_owned(),
        invalidation_target: explanation.invalidation_targets()[0]
            .signal_scope()
            .to_owned(),
    }
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_missing_snapshot_failure_bundle(
    runtime: &RuntimeBridge,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingFailureBundle {
    let error = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-missing-snapshot"),
            execution,
        )
        .expect_err("pricing route should fail when the source snapshot is absent");
    let error_kind = match error {
        BridgeStandardRouteError::Delivery(error) => error.kind(),
        BridgeStandardRouteError::Route(error) => {
            panic!("missing snapshot should fail at delivery, not route planning: {error}")
        }
    };

    let retained_failure = runtime
        .diagnostics()
        .last_failure_record()
        .expect("pricing failure should be retained in diagnostics");

    PricingFailureBundle {
        error_kind,
        failure_class: retained_failure.failure_class().clone(),
        source_commit: retained_failure.source_commit().clone(),
        source_snapshot: retained_failure.source_snapshot().clone(),
    }
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_replay_bundle(
    runtime: &RuntimeBridge,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingReplayBundle {
    runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main"),
            execution,
        )
        .expect("pricing replay control route should succeed");
    let canonical_record = runtime
        .diagnostics()
        .last_canonical_route_record()
        .expect("pricing route should retain a canonical replay record");
    let replay = runtime
        .replay_canonical_record(&canonical_record, execution)
        .expect("pricing route replay should preserve canonical main-branch truth");

    PricingReplayBundle {
        source_commit: replay.source_commit().clone(),
        source_snapshot: replay.source_snapshot().clone(),
        route_identity: replay.route_identity().clone(),
        invalidation_identity: replay.invalidation_identity().clone(),
    }
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_certification_matrix(
    policy: BridgeRuntimePolicy,
    preview_session_identity: BridgePreviewSessionIdentity,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingCertificationMatrix {
    let runtime = build_pricing_runtime_with_policy(
        pricing_reference_source(),
        RecordingSignalBridgeSink::default(),
        policy,
    );

    PricingCertificationMatrix {
        reference: capture_pricing_reference_bundle(
            &runtime,
            preview_session_identity,
            resource_request,
        ),
        replay: capture_pricing_replay_bundle(&runtime, resource_request),
    }
}
