use super::*;

#[test]
fn composed_runtime_surface_proves_facade_handles_stay_proof_bearing_across_preview_and_intents() {
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .intent_authority(TestIntentAuthority)
        .support_profile(intent_support_profile())
        .build_backend_from_parts()
        .build()
        .expect("intent-capable runtime should build");
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            "tasks.deep-runtime",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should install a subscription");
    let titles = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            WorthQueryDerivedView::new("computed.deep.titles", test_aspect_touches(["title"]))
                .depends_on_live(&live)
                .produces(test_aspect_touches(["title.summary"])),
            TitleListMaintainer,
        )
        .expect("source computed should declare");
    let readiness = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            WorthQueryDerivedView::new(
                "computed.deep.readiness",
                test_aspect_touches(["title.summary"]),
            )
            .depends_on_derived(&titles)
            .produces(test_aspect_touches(["validation.state"])),
            SummaryMaintainer,
        )
        .expect("nested computed should declare");
    let effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(
            WorthQueryEffectDeclaration::write_intent(
                "effects.deep.reconcile-readiness",
                WorthQueryEffectTrigger::computed_view(
                    &readiness,
                    test_aspect_touches(["validation.state"]),
                ),
                "strategy.intent.reconcile",
            )
            .with_condition(WorthQueryEffectCondition::expression(
                "expr.validation.ready",
                test_aspect_touches(["validation.state"]),
                test_aspect_touches(["intent.reconcile"]),
            )),
        )
        .expect("conditional write-intent effect should declare");

    let (preview_evidence, preview_outcome) = {
        let mut preview = runtime
            .preview_with_options(
                test_session_label("deep runtime preview"),
                WorthQueryPreviewOptions::sandboxed_write_intent(),
            )
            .expect("preview session should be admitted");
        preview.use_view(&live);
        preview.use_computed(&titles);
        preview.use_computed(&readiness);
        preview
            .use_effect(&effect)
            .expect("sandboxed preview should bind pending-intent effect");
        preview
            .write(insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("preview-deep-task")),
                    (
                        "title.value",
                        test_string_aspect_value("Preview-only deep task"),
                    ),
                ],
            ))
            .expect("preview write should route only preview evidence");
        (
            preview.preview_execution_evidence().to_vec(),
            preview.discard(),
        )
    };

    assert!(preview_evidence.iter().any(|evidence| {
        evidence.kind() == WorthQueryPreviewExecutionKind::LivePatch
            && evidence.handle_name() == "tasks.deep-runtime"
            && evidence.preview_lane() == WorthQueryAuthorityLane::PreviewTruth
    }));
    assert!(preview_evidence.iter().any(|evidence| {
        evidence.kind() == WorthQueryPreviewExecutionKind::ComputedPatch
            && evidence.handle_name() == "computed.deep.titles"
            && evidence.aspect_touches() == test_aspect_touches(["title.summary"]).as_slice()
    }));
    assert!(preview_evidence.iter().any(|evidence| {
        evidence.kind() == WorthQueryPreviewExecutionKind::ComputedPatch
            && evidence.handle_name() == "computed.deep.readiness"
            && evidence.aspect_touches() == test_aspect_touches(["validation.state"]).as_slice()
    }));
    assert!(preview_evidence.iter().any(|evidence| {
        evidence.kind() == WorthQueryPreviewExecutionKind::PendingWriteIntent
            && evidence.handle_name() == "effects.deep.reconcile-readiness"
            && evidence.source_lane() == WorthQueryAuthorityLane::PendingWriteIntent
            && evidence.preview_lane() == WorthQueryAuthorityLane::PreviewTruth
    }));
    assert_eq!(preview_outcome.pending_write_intent_residue_count(), 1);
    assert_eq!(preview_outcome.authoritative_residue_count(), 0);
    assert!(runtime
        .drain_patches(&live)
        .query_delivery_batches
        .is_empty());
    assert_eq!(
        runtime
            .read_derived_result(&readiness)
            .expect("readiness materialization should execute")
            .row_count(),
        0
    );
    assert!(runtime
        .drain_effect_deliveries(&effect)
        .expect("authoritative effect queue should exist")
        .is_empty());

    let receipt = runtime
        .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
            "deep-authoritative-reconcile",
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            test_intent_input([
                ("entity", "intent-task-1"),
                ("title", "Committed deep title"),
            ]),
        ))
        .expect("authoritative intent should execute");

    assert_eq!(
        receipt.execution_kind(),
        WorthQueryIntentExecutionKind::Mutating
    );
    assert_eq!(
        receipt.terminal_affected_live_view_ids_projection(),
        ["tasks.deep-runtime"]
    );
    assert_eq!(
        receipt.terminal_affected_derived_view_ids_projection(),
        ["computed.deep.readiness", "computed.deep.titles"]
    );
    assert_eq!(receipt.considered_computed_view_count(), 2);
    assert_eq!(receipt.considered_effect_count(), 1);
    assert_eq!(receipt.pending_write_intent_count(), 1);
    assert_eq!(receipt.delivered_effect_count(), 0);
    assert_eq!(
        receipt.source_lane(),
        WorthQueryIntentSourceLane::UserAuthored
    );
    assert_eq!(
        receipt.target_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );

    let live_batches = runtime.drain_patches(&live).query_delivery_batches;
    assert_eq!(live_batches.len(), 1);
    assert_eq!(live_batches[0].view_name(), "tasks.deep-runtime");
    assert_eq!(
        live_batches[0].authority_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert_eq!(
        live_batches[0].patch_group_kind(),
        QueryPatchGroupKind::DetailFieldPatchGroup
    );
    assert_eq!(live_batches[0].patch_group_width(), 1);

    let readiness_inspection = runtime
        .inspect_derived_view(&readiness)
        .expect("readiness computed should inspect");
    assert_eq!(
        readiness_inspection.upstream_derived_views(),
        &["computed.deep.titles".to_string()]
    );
    assert_eq!(
        readiness_inspection.produced_aspect_touches(),
        test_aspect_touches(["validation.state"]).as_slice()
    );
    assert_eq!(readiness_inspection.materialized_row_count(), 1);
    assert_eq!(readiness_inspection.pending_incremental_patch_count(), 1);

    let effect_inspection = runtime
        .inspect_effect(&effect)
        .expect("pending effect should inspect");
    assert_eq!(
        effect_inspection.condition_descriptor(),
        "expr.validation.ready"
    );
    assert_eq!(
        effect_inspection.pending_write_intent_count(),
        1,
        "authoritative intent should create exactly one pending write intent"
    );
    assert_eq!(
        effect_inspection.latest_delivery_family(),
        Some(&WorthQueryEffectDeliveryFamily::PendingWriteIntent)
    );
    assert_eq!(
        effect_inspection
            .feedback_graph()
            .expect("effect inspection should include feedback graph")
            .phase_nodes(),
        &[
            WorthQueryFeedbackPhaseNode::TruthRead,
            WorthQueryFeedbackPhaseNode::Derive,
            WorthQueryFeedbackPhaseNode::PendingWriteIntent,
        ]
    );

    let effect_intent = runtime
        .execute_next_effect_write_intent(&effect, "1.0", "effect.intent.input.v1")
        .expect("effect-triggered intent should use the same intent authority path");
    assert_eq!(
        effect_intent.intent_receipt().source_lane(),
        WorthQueryIntentSourceLane::EffectTriggered
    );
    assert_eq!(
        effect_intent.intent_receipt().target_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    let feedback_graph = runtime
        .inspect_effect_feedback_receipt(&effect_intent)
        .expect("effect-triggered intent should expose feedback graph");
    assert_eq!(
        feedback_graph.phase_nodes(),
        &[
            WorthQueryFeedbackPhaseNode::TruthRead,
            WorthQueryFeedbackPhaseNode::Derive,
            WorthQueryFeedbackPhaseNode::PendingWriteIntent,
            WorthQueryFeedbackPhaseNode::Commit,
            WorthQueryFeedbackPhaseNode::BridgeRoute,
            WorthQueryFeedbackPhaseNode::Resubscribe,
        ]
    );
    assert_eq!(
        feedback_graph.termination(),
        WorthQueryFeedbackTermination::CommittedResubscribe
    );

    let second_live_batches = runtime.drain_patches(&live).query_delivery_batches;
    assert_eq!(
        second_live_batches.len(),
        1,
        "effect-triggered intent should resubscribe through query-shaped delivery"
    );
    assert_eq!(
        second_live_batches[0].patch_group_kind(),
        QueryPatchGroupKind::DetailFieldPatchGroup
    );

    let branch_receipt = {
        let mut branch = runtime
            .branch_with_options(
                test_session_label("deep runtime branch"),
                WorthQueryBranchOptions::sandboxed_write_intent(),
            )
            .expect("branch session should be admitted");
        branch
            .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
                "deep-branch-reconcile",
                "strategy.intent.reconcile",
                "1.0",
                "intent.reconcile.input.v1",
                test_intent_input([("entity", "intent-task-1"), ("title", "Branch-only title")]),
            ))
            .expect("branch-local intent should stay branch-local")
    };
    assert_eq!(
        branch_receipt.source_lane(),
        WorthQueryIntentSourceLane::BranchLocal
    );
    assert_eq!(
        branch_receipt.target_lane(),
        WorthQueryAuthorityLane::BranchLocalTruth
    );
    assert!(runtime
        .drain_patches(&live)
        .query_delivery_batches
        .is_empty());
}
