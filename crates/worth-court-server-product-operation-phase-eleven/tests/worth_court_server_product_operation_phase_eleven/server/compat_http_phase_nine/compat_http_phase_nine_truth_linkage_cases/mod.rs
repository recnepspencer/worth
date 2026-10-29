use {worth_foundational::facade::DiagnosticRichnessProfile};
use {worth_query::facade::runtime::WorthQueryRuntimeSupportProfile};
use {worth_server::WorthServerBinaryDownloadRequest};

use {worth_testkit_server::shared_support_compat_http_phase_nine_assertions::assert_policy_alignment, worth_testkit_server::shared_support_compat_http_phase_nine_assertions::assert_same_metadata_identity, worth_testkit_server::shared_support_compat_http_phase_nine_assertions::assert_transfer_disposition};
use {worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::server::build_phase_nine_server_with_workspace_provider, worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::fixtures::canonical_upload, worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::requests::compat_download_execution_input, worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::outcomes::compat_download_success, worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::requests::compat_inspection_execution_input, worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::outcomes::compat_inspection_success, worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::requests::compat_read_execution_input, worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::outcomes::compat_read_success, worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::requests::compat_upload_execution_input, worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::outcomes::compat_upload_success};
use {worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider};

#[test]
fn compat_http_file_metadata_truth_linkage_stays_canonical_across_upload_read_inspect_and_download()
{
    let server =
        build_phase_nine_server_with_workspace_provider(worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));

    let upload = compat_upload_success(server.compat_http().upload(compat_upload_execution_input(
        &server,
        "tenant-a",
        "workspace-42",
        "branch-9",
        "files.asset",
        worth_foundational::facade::DiagnosticRichnessProfile::Standard,
        canonical_upload("phase-nine-file"),
    )));
    let read = compat_read_success(server.compat_http().read(compat_read_execution_input(
        &server,
        "tenant-a",
        "workspace-42",
        "branch-9",
        "files.asset",
        worth_foundational::facade::DiagnosticRichnessProfile::Standard,
    )));
    let inspection = compat_inspection_success(server.compat_http().inspect(
        compat_inspection_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            "files.asset",
            worth_foundational::facade::DiagnosticRichnessProfile::Standard,
        ),
    ));
    let download = compat_download_success(server.compat_http().download(
        compat_download_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            "files.asset",
            worth_foundational::facade::DiagnosticRichnessProfile::Standard,
            worth_server::WorthServerBinaryDownloadRequest::new(b"hello-world".to_vec()),
        ),
    ));

    assert_same_metadata_identity(upload.file_envelope(), read.file_envelope());
    assert_same_metadata_identity(read.file_envelope(), inspection.file_envelope());
    assert_same_metadata_identity(inspection.file_envelope(), download.file_envelope());

    assert!(upload.file_envelope().metadata_receipt().truth_committed());
    assert!(read.file_envelope().metadata_receipt().truth_observed());
    assert!(inspection
        .file_envelope()
        .metadata_receipt()
        .truth_observed());
    assert!(download.file_envelope().metadata_receipt().truth_observed());

    assert_transfer_disposition(
        upload.file_envelope(),
        worth_server::WorthServerFileTransferDisposition::VerifiedIngress,
    );
    assert_eq!(
        upload.file_envelope().transfer_provenance().content_type(),
        None
    );
    assert_transfer_disposition(
        read.file_envelope(),
        worth_server::WorthServerFileTransferDisposition::MetadataOnlyObservation,
    );
    assert_transfer_disposition(
        inspection.file_envelope(),
        worth_server::WorthServerFileTransferDisposition::MetadataOnlyObservation,
    );
    assert_transfer_disposition(
        download.file_envelope(),
        worth_server::WorthServerFileTransferDisposition::SelectedEgress,
    );

    assert_policy_alignment(upload.file_envelope());
    assert_policy_alignment(read.file_envelope());
    assert_policy_alignment(inspection.file_envelope());
    assert_policy_alignment(download.file_envelope());
}

#[test]
fn compat_http_head_downloads_do_not_claim_byte_motion_in_file_transfer_provenance() {
    let server =
        build_phase_nine_server_with_workspace_provider(worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));

    let head_download = compat_download_success(
        server
            .compat_http()
            .download(worth_server::WorthServerBinaryDownloadExecutionInput::new(
                worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::requests::prepared_request(
                    &server,
                    worth_server::WorthServerCompatibilityRequestInput::builder()
                        .with_authenticated_principal_id("principal-9")
                        .with_tenant_id("tenant-a")
                        .with_workspace_id("workspace-42")
                        .with_branch_id("branch-9")
                        .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Download)
                        .with_method("HEAD")
                        .with_path("/compat/downloads/files.asset")
                        .with_header("accept", "application/octet-stream")
                        .with_diagnostics_profile(worth_foundational::facade::DiagnosticRichnessProfile::Standard)
                        .build()
                        .expect("HEAD file download input should validate"),
                ),
                "files.asset",
                worth_server::WorthServerBinaryDownloadRequest::new(b"hello-world".to_vec()),
            )),
    );

    assert_transfer_disposition(
        head_download.file_envelope(),
        worth_server::WorthServerFileTransferDisposition::HeadOnlyEgress,
    );
    assert_eq!(
        head_download
            .file_envelope()
            .transfer_provenance()
            .bytes_selected(),
        0
    );
}
