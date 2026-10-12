//! runtime builder accepts bridge backed backend parts.
use super::*;

#[test]
fn runtime_builder_accepts_bridge_backed_backend_parts() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .write_authority(TestWriteAuthority)
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .aspect_contracts(stateful_bridge_aspect_contracts())
        .expect("native test aspect contracts should admit")
        .build_backend_from_parts()
        .build(request_execution)
        .expect("complete backend parts should build");
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "external.tasks",
            task_live_request(),
            task_schema(),
        )
        .expect("external backend should declare live view");
    let receipt = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("external-1")),
                    ("title.value", test_string_aspect_value("External task")),
                ],
            ),
        )
        .expect("external write authority should execute");

    assert_eq!(view.name(), "external.tasks");
    assert_eq!(
        view.subscription_installation().subscription_family(),
        "collection_membership"
    );
    assert_eq!(
        view.subscription_installation().authority_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert_eq!(
        view.subscription_installation()
            .counters()
            .activation_input_count(),
        1
    );
    assert!(view
        .subscription_installation()
        .support_projection()
        .label()
        .starts_with("worth.query.evidence-identity.v1:"));
    assert!(!view
        .subscription_installation()
        .active_lane_projection()
        .label()
        .is_empty());
    assert!(!view
        .subscription_installation()
        .consumer_attachment_projection()
        .label()
        .is_empty());
    assert!(!view
        .subscription_installation()
        .consumer_projection()
        .label()
        .is_empty());
    assert!(!view
        .subscription_installation()
        .delivery_cursor_projection()
        .label()
        .is_empty());
    assert_eq!(
        view.subscription_installation()
            .active_lane_counters()
            .active_lane_creation_count(),
        1
    );
    assert_eq!(
        view.subscription_installation()
            .consumer_attachment_counters()
            .consumer_attachment_count(),
        1
    );
    assert_eq!(
        view.subscription_installation()
            .subscription_budget_policy(),
        runtime_subscription_budget_policy().policy_label()
    );
    assert_eq!(
        view.subscription_installation()
            .active_lifecycle_budget_policy(),
        RUNTIME_ACTIVE_LIFECYCLE_BUDGET_POLICY
    );
    assert_eq!(
        view.subscription_installation()
            .consumer_attachment_budget_policy(),
        RUNTIME_CONSUMER_ATTACHMENT_BUDGET_POLICY
    );
    assert_eq!(
        view.subscription_installation().runtime_budget_identity(),
        &runtime_subscription_budget_digest()
    );
    let live_inspection = runtime
        .inspect_live_view(&view)
        .expect("inspector should retain live subscription installation");
    assert_eq!(
        live_inspection.installation_projection().label(),
        view.subscription_installation()
            .installation_projection()
            .label()
    );
    assert_eq!(
        receipt
            .commit_identity()
            .bridge_identity()
            .and_then(|identity| identity.relational_commit_id()),
        Some(1)
    );
    assert_eq!(
        receipt.terminal_affected_live_view_ids_projection(),
        &["external.tasks".to_string()]
    );
    {
        let inspector = runtime
            .try_inspect_receipt(&receipt)
            .expect("inspector evidence adapter should inspect receipt");
        assert_eq!(
            inspector.runtime_evidence().artifact_family(),
            "test-write-receipt"
        );
        assert_eq!(
            inspector.runtime_evidence().evidence(),
            &["test-inspector-evidence".to_string()]
        );
    }
    {
        let preview = runtime
            .try_preview(request_execution, test_session_label("external preview"))
            .expect("preview basis adapter should admit preview basis");
        assert_eq!(
            preview.basis_admission().label(),
            test_session_label("external preview").display()
        );
        assert_eq!(
            preview.basis_admission().evidence(),
            vec!["test-preview-basis".to_string()]
        );
    }
}
