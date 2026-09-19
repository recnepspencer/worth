use super::*;

#[test]
fn mutating_intent_cannot_promote_projection_only_truth_into_query_authority() {
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .intent_authority(AuthoritylessIntentAuthority)
        .support_profile(intent_support_profile())
        .build_backend_from_parts()
        .build()
        .expect("intent-capable runtime should build");

    let error = runtime
        .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
            "authorityless-mutating-intent",
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            test_intent_input([("entity", "task-1")]),
        ))
        .expect_err("projection-only truth must not become a current Query receipt");

    match error {
        WorthQueryRuntimeError::IntentCommitDenied { stage, message, .. } => {
            assert_eq!(stage, "mutation-receipt-authority-admission");
            assert!(message.contains("Bridge-authored commit and snapshot handoff"));
        }
        other => panic!("expected intent authority admission denial, got {other:?}"),
    }
}

#[test]
fn invariant_denial_inspection_explains_failed_invariants_without_commit_identity() {
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .intent_authority(InvariantViolationIntentAuthority)
        .support_profile(intent_support_profile())
        .build_backend_from_parts()
        .build()
        .expect("intent-capable runtime should build");

    let error = runtime
        .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
            "inspectable-invariant-denial",
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            test_intent_input([("entity", "task-1"), ("dependency", "cycle")]),
        ))
        .expect_err("invariant violation must deny");

    let evidence = match error {
        WorthQueryRuntimeError::IntentCommitDenied { evidence, .. } => evidence,
        other => panic!("expected invariant admission denial, got {other:?}"),
    };
    let inspection = runtime
        .inspect_intent_denial(&evidence)
        .expect("denial evidence should inspect");

    assert_eq!(inspection.intent_name(), "inspectable-invariant-denial");
    assert_eq!(inspection.stage(), "invariant-admission");
    assert_eq!(
        inspection.execution_kind(),
        Some(WorthQueryIntentExecutionKind::InvariantViolation)
    );
    assert_eq!(
        inspection.returned_strategy_identity(),
        Some("strategy.intent.reconcile")
    );
    assert_eq!(
        inspection.invariant_evidence(),
        [
            "relational-invariant:constraint-a:false",
            "relational-invariant:constraint-b:false"
        ]
    );
    assert!(inspection.attempt_digest().is_some());
    assert!(inspection.snapshot_identity().is_some());
    assert_eq!(
        inspection.denial_digest(),
        evidence.denial_digest().as_str()
    );
    assert!(!inspection.inspection_digest().is_empty());
}

#[test]
fn invariant_violation_intent_denies_with_evidence_without_partial_publication() {
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
        .intent_authority(InvariantViolationIntentAuthority)
        .support_profile(intent_support_profile())
        .build_backend_from_parts()
        .build()
        .expect("intent-capable runtime should build");
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            "tasks.invariant-denial",
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");
    let computed = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            WorthQueryDerivedView::new("computed.invariant-denial", test_aspect_touches(["title"]))
                .depends_on_live(&live)
                .produces(test_aspect_touches(["title.summary"])),
            TitleListMaintainer,
        )
        .expect("computed should declare");
    let delivery_effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::deliver(
            "ui.invariant-denial",
            WorthQueryEffectTrigger::computed_view(
                &computed,
                test_aspect_touches(["title.summary"]),
            ),
            "ui.invariant-denial",
        ))
        .expect("effect should declare");

    let error = runtime
        .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
            "violates-relational-invariants",
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            test_intent_input([("entity", "task-1"), ("dependency", "cycle")]),
        ))
        .expect_err("invariant violation must not mint an intent receipt");

    match error {
        WorthQueryRuntimeError::IntentCommitDenied {
            intent_name,
            stage,
            message,
            evidence,
        } => {
            assert_eq!(intent_name, "violates-relational-invariants");
            assert_eq!(stage, "invariant-admission");
            assert!(message.contains("relational invariants failed"));
            assert_eq!(evidence.intent_name(), "violates-relational-invariants");
            assert_eq!(evidence.stage(), "invariant-admission");
            assert_eq!(
                evidence.execution_kind(),
                Some(WorthQueryIntentExecutionKind::InvariantViolation)
            );
            assert_eq!(
                evidence.invariant_evidence(),
                [
                    "relational-invariant:constraint-a:false",
                    "relational-invariant:constraint-b:false"
                ]
            );
            assert!(evidence.attempt_digest().is_some());
            assert!(evidence.snapshot_identity().is_some());
            assert!(!evidence.denial_digest().as_str().is_empty());
        }
        other => panic!("expected invariant admission denial, got {other:?}"),
    }
    assert_eq!(
        routed.get(),
        0,
        "invariant-denied intent must not route signal invalidation"
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
