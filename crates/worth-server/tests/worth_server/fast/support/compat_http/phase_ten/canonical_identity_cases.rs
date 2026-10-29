use worth_foundational::facade::DiagnosticRichnessProfile;
use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
use worth_server::WorthServerBinaryDownloadRequest;

use crate::shared_support_compat_http_phase_ten_assertions::assert_same_canonical_file_identity;
use crate::shared_support_compat_http_phase_ten_runtime::{
    build_phase_ten_server_with_workspace_provider, canonical_upload,
    compat_download_execution_input, compat_download_success, compat_inspection_execution_input,
    compat_inspection_success, compat_read_execution_input, compat_read_success,
    compat_upload_execution_input, compat_upload_success, reordered_canonical_upload,
};
use crate::shared_support_query_handoff_runtime::ProfiledTestWorkspaceProvider;

#[test]
fn compat_http_phase_ten_keeps_one_canonical_external_file_identity_across_lanes() {
    let server =
        build_phase_ten_server_with_workspace_provider(ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));

    let upload = compat_upload_success(server.compat_http().upload(compat_upload_execution_input(
        &server,
        "tenant-a",
        "workspace-42",
        "branch-10",
        " files.asset ",
        DiagnosticRichnessProfile::Standard,
        canonical_upload("phase-ten-file"),
    )));
    let read = compat_read_success(server.compat_http().read(compat_read_execution_input(
        &server,
        "tenant-a",
        "workspace-42",
        "branch-10",
        " files.asset ",
        DiagnosticRichnessProfile::Standard,
    )));
    let inspection = compat_inspection_success(server.compat_http().inspect(
        compat_inspection_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-10",
            " files.asset ",
            DiagnosticRichnessProfile::Standard,
        ),
    ));
    let download = compat_download_success(server.compat_http().download(
        compat_download_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-10",
            " files.asset ",
            DiagnosticRichnessProfile::Standard,
            WorthServerBinaryDownloadRequest::new(b"hello-world".to_vec()),
        ),
    ));

    assert_eq!(
        upload.file_envelope().canonical_filename().canonical(),
        "files.asset"
    );
    assert_eq!(
        read.file_envelope().canonical_filename().canonical(),
        "files.asset"
    );
    assert_same_canonical_file_identity(upload.file_envelope(), read.file_envelope());
    assert_same_canonical_file_identity(read.file_envelope(), inspection.file_envelope());
    assert_same_canonical_file_identity(inspection.file_envelope(), download.file_envelope());

    assert_eq!(
        upload
            .file_envelope()
            .metadata_normalization_receipt()
            .source_kind(),
        "inline_manifest"
    );
    assert_eq!(
        read.file_envelope()
            .metadata_normalization_receipt()
            .source_kind(),
        "observed_truth"
    );
}

#[test]
fn compat_http_phase_ten_collapses_legal_metadata_encoding_variation_to_one_normalization_shape() {
    let server =
        build_phase_ten_server_with_workspace_provider(ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));

    let canonical =
        compat_upload_success(server.compat_http().upload(compat_upload_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-10",
            "files.asset",
            DiagnosticRichnessProfile::Standard,
            canonical_upload("phase-ten-shared"),
        )));
    let reordered =
        compat_upload_success(server.compat_http().upload(compat_upload_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-10",
            "files.asset",
            DiagnosticRichnessProfile::Standard,
            reordered_canonical_upload("phase-ten-shared"),
        )));

    assert_eq!(
        canonical
            .file_envelope()
            .metadata_normalization_receipt()
            .normalized_key_paths(),
        reordered
            .file_envelope()
            .metadata_normalization_receipt()
            .normalized_key_paths()
    );
    assert_same_canonical_file_identity(canonical.file_envelope(), reordered.file_envelope());
}

#[test]
fn compat_http_phase_ten_collapses_case_variation_into_one_metadata_identity() {
    let server =
        build_phase_ten_server_with_workspace_provider(ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));

    let upper = compat_upload_success(server.compat_http().upload(compat_upload_execution_input(
        &server,
        "tenant-a",
        "workspace-42",
        "branch-10",
        "FILES.ASSET",
        DiagnosticRichnessProfile::Standard,
        canonical_upload("phase-ten-case"),
    )));
    let lower = compat_upload_success(server.compat_http().upload(compat_upload_execution_input(
        &server,
        "tenant-a",
        "workspace-42",
        "branch-10",
        "files.asset",
        DiagnosticRichnessProfile::Standard,
        canonical_upload("phase-ten-case"),
    )));

    assert_same_canonical_file_identity(upper.file_envelope(), lower.file_envelope());
}
