pub(crate) use worth_foundational::facade::DiagnosticRichnessProfile;
pub(crate) use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
pub(crate) use {
    worth_server::WorthServerBinaryDownloadRequest, worth_server::WorthServerStreamSelection,
    worth_server::WorthServerStreamingResponse,
};

pub(crate) use worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider;
pub(crate) use {
    crate::compat_http_phase_eleven::ordinary_context::compat_http_phase_eleven_assertions::assert_binary_counter,
    crate::compat_http_phase_eleven::ordinary_context::compat_http_phase_eleven_assertions::assert_external_counter,
    crate::compat_http_phase_eleven::ordinary_context::compat_http_phase_eleven_assertions::finish_stream,
};
pub(crate) use {
    worth_testkit_server::shared_support_compat_http_phase_four_runtime::request_support::build_phase_four_server,
    worth_testkit_server::shared_support_compat_http_phase_four_runtime::request_support::build_phase_four_server_with_workspace_provider,
    worth_testkit_server::shared_support_compat_http_phase_four_runtime::request_support::compat_stream_input,
    worth_testkit_server::shared_support_compat_http_phase_four_runtime::request_support::oversized_streaming_provider,
    worth_testkit_server::shared_support_compat_http_phase_four_runtime::request_support::streaming_response_success,
};
pub(crate) use {
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::fixtures::canonical_upload,
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::outcomes::compat_download_success,
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::outcomes::compat_read_success,
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::outcomes::compat_upload_success,
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::requests::compat_download_execution_input,
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::requests::compat_read_execution_input,
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::requests::compat_upload_execution_input,
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::server::build_phase_ten_server_with_workspace_provider,
};

pub(crate) fn phase_eleven_server() -> worth_server::WorthServer {
    build_phase_ten_server_with_workspace_provider(worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider::new(
        worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
    ))
}
