use super::super::support::*;
use crate::WorthQueryEvidenceScope;

#[test]
fn preview_local_intent_is_policy_admitted_without_authoritative_execution() {
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

    let (receipt, outcome) = {
        let mut preview = runtime
            .preview_with_options(
                test_session_label("preview local intent"),
                WorthQueryPreviewOptions::sandboxed_write_intent(),
            )
            .expect("preview session should be admitted");
        let receipt = preview
            .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
                "preview-reconcile",
                "strategy.intent.reconcile",
                "1.0",
                "intent.reconcile.input.v1",
                test_intent_input([("entity", "task-1"), ("title", "preview title")]),
            ))
            .expect("sandboxed preview intent should be admitted");

        assert_eq!(receipt.intent_name(), "preview-reconcile");
        assert_eq!(receipt.strategy_identity(), "strategy.intent.reconcile");
        assert_eq!(receipt.strategy_version(), "1.0");
        assert_eq!(
            receipt.source_lane(),
            WorthQueryIntentSourceLane::PreviewLocal
        );
        assert_eq!(receipt.target_lane(), WorthQueryAuthorityLane::PreviewTruth);
        assert_eq!(
            receipt.effect_policy(),
            WorthQueryEffectPolicy::SandboxedWriteIntent
        );
        assert!(!receipt.basis_evidence().is_empty());
        assert!(!receipt.admission_identity().as_str().is_empty());
        assert!(!receipt.receipt_digest().is_empty());
        assert_eq!(
            preview.preview_intent_receipts(),
            std::slice::from_ref(&receipt)
        );
        assert!(preview.preview_execution_evidence().iter().any(|evidence| {
            evidence.kind() == WorthQueryPreviewExecutionKind::PendingWriteIntent
                && evidence.handle_name() == "preview-reconcile"
                && evidence.source_lane() == WorthQueryAuthorityLane::PendingWriteIntent
                && evidence.preview_lane() == WorthQueryAuthorityLane::PreviewTruth
                && evidence.source_evidence_identity() == receipt.receipt_identity()
                && evidence.aspect_touches().is_empty()
                && evidence.intent_strategy_name() == Some("strategy.intent.reconcile")
        }));
        (receipt, preview.discard())
    };

    assert_eq!(
        attempted.get(),
        0,
        "preview-local intent admission must not execute authoritative intent authority"
    );
    assert_eq!(outcome.pending_write_intent_residue_count(), 1);
    assert_eq!(
        outcome
            .closeout_evidence()
            .class_count(WorthQueryPreviewResidueClass::PendingWriteIntent),
        1
    );
    assert_eq!(outcome.authoritative_residue_count(), 0);

    let receipt_inspection = runtime
        .inspect_preview_intent_receipt(&receipt)
        .expect("preview intent inspection should succeed");
    assert_eq!(receipt_inspection.intent_name(), "preview-reconcile");
    assert_eq!(
        receipt_inspection.source_lane(),
        WorthQueryIntentSourceLane::PreviewLocal
    );
    assert_eq!(
        receipt_inspection.target_lane(),
        WorthQueryAuthorityLane::PreviewTruth
    );
    assert_eq!(
        receipt_inspection.effect_policy(),
        WorthQueryEffectPolicy::SandboxedWriteIntent
    );
    assert!(!receipt_inspection.basis_digest().is_empty());
    assert!(!receipt_inspection.inspection_digest().is_empty());
    assert_eq!(
        receipt_inspection.basis_identity().scope(),
        WorthQueryEvidenceScope::PreviewIntentReceiptInspectionBasis
    );
    assert_eq!(
        receipt_inspection.inspection_identity().scope(),
        WorthQueryEvidenceScope::PreviewIntentReceiptInspection
    );
    assert_eq!(
        receipt_inspection.admission_digest(),
        receipt.admission_digest()
    );
    assert_eq!(
        receipt_inspection.admission_identity(),
        receipt.admission_identity()
    );
    assert_eq!(
        receipt_inspection.receipt_digest(),
        receipt.receipt_digest()
    );
    assert_eq!(
        receipt_inspection.receipt_identity(),
        receipt.receipt_identity()
    );

    let outcome_inspection = runtime
        .inspect_preview_outcome(&outcome)
        .expect("preview outcome inspection should succeed");
    assert_eq!(outcome_inspection.subscription_residue_count(), 0);
    assert_eq!(outcome_inspection.derived_runtime_residue_count(), 0);
    assert_eq!(outcome_inspection.pending_write_intent_residue_count(), 1);
    assert_eq!(outcome_inspection.preview_write_staging_count(), 0);
    assert_eq!(outcome_inspection.promoted_write_count(), 0);
    assert_eq!(outcome_inspection.authoritative_residue_count(), 0);
}

#[test]
fn derive_only_preview_intent_denies_before_authoritative_execution() {
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

    let error = {
        let mut preview = runtime
            .preview(test_session_label("derive-only preview intent"))
            .expect("preview session should be admitted");
        preview
            .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
                "preview-denied",
                "strategy.intent.reconcile",
                "1.0",
                "intent.reconcile.input.v1",
                test_intent_input([("entity", "task-1")]),
            ))
            .expect_err("derive-only preview must deny write intents")
    };

    match error {
        WorthQueryRuntimeError::IntentCommitDenied {
            intent_name,
            stage,
            message,
            evidence: _,
        } => {
            assert_eq!(intent_name, "preview-denied");
            assert_eq!(stage, "preview-effect-policy-admission");
            assert!(message.contains("derive-only"));
            assert!(message.contains("write-intent"));
        }
        other => panic!("expected preview policy intent denial, got {other:?}"),
    }
    assert_eq!(
        attempted.get(),
        0,
        "preview policy denial must happen before authoritative intent authority"
    );
}

#[test]
fn preview_local_intent_requires_intent_support_for_preview_lane() {
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
        .support_profile(
            WorthQueryRuntimeSupportProfile::bridge_backed(
                "test-subscription-activation",
                "test-preview-basis",
                "test-inspector-evidence",
            )
            .with_family_support(WorthQueryRuntimeFamilySupport::supported(
                WorthQueryRuntimeFacadeFamily::Intent,
                [WorthQueryAuthorityLane::AuthoritativeTruth],
                [],
                ["test-intent-authority"],
            )),
        )
        .build_backend_from_parts()
        .build()
        .expect("runtime can support authoritative-only intents");

    let error = {
        let mut preview = runtime
            .preview_with_options(
                test_session_label("preview lane unsupported"),
                WorthQueryPreviewOptions::sandboxed_write_intent(),
            )
            .expect("preview session should be admitted");
        preview
            .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
                "preview-lane-denied",
                "strategy.intent.reconcile",
                "1.0",
                "intent.reconcile.input.v1",
                test_intent_input([("entity", "task-1")]),
            ))
            .expect_err("preview-local intent requires preview support metadata")
    };

    match error {
        WorthQueryRuntimeError::UnsupportedFacadeFamily(denial) => {
            assert_eq!(denial.family(), WorthQueryRuntimeFacadeFamily::Intent);
            assert!(denial.reason().contains("preview-truth"));
        }
        other => panic!("expected preview lane support denial, got {other:?}"),
    }
    assert_eq!(
        attempted.get(),
        0,
        "preview lane support denial must happen before authoritative intent authority"
    );
}
