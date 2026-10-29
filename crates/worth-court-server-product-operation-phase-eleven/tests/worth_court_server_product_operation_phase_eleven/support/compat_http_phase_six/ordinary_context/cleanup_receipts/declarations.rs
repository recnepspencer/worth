pub(crate) use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc};

pub(crate) use {worth_query::facade::runtime::WorthQueryRuntimeSupportProfile};
pub(crate) use {worth_server::WorthServerQueryHandoffDenialCode, worth_server::WorthServerUploadCleanupReason};

pub(crate) use {crate::compat_http_phase_six::ordinary_context::compat_http_phase_six_assertions::assert_cleanup_receipt, crate::compat_http_phase_six::ordinary_context::compat_http_phase_six_assertions::assert_counter, crate::compat_http_phase_six::ordinary_context::compat_http_phase_six_assertions::assert_upload_denial};
pub(crate) use {worth_testkit_server::shared_support_compat_http_phase_six_runtime::build_phase_six_server_with_workspace_provider, worth_testkit_server::shared_support_compat_http_phase_six_runtime::prepared_request, worth_testkit_server::shared_support_compat_http_phase_six_runtime::prepared_upload, worth_testkit_server::shared_support_compat_http_phase_six_runtime::upload_input};
pub(crate) use {worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledCountingTestWorkspaceProvider};
pub(crate) use {crate::compat_http_phase_six::ordinary_context::upload_fixtures::single_insert_body};

pub(crate) fn begin_session(
    server: &worth_server::WorthServer,
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    boundary: &str,
) -> worth_server::WorthServerBinaryIngressSession {
    let prepared = prepared_upload(
        server,
        tenant_id,
        workspace_id,
        branch_id,
        operation_name,
        boundary,
        cleanup_upload(operation_name),
    );
    match server.compat_http().begin_binary_ingress(prepared) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected binary ingress session, got {other:?}"),
    }
}

pub(crate) fn cleanup_upload(identity: &str) -> worth_server::WorthServerMultipartUpload {
    worth_server::WorthServerMultipartUpload::new(
        worth_server::WorthServerUploadManifest::new(single_insert_body(identity))
            .with_file_part("avatar"),
    )
    .with_expectation(worth_server::WorthServerUploadExpectation::continue_optional())
    .with_part(
        worth_server::WorthServerUploadPart::file("avatar")
            .with_content_type("image/png")
            .with_declared_length(8)
            .with_body_bytes(b"cleanup!".to_vec()),
    )
}