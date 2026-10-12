use super::super::support::*;
#[test]
fn runtime_builder_rejects_missing_backend_inputs() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let error =
        match WorthQueryRuntime::builder(test_product_world_resources()).build(request_execution) {
            Ok(_) => panic!("builder should reject missing v1 backend"),
            Err(error) => error,
        };

    assert!(matches!(error, WorthQueryRuntimeError::MissingBackend));
    assert!(error.to_string().contains("build_backend_from_parts()"));
    assert!(error.to_string().contains("backend(...) only"));
}

#[test]
fn runtime_builder_rejects_incomplete_backend_parts() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let error = WorthQueryRuntime::builder(test_product_world_resources())
        .build_backend_from_parts()
        .build(request_execution);
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
        .build(request_execution);
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
        .build(request_execution);
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
        .build(request_execution);
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
        .build(request_execution);
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
        .build(request_execution);
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
        .build(request_execution);
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
        .build(request_execution);
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
        .build(request_execution);
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

#[path = "builder/runtime_builder_accepts_bridge_backed_backend_parts.rs"]
mod runtime_builder_accepts_bridge_backed_backend_parts;

#[test]
fn runtime_builder_rejects_replacing_explicit_backend_with_backend_parts() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

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
        .build(request_execution);
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
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

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
        .build(request_execution);
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
