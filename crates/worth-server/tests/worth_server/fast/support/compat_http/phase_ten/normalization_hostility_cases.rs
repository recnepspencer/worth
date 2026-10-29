use worth_foundational::facade::DiagnosticRichnessProfile;
use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
use worth_server::{
    WorthServerBinaryDownloadExecutionInput, WorthServerBinaryDownloadRequest,
    WorthServerCompatHttpRouteFamily, WorthServerCompatibilityExecutionInput,
    WorthServerCompatibilityRequestInput, WorthServerQueryHandoffDenialCode,
};

use crate::shared_support_compat_http_phase_ten_assertions::assert_denial;
use crate::shared_support_compat_http_phase_ten_runtime::{
    ambiguous_metadata_upload, build_phase_ten_server_with_workspace_provider,
    compat_download_denied, compat_read_denied, compat_upload_denied,
    compat_upload_execution_input, non_ascii_metadata_upload, prepared_request,
};
use crate::shared_support_query_handoff_runtime::ProfiledTestWorkspaceProvider;

#[test]
fn compat_http_phase_ten_rejects_non_portable_external_names_before_truth_linkage() {
    let server =
        build_phase_ten_server_with_workspace_provider(ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));

    let read = compat_read_denied(
        server
            .compat_http()
            .read(WorthServerCompatibilityExecutionInput::new(
                prepared_request(
                    &server,
                    WorthServerCompatibilityRequestInput::builder()
                        .with_authenticated_principal_id("principal-10")
                        .with_tenant_id("tenant-a")
                        .with_workspace_id("workspace-42")
                        .with_branch_id("branch-10")
                        .with_route_family(WorthServerCompatHttpRouteFamily::Read)
                        .with_method("GET")
                        .with_path("/compat/reads/../secrets")
                        .with_header("accept", "application/json")
                        .with_diagnostics_profile(DiagnosticRichnessProfile::Standard)
                        .build()
                        .expect("path-like read input should validate structurally"),
                ),
                "../secrets",
            )),
    );
    assert_denial(
        &read,
        WorthServerQueryHandoffDenialCode::DirectDeclarationBindingInvalid,
        "path separators",
    );

    let download = compat_download_denied(
        server
            .compat_http()
            .download(WorthServerBinaryDownloadExecutionInput::new(
                prepared_request(
                    &server,
                    WorthServerCompatibilityRequestInput::builder()
                        .with_authenticated_principal_id("principal-10")
                        .with_tenant_id("tenant-a")
                        .with_workspace_id("workspace-42")
                        .with_branch_id("branch-10")
                        .with_route_family(WorthServerCompatHttpRouteFamily::Download)
                        .with_method("GET")
                        .with_path("/compat/downloads/folder\\asset")
                        .with_header("accept", "application/octet-stream")
                        .with_diagnostics_profile(DiagnosticRichnessProfile::Standard)
                        .build()
                        .expect("path-like download input should validate structurally"),
                ),
                "folder\\asset",
                WorthServerBinaryDownloadRequest::new(b"hello-world".to_vec()),
            )),
    );
    assert_denial(
        &download,
        WorthServerQueryHandoffDenialCode::CompatibilityDownloadRequestInvalid,
        "path separators",
    );

    let control_character = compat_read_denied(
        server
            .compat_http()
            .read(WorthServerCompatibilityExecutionInput::new(
                prepared_request(
                    &server,
                    WorthServerCompatibilityRequestInput::builder()
                        .with_authenticated_principal_id("principal-10")
                        .with_tenant_id("tenant-a")
                        .with_workspace_id("workspace-42")
                        .with_branch_id("branch-10")
                        .with_route_family(WorthServerCompatHttpRouteFamily::Read)
                        .with_method("GET")
                        .with_path("/compat/reads/files\tasset")
                        .with_header("accept", "application/json")
                        .with_diagnostics_profile(DiagnosticRichnessProfile::Standard)
                        .build()
                        .expect("control-character read input should validate structurally"),
                ),
                "files\tasset",
            )),
    );
    assert_denial(
        &control_character,
        WorthServerQueryHandoffDenialCode::DirectDeclarationBindingInvalid,
        "ASCII-printable",
    );
}

#[test]
fn compat_http_phase_ten_rejects_operation_name_and_path_identity_drift() {
    let server =
        build_phase_ten_server_with_workspace_provider(ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));

    let read = compat_read_denied(
        server
            .compat_http()
            .read(WorthServerCompatibilityExecutionInput::new(
                prepared_request(
                    &server,
                    WorthServerCompatibilityRequestInput::builder()
                        .with_authenticated_principal_id("principal-10")
                        .with_tenant_id("tenant-a")
                        .with_workspace_id("workspace-42")
                        .with_branch_id("branch-10")
                        .with_route_family(WorthServerCompatHttpRouteFamily::Read)
                        .with_method("GET")
                        .with_path("/compat/reads/files.asset")
                        .with_header("accept", "application/json")
                        .with_diagnostics_profile(DiagnosticRichnessProfile::Standard)
                        .build()
                        .expect("mismatched read input should validate structurally"),
                ),
                "other.asset",
            )),
    );
    assert_denial(
        &read,
        WorthServerQueryHandoffDenialCode::DirectDeclarationBindingInvalid,
        "did not match the external request path identity",
    );
}

#[test]
fn compat_http_phase_ten_rejects_ambiguous_or_non_ascii_manifest_metadata_keys() {
    let server =
        build_phase_ten_server_with_workspace_provider(ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));

    let ambiguous =
        compat_upload_denied(server.compat_http().upload(compat_upload_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-10",
            "files.asset",
            DiagnosticRichnessProfile::Standard,
            ambiguous_metadata_upload(),
        )));
    assert_denial(
        &ambiguous,
        WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
        "ambiguous keys",
    );

    let non_ascii =
        compat_upload_denied(server.compat_http().upload(compat_upload_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-10",
            "files.asset",
            DiagnosticRichnessProfile::Standard,
            non_ascii_metadata_upload(),
        )));
    assert_denial(
        &non_ascii,
        WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
        "ASCII-printable",
    );
}
