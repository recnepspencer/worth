use super::super::support::*;
use crate::runtime::async_result_state::runtime_async_checkpoint_label_identity;

#[test]
fn unsupported_facade_family_stop_class_preserves_denied_family_and_reason() {
    let denial = WorthQueryRuntimeSupportDenial::new(
        WorthQueryRuntimeFacadeFamily::Temporal,
        WorthQueryRuntimeFamilySupportStatus::Unsupported,
        Some(WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly),
        "temporal lane is support-gated",
    );
    let error = WorthQueryRuntimeError::UnsupportedFacadeFamily(denial);

    match error.stop_class() {
        WorthQueryStopClass::FamilyAdmissionDenied {
            family,
            status,
            teaching_posture,
            reason,
        } => {
            assert_eq!(family, WorthQueryRuntimeFacadeFamily::Temporal);
            assert_eq!(status, WorthQueryRuntimeFamilySupportStatus::Unsupported);
            assert_eq!(
                teaching_posture,
                Some(WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly)
            );
            assert_eq!(reason, "temporal lane is support-gated");
        }
        other => panic!("expected family admission denial stop class, got {other:?}"),
    }
}

#[test]
fn preview_promotion_stop_class_preserves_kind_and_evidence() {
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(DenyingWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .build_backend_from_parts()
        .build()
        .expect("denying write backend should build");

    let error = {
        let mut preview = runtime
            .preview(test_session_label("stop-class-write-failure"))
            .expect("preview should admit");
        preview
            .write(insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("denied-preview")),
                    (
                        "title.value",
                        test_string_aspect_value("Denied preview write"),
                    ),
                ],
            ))
            .expect("preview write should stage");
        preview.promote().expect_err("promotion should fail")
    };

    match error.stop_class() {
        WorthQueryStopClass::PreviewPromotionDenied { kind, evidence } => {
            assert_eq!(kind, WorthQueryPreviewPromotionDenialKind::WriteFailed);
            assert_eq!(
                evidence.kind(),
                WorthQueryPreviewPromotionDenialKind::WriteFailed
            );
            assert_eq!(evidence.failed_write_sequence(), Some(1));
        }
        other => panic!("expected preview promotion stop class, got {other:?}"),
    }
}

