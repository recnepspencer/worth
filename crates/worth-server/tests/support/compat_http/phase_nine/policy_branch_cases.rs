use worth_foundational::facade::DiagnosticRichnessProfile;
use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
use worth_server::WorthServerBinaryDownloadRequest;

use crate::shared_support_compat_http_phase_nine_assertions::{
    assert_diagnostics_profile, assert_policy_alignment,
};
use crate::shared_support_compat_http_phase_nine_runtime::{
    build_phase_nine_server_with_workspace_provider, compat_download_execution_input,
    compat_download_success, compat_read_execution_input, compat_read_success,
};
use crate::shared_support_query_handoff_runtime::ProfiledTestWorkspaceProvider;

#[test]
fn compat_http_branch_and_diagnostics_variation_keep_policy_and_metadata_truth_aligned() {
    let server =
        build_phase_nine_server_with_workspace_provider(ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));

    let standard = compat_read_success(server.compat_http().read(compat_read_execution_input(
        &server,
        "tenant-a",
        "workspace-42",
        "branch-9",
        "files.asset",
        DiagnosticRichnessProfile::Standard,
    )));
    let forensic = compat_download_success(server.compat_http().download(
        compat_download_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            "files.asset",
            DiagnosticRichnessProfile::Forensic,
            WorthServerBinaryDownloadRequest::new(b"hello-world".to_vec()),
        ),
    ));
    let branch_variant =
        compat_read_success(server.compat_http().read(compat_read_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-10",
            "files.asset",
            DiagnosticRichnessProfile::Standard,
        )));

    assert_eq!(
        standard
            .file_envelope()
            .metadata_receipt()
            .metadata_identity(),
        forensic
            .file_envelope()
            .metadata_receipt()
            .metadata_identity()
    );
    assert_ne!(
        standard
            .file_envelope()
            .policy_decision()
            .canonical_digest(),
        forensic
            .file_envelope()
            .policy_decision()
            .canonical_digest()
    );
    assert_ne!(
        standard
            .file_envelope()
            .metadata_receipt()
            .metadata_identity(),
        branch_variant
            .file_envelope()
            .metadata_receipt()
            .metadata_identity()
    );
    assert_eq!(
        standard
            .file_envelope()
            .transfer_provenance()
            .content_type(),
        None
    );

    assert_diagnostics_profile(
        standard.file_envelope(),
        DiagnosticRichnessProfile::Standard,
    );
    assert_diagnostics_profile(
        forensic.file_envelope(),
        DiagnosticRichnessProfile::Forensic,
    );
    assert_policy_alignment(standard.file_envelope());
    assert_policy_alignment(forensic.file_envelope());
    assert_policy_alignment(branch_variant.file_envelope());
    assert_eq!(
        forensic
            .file_envelope()
            .transfer_provenance()
            .content_type(),
        Some("application/octet-stream")
    );
}
