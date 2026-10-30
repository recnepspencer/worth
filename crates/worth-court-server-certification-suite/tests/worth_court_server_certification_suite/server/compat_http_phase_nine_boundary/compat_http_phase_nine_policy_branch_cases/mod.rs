use worth_foundational::facade::DiagnosticRichnessProfile;
use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
use worth_server::WorthServerBinaryDownloadRequest;

use {worth_testkit_server::shared_support_compat_http_phase_nine_assertions::assert_diagnostics_profile, worth_testkit_server::shared_support_compat_http_phase_nine_assertions::assert_policy_alignment};
use {worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::server::build_phase_nine_server_with_workspace_provider, worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::requests::compat_download_execution_input, worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::outcomes::compat_download_success, worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::requests::compat_read_execution_input, worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::outcomes::compat_read_success};
use {worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider};

#[test]
fn compat_http_branch_and_diagnostics_variation_keep_policy_and_metadata_truth_aligned() {
    let server =
        build_phase_nine_server_with_workspace_provider(worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));

    let standard = compat_read_success(server.compat_http().read(compat_read_execution_input(
        &server,
        "tenant-a",
        "workspace-42",
        "branch-9",
        "files.asset",
        worth_foundational::facade::DiagnosticRichnessProfile::Standard,
    )));
    let forensic = compat_download_success(server.compat_http().download(
        compat_download_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            "files.asset",
            worth_foundational::facade::DiagnosticRichnessProfile::Forensic,
            worth_server::WorthServerBinaryDownloadRequest::new(b"hello-world".to_vec()),
        ),
    ));
    let branch_variant =
        compat_read_success(server.compat_http().read(compat_read_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-10",
            "files.asset",
            worth_foundational::facade::DiagnosticRichnessProfile::Standard,
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
        worth_foundational::facade::DiagnosticRichnessProfile::Standard,
    );
    assert_diagnostics_profile(
        forensic.file_envelope(),
        worth_foundational::facade::DiagnosticRichnessProfile::Forensic,
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