#[test]
fn preview_promotion_stop_class_preserves_all_denial_kinds() {
    let stale_basis_error = {
        let mut runtime = test_product_runtime_builder()
            .schema_adapter(TestSchemaAdapter)
            .source_adapter(TestSourceAdapter::default())
            .snapshot_identity(DriftingSnapshotIdentityAdapter::default())
            .write_authority(TestWriteAuthority)
            .signal_sink(TestSignalSink)
            .subscription_activation(TestSubscriptionActivation)
            .preview_basis(TestPreviewBasis)
            .inspector_evidence(TestInspectorEvidence)
            .build_backend_from_parts()
            .build()
            .expect("drifting backend should build");
        let mut preview = runtime
            .preview(test_session_label("stop-class stale basis"))
            .expect("preview session should admit");
        preview
            .write(insert_command(
                "Task",
                [
                    (
                        "identity.id",
                        test_string_aspect_value("stale-preview-stop-class"),
                    ),
                    (
                        "title.value",
                        test_string_aspect_value("Should not promote"),
                    ),
                ],
            ))
            .expect("preview write should stage");
        preview
            .promote()
            .expect_err("drifting basis should deny promotion")
    };

    match stale_basis_error.stop_class() {
        WorthQueryStopClass::PreviewPromotionDenied { kind, evidence } => {
            assert_eq!(kind, WorthQueryPreviewPromotionDenialKind::StaleBasis);
            assert_eq!(
                evidence.kind(),
                WorthQueryPreviewPromotionDenialKind::StaleBasis
            );
            assert_eq!(evidence.staged_preview_write_count(), 1);
            assert_eq!(evidence.promoted_write_count(), 0);
        }
        other => panic!("expected stale-basis stop class, got {other:?}"),
    }

    let atomic_batch_error = {
        let attempted_writes = std::rc::Rc::new(std::cell::Cell::new(0));
        let mut runtime = test_product_runtime_builder()
            .schema_adapter(TestSchemaAdapter)
            .source_adapter(TestSourceAdapter::default())
            .snapshot_identity(TestSnapshotIdentityAdapter)
            .write_authority(CountingWriteAuthority {
                attempted_writes: attempted_writes.clone(),
            })
            .signal_sink(TestSignalSink)
            .subscription_activation(TestSubscriptionActivation)
            .preview_basis(TestPreviewBasis)
            .inspector_evidence(TestInspectorEvidence)
            .build_backend_from_parts()
            .build()
            .expect("counting backend should build");
        let mut preview = runtime
            .preview(test_session_label("stop-class atomic batch"))
            .expect("preview session should admit");
        preview
            .write(insert_command(
                "Task",
                [
                    (
                        "identity.id",
                        test_string_aspect_value("preview-batch-stop-class-1"),
                    ),
                    (
                        "title.value",
                        test_string_aspect_value("First staged write"),
                    ),
                ],
            ))
            .expect("first preview write should stage");
        preview
            .write(insert_command(
                "Task",
                [
                    (
                        "identity.id",
                        test_string_aspect_value("preview-batch-stop-class-2"),
                    ),
                    (
                        "title.value",
                        test_string_aspect_value("Second staged write"),
                    ),
                ],
            ))
            .expect("second preview write should stage");
        let error = preview
            .promote()
            .expect_err("multi-write promotion should deny before authority");
        assert_eq!(attempted_writes.get(), 0);
        error
    };

    match atomic_batch_error.stop_class() {
        WorthQueryStopClass::PreviewPromotionDenied { kind, evidence } => {
            assert_eq!(
                kind,
                WorthQueryPreviewPromotionDenialKind::AtomicBatchUnsupported
            );
            assert_eq!(
                evidence.kind(),
                WorthQueryPreviewPromotionDenialKind::AtomicBatchUnsupported
            );
            assert_eq!(evidence.staged_preview_write_count(), 2);
            assert_eq!(evidence.promoted_write_count(), 0);
            assert_eq!(evidence.failed_write_sequence(), None);
        }
        other => panic!("expected atomic-batch stop class, got {other:?}"),
    }

    let rebinding_required_error = {
        let mut runtime = stateful_bridge_task_runtime();
        let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
            .declare_live_view(
                "tasks.preview-promotion-stop-class-mismatch",
                task_live_request(),
                task_schema(),
            )
            .expect("live view should declare");
        let (_, generation_digest) = live_subscription_async_identity(&runtime, view.name());
        runtime
            .project_async_result_state(
                view.name(),
                &WorthQueryRuntimeAsyncResultProjection::completion_state(
                    BridgeAsyncCompletionState::Denied(
                        BridgeAsyncCompletionDenialClass::SignalLifecycleDenied,
                    ),
                    "async:preview-stop-class-mismatch",
                ),
                &runtime_async_checkpoint_label_identity("basis:drifted"),
                &generation_digest,
            )
            .expect("preview mismatch should remain typed");
        let mut preview = runtime
            .preview_with_options(
                test_session_label("preview promotion stop class mismatch"),
                WorthQueryPreviewOptions::sandboxed_write_intent(),
            )
            .expect("preview session should admit");
        preview.use_view(&view);
        preview
            .write(insert_command(
                "Task",
                [
                    (
                        "identity.id",
                        test_string_aspect_value("preview-promotion-stop-class-mismatch"),
                    ),
                    (
                        "title.value",
                        test_string_aspect_value("Should require rebinding"),
                    ),
                ],
            ))
            .expect("preview write should stage");
        preview
            .promote()
            .expect_err("crossed residue should require rebinding")
    };

    match rebinding_required_error.stop_class() {
        WorthQueryStopClass::PreviewPromotionDenied { kind, evidence } => {
            assert_eq!(
                kind,
                WorthQueryPreviewPromotionDenialKind::RebindingRequired
            );
            assert_eq!(
                evidence.kind(),
                WorthQueryPreviewPromotionDenialKind::RebindingRequired
            );
            assert_eq!(evidence.crossed_authoritative_residue_count(), 1);
            assert_eq!(
                evidence.recovery_posture(),
                "discard_preview_and_readmit_authoritative"
            );
        }
        other => panic!("expected rebinding-required stop class, got {other:?}"),
    }
}

#[test]
fn preview_operation_effect_denial_stop_class_preserves_typed_label_identity() {
    let label = test_session_label("preview.operation.effect.denied");
    let error = WorthQueryRuntimeError::PreviewOperationEffectDenied {
        label: label.clone(),
        stage: "effect-admission",
        message: "preview effect denied".to_string(),
    };

    match error.stop_class() {
        WorthQueryStopClass::PreviewOperationEffectDenied {
            label: classified_label,
            stage,
            message,
        } => {
            assert_eq!(classified_label.identity_digest(), label.identity_digest());
            assert_eq!(stage, "effect-admission");
            assert_eq!(message, "preview effect denied");
        }
        other => panic!("expected preview operation effect stop class, got {other:?}"),
    }
}

#[test]
fn intent_commit_stop_class_preserves_stage_and_evidence() {
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
        .expect("intent runtime should build");

    let error = {
        let mut branch = runtime
            .branch(test_session_label("derive-only branch intent"))
            .expect("branch should admit");
        branch
            .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
                "branch-denied",
                "strategy.intent.reconcile",
                "1.0",
                "intent.reconcile.input.v1",
                test_intent_input([("entity", "task-1")]),
            ))
            .expect_err("derive-only branch must deny write intents")
    };

    match error.stop_class() {
        WorthQueryStopClass::IntentCommitDenied {
            intent_name,
            stage,
            evidence,
            ..
        } => {
            assert_eq!(intent_name, "branch-denied");
            assert_eq!(stage, "branch-effect-policy-admission");
            assert_eq!(evidence.intent_name(), "branch-denied");
            assert_eq!(evidence.stage(), "branch-effect-policy-admission");
        }
        other => panic!("expected intent commit stop class, got {other:?}"),
    }
}
