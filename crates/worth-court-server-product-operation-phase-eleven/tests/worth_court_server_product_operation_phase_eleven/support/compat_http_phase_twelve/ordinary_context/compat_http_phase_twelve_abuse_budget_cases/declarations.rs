pub(crate) use {serde_json::json};
pub(crate) use {worth_foundational::facade::DiagnosticRichnessProfile};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_server::WorthServerBinaryDownloadRequest, worth_server::WorthServerCompatHttpRouteFamily, worth_server::WorthServerTransferByteClass};

pub(crate) use {worth_testkit_server::shared_support_compat_http_phase_twelve_assertions::assert_binary_counter, worth_testkit_server::shared_support_compat_http_phase_twelve_assertions::assert_budget_receipt, worth_testkit_server::shared_support_compat_http_phase_twelve_assertions::assert_budget_scope, worth_testkit_server::shared_support_compat_http_phase_twelve_assertions::assert_external_counter, worth_testkit_server::shared_support_compat_http_phase_twelve_runtime::build_phase_twelve_budget_limited_server, worth_testkit_server::shared_support_compat_http_phase_twelve_runtime::build_phase_twelve_server, worth_testkit_server::shared_support_compat_http_phase_twelve_runtime::download_input, worth_testkit_server::shared_support_compat_http_phase_twelve_runtime::head_read_request, worth_testkit_server::shared_support_compat_http_phase_twelve_runtime::prepared_request, worth_testkit_server::shared_support_compat_http_phase_twelve_runtime::read_request, worth_testkit_server::shared_support_compat_http_phase_twelve_runtime::upload_input};

pub(crate) fn canonical_upload(identity: &str) -> worth_server::WorthServerMultipartUpload {
    worth_server::WorthServerMultipartUpload::new(
        worth_server::WorthServerUploadManifest::new(json!({
            "command": {
                "family": "insert",
                "collection": "Task",
                "aspects": {
                    "identity.id": identity,
                    "title.value": format!("Title for {identity}")
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