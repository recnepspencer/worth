#[allow(unused_imports)]
use crate::compat_http_phase_six::ordinary_context::integrity::*;

use {
    crate::compat_http_phase_six::ordinary_context::compat_http_phase_six_assertions::assert_ingress_counters,
    crate::compat_http_phase_six::ordinary_context::compat_http_phase_six_assertions::assert_upload_denial,
    crate::compat_http_phase_six::ordinary_context::compat_http_phase_six_assertions::stable_digest,
};
use {
    worth_testkit_server::shared_support_compat_http_phase_six_runtime::build_phase_six_server_with_workspace_provider,
    worth_testkit_server::shared_support_compat_http_phase_six_runtime::compat_upload_denied,
    worth_testkit_server::shared_support_compat_http_phase_six_runtime::compat_upload_execution_input,
    worth_testkit_server::shared_support_compat_http_phase_six_runtime::compat_upload_success,
};

#[test]
fn compat_http_upload_integrity_preserves_exact_digests_and_blocks_mismatch_before_commit() {
    let success_writes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let success_server =
        build_phase_six_server_with_workspace_provider(worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledCountingTestWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            success_writes.clone(),
        ));
    let success_upload = integrity_honest_upload();
    let expected_manifest_digest =
        stable_digest(success_upload.manifest().integrity_basis().as_bytes());
    let expected_avatar_digest = stable_digest(b"avatar-phase-six");
    let success = compat_upload_success(success_server.compat_http().upload(
        compat_upload_execution_input(
            &success_server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            "files.integrity.upload",
            "boundary-integrity",
            success_upload,
        ),
    ));

    assert_eq!(
        success.ingress_integrity().manifest_digest(),
        expected_manifest_digest
    );
    assert_eq!(
        success.ingress_integrity().part_digest("avatar"),
        Some(expected_avatar_digest.as_str())
    );
    assert_ingress_counters(success.ingress_performance(), 16, 16, 0, 0, 1);
    assert_eq!(success_writes.load(std::sync::atomic::Ordering::Relaxed), 1);

    let failed_writes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let failed_server =
        build_phase_six_server_with_workspace_provider(worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledCountingTestWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            failed_writes.clone(),
        ));
    let denial = compat_upload_denied(failed_server.compat_http().upload(
        compat_upload_execution_input(
            &failed_server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            "files.integrity.upload",
            "boundary-integrity-denied",
            integrity_mismatched_upload(),
        ),
    ));

    assert_upload_denial(
        &denial,
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
        "integrity digest mismatch",
    );
    assert_eq!(
        failed_writes.load(std::sync::atomic::Ordering::Relaxed),
        0,
        "integrity mismatch must deny before metadata truth commits"
    );
}
