use super::*;

#[test]
fn intent_support_profile_claim_requires_executable_authority_adapter() {
    let error = match test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .support_profile(intent_support_profile())
        .build_backend_from_parts()
        .build()
    {
        Ok(_) => panic!("intent support claim without adapter should fail"),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        WorthQueryRuntimeError::UnsupportedFacadeFamily(denial)
            if denial.family() == WorthQueryRuntimeFacadeFamily::Intent
                && denial.reason().contains("intent authority adapter")
    ));
}

#[test]
fn intent_source_lanes_that_need_policy_deny_before_authority_execution() {
    let attempted = std::rc::Rc::new(std::cell::Cell::new(0));
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .intent_authority(CountingIntentAuthority {
            attempted: attempted.clone(),
        })
        .support_profile(intent_support_profile())
        .build_backend_from_parts()
        .build()
        .expect("intent-capable runtime should build");

    for source_lane in [
        WorthQueryIntentSourceLane::EffectTriggered,
        WorthQueryIntentSourceLane::PreviewLocal,
        WorthQueryIntentSourceLane::BranchLocal,
        WorthQueryIntentSourceLane::DerivedRuntime,
    ] {
        let error = runtime
            .execute_intent(
                WorthQueryIntentDeclaration::strategy_commit(
                    "non-user-authored",
                    "strategy.intent.reconcile",
                    "1.0",
                    "intent.reconcile.input.v1",
                    test_intent_input([("entity", "task-1")]),
                )
                .with_source_lane(source_lane),
            )
            .expect_err("non-user-authored lanes require later explicit policy admission");

        match error {
            WorthQueryRuntimeError::IntentCommitDenied {
                intent_name,
                stage,
                message,
                evidence,
            } => {
                assert_eq!(intent_name, "non-user-authored");
                assert_eq!(stage, "source-lane-admission");
                assert!(message.contains(source_lane.as_str()));
                assert_eq!(evidence.execution_kind(), None);
                assert_eq!(evidence.source_lane(), source_lane);
            }
            other => panic!("expected source lane intent denial, got {other:?}"),
        }
    }

    assert_eq!(
        attempted.get(),
        0,
        "source lane denial must happen before the intent authority can publish"
    );
}

#[test]
fn intent_execution_strategy_drift_denies_before_signal_routing() {
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
        .intent_authority(DriftingIntentAuthority)
        .support_profile(intent_support_profile())
        .build_backend_from_parts()
        .build()
        .expect("intent-capable runtime should build");

    let error = runtime
        .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
            "drifting-strategy",
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            test_intent_input([("entity", "task-1")]),
        ))
        .expect_err("strategy drift must not mint an intent receipt");

    match error {
        WorthQueryRuntimeError::IntentCommitDenied {
            intent_name,
            stage,
            message,
            evidence,
        } => {
            assert_eq!(intent_name, "drifting-strategy");
            assert_eq!(stage, "strategy-admission");
            assert!(message.contains("returned strategy"));
            assert_eq!(
                evidence.execution_kind(),
                Some(WorthQueryIntentExecutionKind::Mutating)
            );
            assert_eq!(evidence.strategy_identity(), "strategy.intent.reconcile");
            assert_eq!(
                evidence.returned_strategy_identity(),
                Some("strategy.intent.other")
            );
            assert_eq!(
                evidence.returned_strategy_descriptor_digest(),
                Some("test-strategy-descriptor-digest")
            );
        }
        other => panic!("expected backend execution denial before routing, got {other:?}"),
    }
    assert_eq!(
        routed.get(),
        0,
        "drifted intent execution must not route signal invalidations"
    );
}

#[test]
fn strategy_drift_denial_inspection_keeps_declared_and_returned_strategy_separate() {
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .intent_authority(DriftingIntentAuthority)
        .support_profile(intent_support_profile())
        .build_backend_from_parts()
        .build()
        .expect("intent-capable runtime should build");

    let error = runtime
        .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
            "inspectable-drifting-strategy",
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            test_intent_input([("entity", "task-1")]),
        ))
        .expect_err("strategy drift must deny");

    let evidence = match error {
        WorthQueryRuntimeError::IntentCommitDenied { evidence, .. } => evidence,
        other => panic!("expected strategy drift denial, got {other:?}"),
    };
    let inspection = runtime
        .inspect_intent_denial(&evidence)
        .expect("strategy drift denial should inspect");

    assert_eq!(inspection.strategy_identity(), "strategy.intent.reconcile");
    assert_eq!(
        inspection.returned_strategy_identity(),
        Some("strategy.intent.other")
    );
    assert_eq!(
        inspection.returned_strategy_descriptor_digest(),
        Some("test-strategy-descriptor-digest")
    );
    assert_eq!(
        inspection.execution_kind(),
        Some(WorthQueryIntentExecutionKind::Mutating)
    );
    assert_eq!(
        inspection.denial_digest(),
        evidence.denial_digest().as_str()
    );
    assert!(!inspection.inspection_digest().is_empty());
}
