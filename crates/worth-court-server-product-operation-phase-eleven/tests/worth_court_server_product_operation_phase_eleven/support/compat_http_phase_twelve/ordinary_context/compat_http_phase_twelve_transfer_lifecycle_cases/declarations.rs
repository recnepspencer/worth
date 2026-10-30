pub(crate) use serde_json::json;
pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_server::WorthServerBinaryDownloadRequest, worth_server::WorthServerCompatHttpRouteFamily,
    worth_server::WorthServerTransferByteClass, worth_server::WorthServerTransferCleanupReason,
};

pub(crate) use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
pub(crate) use {
    worth_testkit_server::shared_support_compat_http_phase_eight_runtime::build_phase_eight_server_with_workspace_provider,
    worth_testkit_server::shared_support_compat_http_phase_eight_runtime::compat_download_success,
    worth_testkit_server::shared_support_compat_http_phase_eight_runtime::compat_resume_success,
    worth_testkit_server::shared_support_compat_http_phase_eight_runtime::download_input as resume_download_input,
    worth_testkit_server::shared_support_compat_http_phase_eight_runtime::prepared_request as resume_prepared_request,
    worth_testkit_server::shared_support_compat_http_phase_four_runtime::request_support::build_phase_four_server,
    worth_testkit_server::shared_support_compat_http_phase_four_runtime::request_support::compat_stream_input,
    worth_testkit_server::shared_support_compat_http_phase_four_runtime::request_support::default_stream_selection,
    worth_testkit_server::shared_support_compat_http_phase_four_runtime::request_support::streaming_response_success,
    worth_testkit_server::shared_support_compat_http_phase_twelve_assertions::assert_binary_counter,
    worth_testkit_server::shared_support_compat_http_phase_twelve_assertions::assert_cleanup_evidence,
    worth_testkit_server::shared_support_compat_http_phase_twelve_assertions::assert_cleanup_scope,
    worth_testkit_server::shared_support_compat_http_phase_twelve_assertions::assert_external_counter,
    worth_testkit_server::shared_support_compat_http_phase_twelve_runtime::build_phase_twelve_server,
    worth_testkit_server::shared_support_compat_http_phase_twelve_runtime::upload_input,
    worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider,
};

pub(crate) fn canonical_upload(identity: &str) -> worth_server::WorthServerMultipartUpload {
    worth_server::WorthServerMultipartUpload::new(
        worth_server::WorthServerUploadManifest::new(json!({
            "command": {
                "family": "insert",
                "collection": "Task",
                "aspects": {
                    "identity.id": identity,
                    "title.value": format!("Cleanup {identity}")
                }
            }
        }))
        .with_file_part("blob"),
    )
    .with_part(
        worth_server::WorthServerUploadPart::file("blob")
            .with_content_type("application/octet-stream")
            .with_declared_length(11)
            .with_body_bytes(b"hello-world".to_vec()),
    )
}
