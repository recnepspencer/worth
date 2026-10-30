use {
    crate::shared_support_compat_http_phase_ten_assertions::assert_cacheability_matches_read_policy,
    crate::shared_support_compat_http_phase_ten_assertions::assert_private_cacheability,
};
use {
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::fixtures::canonical_upload,
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::outcomes::compat_download_success,
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::outcomes::compat_inspection_success,
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::outcomes::compat_read_success,
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::outcomes::compat_upload_success,
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::requests::compat_download_execution_input,
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::requests::compat_inspection_execution_input,
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::requests::compat_read_execution_input,
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::requests::compat_upload_execution_input,
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::server::build_phase_ten_server_with_workspace_provider,
};

#[test]
fn compat_http_marks_metadata_and_transfer_surfaces_as_intermediary_unsafe() {
    let server =
        build_phase_ten_server_with_workspace_provider(worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));

    let upload = compat_upload_success(server.compat_http().upload(compat_upload_execution_input(
        &server,
        "tenant-a",
        "workspace-42",
        "branch-private",
        "files.asset",
        worth_foundational::facade::DiagnosticRichnessProfile::Forensic,
        canonical_upload("cacheability-phase-ten"),
    )));
    let read = compat_read_success(server.compat_http().read(compat_read_execution_input(
        &server,
        "tenant-a",
        "workspace-42",
        "branch-private",
        "files.asset",
        worth_foundational::facade::DiagnosticRichnessProfile::Forensic,
    )));
    let inspection = compat_inspection_success(server.compat_http().inspect(
        compat_inspection_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-private",
            "files.asset",
            worth_foundational::facade::DiagnosticRichnessProfile::Forensic,
        ),
    ));
    let download = compat_download_success(server.compat_http().download(
        compat_download_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-private",
            "files.asset",
            worth_foundational::facade::DiagnosticRichnessProfile::Forensic,
            worth_server::WorthServerBinaryDownloadRequest::new(b"hello-world".to_vec()),
        ),
    ));

    assert_private_cacheability(upload.file_envelope(), "upload_ingress");
    assert_private_cacheability(read.file_envelope(), "metadata_read");
    assert_private_cacheability(inspection.file_envelope(), "metadata_inspection");
    assert_private_cacheability(download.file_envelope(), "binary_egress");
    assert_cacheability_matches_read_policy(&read);
}
