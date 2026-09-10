use super::super::support::*;
#[test]
fn runtime_builder_rejects_missing_backend_inputs() {
    let error = match WorthQueryRuntime::builder(test_product_world_resources()).build() {
        Ok(_) => panic!("builder should reject missing v1 backend"),
        Err(error) => error,
    };

    assert!(matches!(error, WorthQueryRuntimeError::MissingBackend));
    assert!(error.to_string().contains("build_backend_from_parts()"));
    assert!(error.to_string().contains("backend(...) only"));
}

#[test]
fn runtime_builder_rejects_incomplete_backend_parts() {
    let error = WorthQueryRuntime::builder(test_product_world_resources())
        .build_backend_from_parts()
        .build();
    let error = match error {
        Ok(_) => panic!("missing bridge should reject"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        WorthQueryRuntimeError::MissingRuntimeBridge
    ));
    assert!(error.to_string().contains("runtime_bridge(...)"));
    assert!(error.to_string().contains("build_backend_from_parts()"));

    let error = test_product_runtime_builder()
        .build_backend_from_parts()
        .build();
    let error = match error {
        Ok(_) => panic!("missing schema adapter should reject"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        WorthQueryRuntimeError::MissingSchemaAdapter
    ));
    assert!(error.to_string().contains("schema_adapter(...)"));

    let error = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .build_backend_from_parts()
        .build();
    let error = match error {
        Ok(_) => panic!("missing source adapter should reject"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        WorthQueryRuntimeError::MissingSourceAdapter
    ));
    assert!(error.to_string().contains("source_adapter(...)"));

    let error = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .build_backend_from_parts()
        .build();
    let error = match error {
        Ok(_) => panic!("missing snapshot identity adapter should reject"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        WorthQueryRuntimeError::MissingSnapshotIdentityAdapter
    ));
    assert!(error.to_string().contains("snapshot_identity(...)"));

    let error = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .build_backend_from_parts()
        .build();
    let error = match error {
        Ok(_) => panic!("missing write authority should reject"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        WorthQueryRuntimeError::MissingWriteAuthority
    ));
    assert!(error.to_string().contains("write_authority(...)"));

    let error = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .build_backend_from_parts()
        .build();
    let error = match error {
        Ok(_) => panic!("missing signal sink should reject"),
        Err(error) => error,
    };
    assert!(matches!(error, WorthQueryRuntimeError::MissingSignalSink));
    assert!(error.to_string().contains("signal_sink(...)"));

    let error = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .build_backend_from_parts()
        .build();
    let error = match error {
        Ok(_) => panic!("missing subscription activation should reject"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        WorthQueryRuntimeError::MissingSubscriptionActivation
    ));
    assert!(error.to_string().contains("subscription_activation(...)"));

    let error = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .build_backend_from_parts()
        .build();
    let error = match error {
        Ok(_) => panic!("missing preview basis should reject"),
        Err(error) => error,
    };
    assert!(matches!(error, WorthQueryRuntimeError::MissingPreviewBasis));
    assert!(error.to_string().contains("preview_basis(...)"));

    let error = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .write_authority(TestWriteAuthority)
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .build_backend_from_parts()
        .build();
    let error = match error {
        Ok(_) => panic!("missing inspector evidence should reject"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        WorthQueryRuntimeError::MissingInspectorEvidence
    ));
    assert!(error.to_string().contains("inspector_evidence(...)"));
}

#[test]
fn runtime_builder_accepts_bridge_backed_backend_parts() {
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
        .build()
        .expect("complete backend parts should build");
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view("external.tasks", task_live_request(), task_schema())
        .expect("external backend should declare live view");
    let receipt = runtime
        .write(insert_command(
            "Task",
            [
                ("identity.id", test_string_aspect_value("external-1")),
                ("title.value", test_string_aspect_value("External task")),
            ],
        ))
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
            .try_preview(test_session_label("external preview"))
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

#[test]
fn runtime_builder_rejects_replacing_explicit_backend_with_backend_parts() {
    let explicit_backend = WorthQueryBridgeBackedRuntimeBackend::from_parts(
        WorthQueryRuntimeBackendParts::new()
            .runtime_bridge(test_bridge())
            .schema_adapter(TestSchemaAdapter)
            .source_adapter(TestSourceAdapter::default())
            .write_authority(TestWriteAuthority)
            .snapshot_identity(TestSnapshotIdentityAdapter)
            .signal_sink(TestSignalSink)
            .subscription_activation(TestSubscriptionActivation)
            .preview_basis(TestPreviewBasis)
            .inspector_evidence(TestInspectorEvidence),
    )
    .expect("explicit backend should build for replacement test");
    let error = WorthQueryRuntime::builder(test_product_world_resources())
        .backend(explicit_backend)
        .runtime_bridge(test_bridge())
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .write_authority(TestWriteAuthority)
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .build_backend_from_parts()
        .build();
    let error = match error {
        Ok(_) => panic!("explicit backend replacement should reject"),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        WorthQueryRuntimeError::InvariantRegistration {
            stage: "runtime_backend_authority_selection",
            ..
        }
    ));
    assert!(error.to_string().contains("build_backend_from_parts()"));
    assert!(error.to_string().contains("backend(...)"));
}

#[test]
fn runtime_builder_rejects_explicit_backend_with_stray_backend_parts() {
    let explicit_backend = WorthQueryBridgeBackedRuntimeBackend::from_parts(
        WorthQueryRuntimeBackendParts::new()
            .runtime_bridge(test_bridge())
            .schema_adapter(TestSchemaAdapter)
            .source_adapter(TestSourceAdapter::default())
            .write_authority(TestWriteAuthority)
            .snapshot_identity(TestSnapshotIdentityAdapter)
            .signal_sink(TestSignalSink)
            .subscription_activation(TestSubscriptionActivation)
            .preview_basis(TestPreviewBasis)
            .inspector_evidence(TestInspectorEvidence),
    )
    .expect("explicit backend should build for stray-parts test");
    let error = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .backend(explicit_backend)
        .build();
    let error = match error {
        Ok(_) => panic!("explicit backend with stray backend parts should reject"),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        WorthQueryRuntimeError::InvariantRegistration {
            stage: "runtime_backend_authority_selection",
            ..
        }
    ));
    assert!(error.to_string().contains("runtime_bridge(...)"));
    assert!(error.to_string().contains("backend authority path"));
}
