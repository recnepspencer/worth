use super::support::*;

fn branch_intent_runtime(attempted: std::rc::Rc<std::cell::Cell<usize>>) -> WorthQueryRuntime {
    test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .intent_authority(CountingIntentAuthority { attempted })
        .support_profile(intent_support_profile())
        .build_backend_from_parts()
        .build()
        .expect("intent-capable runtime should build")
}

fn branch_intent_input(
    fields: impl IntoIterator<Item = (&'static str, &'static str)>,
) -> WorthQueryIntentInput {
    WorthQueryIntentInput::object(
        fields
            .into_iter()
            .map(|(field, value)| (field, WorthQueryIntentInput::string(value))),
    )
}

#[test]
fn branch_local_intent_is_policy_admitted_without_authoritative_execution() {
    let attempted = std::rc::Rc::new(std::cell::Cell::new(0));
    let mut runtime = branch_intent_runtime(attempted.clone());

    let mut branch = runtime
        .branch_with_options(
            test_session_label("branch local intent"),
            WorthQueryBranchOptions::sandboxed_write_intent(),
        )
        .expect("branch session should be admitted");
    let receipt = branch
        .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
            "branch-reconcile",
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            branch_intent_input([("entity", "task-1"), ("title", "branch title")]),
        ))
        .expect("sandboxed branch intent should be admitted");

    assert_eq!(
        branch.label(),
        test_session_label("branch local intent").display()
    );
    assert_eq!(
        branch.basis_admission().authority_lane(),
        WorthQueryAuthorityLane::BranchLocalTruth
    );
    assert_eq!(receipt.intent_name(), "branch-reconcile");
    assert_eq!(receipt.strategy_identity(), "strategy.intent.reconcile");
    assert_eq!(receipt.strategy_version(), "1.0");
    assert_eq!(
        receipt.source_lane(),
        WorthQueryIntentSourceLane::BranchLocal
    );
    assert_eq!(
        receipt.target_lane(),
        WorthQueryAuthorityLane::BranchLocalTruth
    );
    assert_eq!(
        receipt.effect_policy(),
        WorthQueryEffectPolicy::SandboxedWriteIntent
    );
    assert!(!receipt.basis_evidence().is_empty());
    assert!(!receipt
        .basis_snapshot_identity()
        .evidence_identity()
        .as_str()
        .is_empty());
    assert!(!receipt.admission_identity().as_str().is_empty());
    assert!(!receipt.receipt_digest().is_empty());
    assert_eq!(
        branch.branch_intent_receipts(),
        std::slice::from_ref(&receipt)
    );
    assert_eq!(
        attempted.get(),
        0,
        "branch-local intent staging must not execute authoritative intent authority"
    );

    let inspection = runtime
        .inspect_branch_intent_receipt(&receipt)
        .expect("branch intent inspection should succeed");
    assert_eq!(inspection.intent_name(), "branch-reconcile");
    assert_eq!(
        inspection.source_lane(),
        WorthQueryIntentSourceLane::BranchLocal
    );
    assert_eq!(
        inspection.target_lane(),
        WorthQueryAuthorityLane::BranchLocalTruth
    );
    assert_eq!(
        inspection.effect_policy(),
        WorthQueryEffectPolicy::SandboxedWriteIntent
    );
    assert!(!inspection.basis_digest().is_empty());
    assert_eq!(
        inspection.basis_snapshot_identity(),
        receipt.basis_snapshot_identity()
    );
    assert!(!inspection.inspection_digest().is_empty());
}

#[test]
fn derive_only_branch_intent_denies_before_authoritative_execution() {
    let attempted = std::rc::Rc::new(std::cell::Cell::new(0));
    let mut runtime = branch_intent_runtime(attempted.clone());

    let error = {
        let mut branch = runtime
            .branch(test_session_label("derive-only branch intent"))
            .expect("branch session should be admitted");
        branch
            .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
                "branch-denied",
                "strategy.intent.reconcile",
                "1.0",
                "intent.reconcile.input.v1",
                branch_intent_input([("entity", "task-1")]),
            ))
            .expect_err("derive-only branch must deny write intents")
    };

    match error {
        WorthQueryRuntimeError::IntentCommitDenied {
            intent_name,
            stage,
            message,
            evidence: _,
        } => {
            assert_eq!(intent_name, "branch-denied");
            assert_eq!(stage, "branch-effect-policy-admission");
            assert!(message.contains("sandboxed-write-intent"));
            assert!(message.contains("derive-only"));
        }
        other => panic!("expected branch policy intent denial, got {other:?}"),
    }
    assert_eq!(
        attempted.get(),
        0,
        "branch policy denial must happen before authoritative intent authority"
    );
}

#[test]
fn branch_local_intent_requires_intent_support_for_branch_lane() {
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
        let mut branch = runtime
            .branch_with_options(
                test_session_label("branch lane unsupported"),
                WorthQueryBranchOptions::sandboxed_write_intent(),
            )
            .expect("branch session should be admitted");
        branch
            .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
                "branch-lane-denied",
                "strategy.intent.reconcile",
                "1.0",
                "intent.reconcile.input.v1",
                branch_intent_input([("entity", "task-1")]),
            ))
            .expect_err("branch-local intent requires branch-local support metadata")
    };

    match error {
        WorthQueryRuntimeError::UnsupportedFacadeFamily(denial) => {
            assert_eq!(denial.family(), WorthQueryRuntimeFacadeFamily::Intent);
            assert!(denial.reason().contains("branch-local-truth"));
        }
        other => panic!("expected branch lane support denial, got {other:?}"),
    }
    assert_eq!(
        attempted.get(),
        0,
        "branch lane support denial must happen before authoritative intent authority"
    );
}
