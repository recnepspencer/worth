use super::*;

#[test]
fn idempotent_intent_noop_emits_receipt_without_mutation_or_signal_routing() {
    let routed = std::rc::Rc::new(std::cell::Cell::new(0));
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(CountingSignalSink {
            routed: routed.clone(),
        })
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .intent_authority(NoopIntentAuthority)
        .support_profile(intent_support_profile())
        .aspect_contracts(stateful_bridge_aspect_contracts())
        .expect("native intent test contracts should admit")
        .build_backend_from_parts()
        .build()
        .expect("intent-capable runtime should build");
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            "tasks.noop-intent",
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");
    let computed = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            WorthQueryDerivedView::new("computed.noop-intent", test_aspect_touches(["title"]))
                .depends_on_live(&live)
                .produces(test_aspect_touches(["title.summary"])),
            TitleListMaintainer,
        )
        .expect("computed should declare");
    let delivery_effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::deliver(
            "ui.noop-intent",
            WorthQueryEffectTrigger::computed_view(
                &computed,
                test_aspect_touches(["title.summary"]),
            ),
            "ui.noop-intent",
        ))
        .expect("effect should declare");

    let receipt = runtime
        .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
            "idempotent-title-reconcile",
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            test_intent_input([("entity", "task-1"), ("title", "already committed")]),
        ))
        .expect("idempotent intent should still mint a receipt");

    assert_eq!(
        receipt.execution_kind(),
        WorthQueryIntentExecutionKind::IdempotentNoop
    );
    assert!(receipt.is_idempotent_noop());
    assert!(receipt
        .terminal_affected_live_view_ids_projection()
        .is_empty());
    assert!(receipt
        .terminal_affected_derived_view_ids_projection()
        .is_empty());
    assert_eq!(receipt.considered_computed_view_count(), 0);
    assert_eq!(receipt.considered_effect_count(), 0);
    assert_eq!(receipt.delivered_effect_count(), 0);
    assert_eq!(receipt.pending_write_intent_count(), 0);
    assert!(!receipt.outcome_digest().is_empty());
    assert!(receipt.produced_mutation_digest().is_none());
    assert_eq!(
        receipt.invariant_evidence(),
        ["test-invariant-authority", "idempotent-noop"]
    );
    assert!(!receipt.commit_identity().is_empty());
    assert!(!receipt.snapshot_evidence_identity().as_str().is_empty());
    assert!(!receipt.receipt_digest().is_empty());
    assert_eq!(
        routed.get(),
        0,
        "no-op intent must not route signal invalidation"
    );
    assert_eq!(runtime.drain_patches(&live).query_delivery_batches.len(), 0);
    assert_eq!(
        runtime
            .read_derived_result(&computed)
            .expect("computed materialization should execute")
            .row_count(),
        0
    );
    assert_eq!(
        runtime
            .drain_effect_deliveries(&delivery_effect)
            .expect("effect queue should exist")
            .len(),
        0
    );
}

#[test]
fn idempotent_intent_inspection_preserves_outcome_without_mutation_claim() {
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .intent_authority(NoopIntentAuthority)
        .support_profile(intent_support_profile())
        .build_backend_from_parts()
        .build()
        .expect("intent-capable runtime should build");

    let receipt = runtime
        .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
            "inspectable-noop-intent",
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            test_intent_input([("entity", "task-1"), ("title", "already committed")]),
        ))
        .expect("idempotent intent should execute");
    let inspection = runtime
        .inspect_intent_receipt(&receipt)
        .expect("no-op intent receipt should inspect");

    assert_eq!(
        inspection.execution_kind(),
        WorthQueryIntentExecutionKind::IdempotentNoop
    );
    assert!(!inspection.outcome_digest().is_empty());
    assert_eq!(inspection.produced_mutation_digest(), None);
    assert_eq!(inspection.delivery_counters().affected_live_view_count(), 0);
    assert_eq!(inspection.delivery_counters().delivered_effect_count(), 0);
    assert!(!inspection.inspection_digest().is_empty());
}

#[test]
fn mutating_intent_with_empty_delta_denies_before_signal_routing() {
    let routed = std::rc::Rc::new(std::cell::Cell::new(0));
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(CountingSignalSink {
            routed: routed.clone(),
        })
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .intent_authority(EmptyMutatingIntentAuthority)
        .support_profile(intent_support_profile())
        .build_backend_from_parts()
        .build()
        .expect("intent-capable runtime should build");

    let error = runtime
        .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
            "empty-mutating-intent",
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            test_intent_input([("entity", "task-1")]),
        ))
        .expect_err("empty mutating execution must use the no-op constructor");

    match error {
        WorthQueryRuntimeError::IntentCommitDenied {
            intent_name,
            stage,
            message,
            evidence,
        } => {
            assert_eq!(intent_name, "empty-mutating-intent");
            assert_eq!(stage, "mutation-receipt-admission");
            assert!(message.contains("idempotent-noop"));
            assert_eq!(evidence.intent_name(), "empty-mutating-intent");
            assert_eq!(evidence.stage(), "mutation-receipt-admission");
            assert_eq!(
                evidence.execution_kind(),
                Some(WorthQueryIntentExecutionKind::Mutating)
            );
            assert!(evidence.attempt_digest().is_some());
            assert!(!evidence.denial_digest().as_str().is_empty());
        }
        other => panic!("expected mutation receipt admission denial, got {other:?}"),
    }
    assert_eq!(
        routed.get(),
        0,
        "denied empty mutation must not route signal invalidation"
    );
}
