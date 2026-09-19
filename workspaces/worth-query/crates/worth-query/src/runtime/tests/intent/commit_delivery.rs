use super::*;

#[test]
fn strategy_intent_commit_routes_query_delivery_and_returns_canonical_receipt() {
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
        .aspect_contracts(stateful_bridge_aspect_contracts())
        .expect("native intent test contracts should admit")
        .build_backend_from_parts()
        .build()
        .expect("intent-capable runtime should build");
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            "tasks.intent",
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");
    let computed = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            WorthQueryDerivedView::new("computed.intent", test_aspect_touches(["title"]))
                .depends_on_live(&live)
                .produces(test_aspect_touches(["title.summary"])),
            TitleListMaintainer,
        )
        .expect("computed should declare");
    let delivery_effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::deliver(
            "ui.intent",
            WorthQueryEffectTrigger::computed_view(
                &computed,
                test_aspect_touches(["title.summary"]),
            ),
            "ui.intent",
        ))
        .expect("effect should declare");

    let receipt = runtime
        .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
            "reconcile-task-title",
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            test_intent_input([("entity", "task-1"), ("title", "Intent committed title")]),
        ))
        .expect("intent should execute");

    assert_eq!(receipt.intent_name(), "reconcile-task-title");
    assert_eq!(receipt.strategy_identity(), "strategy.intent.reconcile");
    assert_eq!(
        receipt.execution_kind(),
        WorthQueryIntentExecutionKind::Mutating
    );
    assert!(!receipt.is_idempotent_noop());
    assert_eq!(receipt.strategy_version(), "1.0");
    assert_eq!(
        receipt.canonical_input_digest(),
        WorthQueryIntentDeclaration::strategy_commit(
            "reconcile-task-title",
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            test_intent_input([("entity", "task-1"), ("title", "Intent committed title"),]),
        )
        .input_digest()
    );
    assert_eq!(
        receipt.target_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert_eq!(
        receipt.terminal_affected_live_view_ids_projection(),
        ["tasks.intent"]
    );
    assert_eq!(
        receipt.terminal_affected_derived_view_ids_projection(),
        ["computed.intent"]
    );
    assert_eq!(receipt.considered_computed_view_count(), 1);
    assert_eq!(receipt.considered_effect_count(), 1);
    assert_eq!(receipt.delivered_effect_count(), 1);
    assert_eq!(receipt.pending_write_intent_count(), 0);
    assert_eq!(receipt.suppressed_effect_count(), 0);
    assert_eq!(receipt.meaningful_effect_suppression_count(), 0);
    assert_eq!(receipt.effect_expression_failure_count(), 0);
    assert!(!receipt.refresh_fallback());
    assert!(!receipt.outcome_digest().is_empty());
    assert!(receipt.produced_mutation_digest().is_some());
    assert!(!receipt.receipt_digest().is_empty());
    assert_eq!(receipt.invariant_evidence(), ["test-invariant-authority"]);
    assert!(receipt
        .commit_identity()
        .is_same_current_identity_as(&receipt.commit_identity().clone()));
    assert!(receipt
        .snapshot_identity()
        .is_same_current_identity_as(&receipt.snapshot_identity().clone()));
    let copied_commit = crate::memory_workspace::admit_external_commit_label(
        receipt
            .commit_identity()
            .terminal_projection_for_reporting(),
    );
    let copied_snapshot = crate::memory_workspace::admit_external_snapshot_label(
        receipt
            .snapshot_identity()
            .terminal_projection_for_reporting(),
    );
    assert!(!receipt
        .commit_identity()
        .is_same_current_identity_as(&copied_commit));
    assert!(!receipt
        .snapshot_identity()
        .is_same_current_identity_as(&copied_snapshot));
    assert_eq!(runtime.drain_patches(&live).query_delivery_batches.len(), 1);
    assert_eq!(
        runtime
            .read_derived_result(&computed)
            .expect("computed materialization should execute")
            .row_count(),
        1
    );
    assert_eq!(
        runtime
            .drain_effect_deliveries(&delivery_effect)
            .expect("effect queue should exist")
            .len(),
        1
    );
}

#[test]
fn intent_receipt_inspection_explains_strategy_lanes_and_delivery_counters() {
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
            "tasks.intent-inspection",
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");
    let computed = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            WorthQueryDerivedView::new(
                "computed.intent-inspection",
                test_aspect_touches(["title"]),
            )
            .depends_on_live(&live)
            .produces(test_aspect_touches(["title.summary"])),
            TitleListMaintainer,
        )
        .expect("computed should declare");
    runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::deliver(
            "ui.intent-inspection",
            WorthQueryEffectTrigger::computed_view(
                &computed,
                test_aspect_touches(["title.summary"]),
            ),
            "ui.intent-inspection",
        ))
        .expect("effect should declare");

    let receipt = runtime
        .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
            "inspectable-intent",
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            test_intent_input([("entity", "task-1"), ("title", "Intent committed title")]),
        ))
        .expect("intent should execute");
    let inspection = runtime
        .inspect_intent_receipt(&receipt)
        .expect("intent receipt should inspect");

    assert_eq!(inspection.intent_name(), "inspectable-intent");
    assert_eq!(
        inspection.execution_kind(),
        WorthQueryIntentExecutionKind::Mutating
    );
    assert_eq!(inspection.strategy_identity(), "strategy.intent.reconcile");
    assert_eq!(
        inspection.source_lane(),
        WorthQueryIntentSourceLane::UserAuthored
    );
    assert_eq!(
        inspection.target_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert_eq!(inspection.outcome_digest(), receipt.outcome_digest());
    assert_eq!(
        inspection.produced_mutation_digest(),
        receipt.produced_mutation_digest()
    );
    assert_eq!(inspection.receipt_digest(), receipt.receipt_digest());
    assert_eq!(inspection.delivery_counters().affected_live_view_count(), 1);
    assert_eq!(
        inspection.delivery_counters().affected_derived_view_count(),
        1
    );
    assert_eq!(
        inspection
            .delivery_counters()
            .considered_computed_view_count(),
        1
    );
    assert_eq!(inspection.delivery_counters().delivered_effect_count(), 1);
    assert!(!inspection.delivery_counters().counter_digest().is_empty());
    assert!(!inspection.inspection_digest().is_empty());
}
