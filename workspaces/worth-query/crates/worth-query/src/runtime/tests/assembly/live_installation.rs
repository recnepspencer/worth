use super::super::support::*;

#[test]
fn runtime_live_declaration_denies_backend_admission_before_subscription_install() {
    let source_declarations = std::rc::Rc::new(std::cell::Cell::new(0));
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(DenyingSchemaAdapter)
        .source_adapter(CountingSourceAdapter::new(source_declarations.clone()))
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .build_backend_from_parts()
        .build()
        .expect("backend with denying schema admission should still build");

    let error = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            "external.schema-denied",
            task_live_request(),
            task_schema(),
        )
        .expect_err("backend admission denial must block subscription installation");

    assert_live_subscription_installation_error(
        error,
        "external.schema-denied",
        "backend-live-admission",
        "schema admission denied by test adapter",
    );
    assert_eq!(source_declarations.get(), 0);
    assert_no_live_subscription_residue(&runtime);
}

#[test]
fn runtime_live_declaration_closes_active_subscription_when_source_declaration_fails() {
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::fail_declare())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .build_backend_from_parts()
        .build()
        .expect("backend with failing source declaration should still build");

    let error = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            "external.source-denied",
            task_live_request(),
            task_schema(),
        )
        .expect_err("source declaration denial must close active subscription");

    match error {
        WorthQueryRuntimeError::LiveSubscriptionInstallation {
            view_name,
            stage,
            message,
        } => {
            assert_eq!(view_name, "external.source-denied");
            assert_eq!(stage, "source-declaration");
            assert!(message.contains("source declaration denied by test adapter"));
            assert!(message.contains("active subscription closeout:"));
            assert!(message.contains("terminal:true"));
        }
        other => panic!("expected source declaration denial, got {other:?}"),
    }
    assert_eq!(runtime.active_subscriptions.lane_count(), 0);
    assert!(runtime.live_subscriptions.is_empty());
}

#[test]
fn runtime_equivalent_live_declarations_share_active_lane_with_distinct_consumers() {
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .build_backend_from_parts()
        .build()
        .expect("complete backend parts should build");

    let first: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view("external.tasks.first", task_live_request(), task_schema())
        .expect("first live view should install active lane");
    let second: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view("external.tasks.second", task_live_request(), task_schema())
        .expect("equivalent live view should join active lane");

    assert_eq!(
        first
            .subscription_installation()
            .active_lane_projection()
            .label(),
        second
            .subscription_installation()
            .active_lane_projection()
            .label()
    );
    assert_ne!(
        first
            .subscription_installation()
            .consumer_attachment_projection()
            .label(),
        second
            .subscription_installation()
            .consumer_attachment_projection()
            .label()
    );
    assert_eq!(
        second
            .subscription_installation()
            .active_lane_counters()
            .active_lane_join_count(),
        1
    );
    assert_eq!(
        second
            .subscription_installation()
            .active_lane_counters()
            .shared_lane_count(),
        1
    );
    assert_eq!(
        second
            .subscription_installation()
            .consumer_attachment_counters()
            .consumer_attachment_count(),
        1
    );
    assert_eq!(
        second
            .subscription_installation()
            .consumer_attachment_counters()
            .affected_consumer_attachment_width(),
        2
    );
}

#[test]
fn runtime_live_declaration_denies_before_source_when_subscription_activation_rejects() {
    let source_declarations = std::rc::Rc::new(std::cell::Cell::new(0));
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(CountingSourceAdapter::new(source_declarations.clone()))
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(DenyingSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .build_backend_from_parts()
        .build()
        .expect("backend with denying activation should still build");

    let error = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            "external.denied",
            task_live_request(),
            task_schema(),
        )
        .expect_err("activation denial must block source declaration");

    assert_live_subscription_installation_error(
        error,
        "external.denied",
        "activation-admission",
        "activation denied by test adapter",
    );
    assert_eq!(source_declarations.get(), 0);
    assert_no_live_subscription_residue(&runtime);
}

#[test]
fn runtime_live_declaration_denies_when_admission_receipt_drifts_from_request() {
    let source_declarations = std::rc::Rc::new(std::cell::Cell::new(0));
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(DriftingSchemaReceiptAdapter)
        .source_adapter(CountingSourceAdapter::new(source_declarations.clone()))
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .build_backend_from_parts()
        .build()
        .expect("backend with drifting schema receipt should still build");

    let error = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            "external.receipt-drift",
            task_live_request(),
            task_schema(),
        )
        .expect_err("admission receipt drift must deny live declaration");

    assert_live_subscription_installation_error(
        error,
        "external.receipt-drift",
        "backend-live-admission-receipt",
        "drifted",
    );
    assert_eq!(source_declarations.get(), 0);
    assert_no_live_subscription_residue(&runtime);
}

#[test]
fn runtime_live_declaration_denies_when_activation_receipt_drifts_from_request() {
    let source_declarations = std::rc::Rc::new(std::cell::Cell::new(0));
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(CountingSourceAdapter::new(source_declarations.clone()))
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(DriftingSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .build_backend_from_parts()
        .build()
        .expect("backend with drifting activation receipt should still build");

    let error = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            "external.activation-receipt-drift",
            task_live_request(),
            task_schema(),
        )
        .expect_err("activation receipt drift must deny live declaration");

    assert_live_subscription_installation_error(
        error,
        "external.activation-receipt-drift",
        "activation-admission",
        "drifted",
    );
    assert_eq!(source_declarations.get(), 0);
    assert_no_live_subscription_residue(&runtime);
}

fn assert_live_subscription_installation_error(
    error: WorthQueryRuntimeError,
    expected_view_name: &str,
    expected_stage: &'static str,
    expected_message_fragment: &str,
) {
    match error {
        WorthQueryRuntimeError::LiveSubscriptionInstallation {
            view_name,
            stage,
            message,
        } => {
            assert_eq!(view_name, expected_view_name);
            assert_eq!(stage, expected_stage);
            assert!(message.contains(expected_message_fragment));
        }
        other => panic!("expected live subscription installation denial, got {other:?}"),
    }
}

fn assert_no_live_subscription_residue(runtime: &WorthQueryRuntime) {
    assert_eq!(runtime.active_subscriptions.lane_count(), 0);
    assert!(runtime.live_subscriptions.is_empty());
}
