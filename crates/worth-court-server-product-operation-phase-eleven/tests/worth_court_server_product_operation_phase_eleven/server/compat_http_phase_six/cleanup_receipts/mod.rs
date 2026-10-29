#[allow(unused_imports)] use crate::compat_http_phase_six::ordinary_context::cleanup_receipts::*;
use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc};

use {worth_query::facade::runtime::WorthQueryRuntimeSupportProfile};
use {worth_server::WorthServerQueryHandoffDenialCode, worth_server::WorthServerUploadCleanupReason};

use {crate::compat_http_phase_six::ordinary_context::compat_http_phase_six_assertions::assert_cleanup_receipt, crate::compat_http_phase_six::ordinary_context::compat_http_phase_six_assertions::assert_counter, crate::compat_http_phase_six::ordinary_context::compat_http_phase_six_assertions::assert_upload_denial};
use {worth_testkit_server::shared_support_compat_http_phase_six_runtime::build_phase_six_server_with_workspace_provider, worth_testkit_server::shared_support_compat_http_phase_six_runtime::prepared_request, worth_testkit_server::shared_support_compat_http_phase_six_runtime::prepared_upload, worth_testkit_server::shared_support_compat_http_phase_six_runtime::upload_input};
use {worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledCountingTestWorkspaceProvider};
use {crate::compat_http_phase_six::ordinary_context::upload_fixtures::single_insert_body};

#[test]
fn compat_http_staged_upload_cleanup_receipts_prevent_authoritative_truth_drift() {
    let attempted_writes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server =
        build_phase_six_server_with_workspace_provider(worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledCountingTestWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            attempted_writes.clone(),
        ));

    let interrupted_session = begin_session(
        &server,
        "tenant-a",
        "workspace-42",
        "branch-9",
        "cleanup.interrupted",
        "boundary-interrupted",
    );
    let interrupted = server
        .compat_http()
        .interrupt_binary_ingress(&interrupted_session)
        .expect("interrupted session cleanup should succeed");
    let expired = server
        .compat_http()
        .expire_binary_ingress(&begin_session(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            "cleanup.expired",
            "boundary-expired",
        ))
        .expect("expired session cleanup should succeed");
    let abandoned = server
        .compat_http()
        .abandon_binary_ingress(&begin_session(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            "cleanup.abandoned",
            "boundary-abandoned",
        ))
        .expect("abandoned session cleanup should succeed");
    let mismatched_session = begin_session(
        &server,
        "tenant-a",
        "workspace-42",
        "branch-9",
        "cleanup.mismatch",
        "boundary-mismatch",
    );
    let mismatched_request = prepared_request(
        &server,
        upload_input(
            "tenant-z",
            "workspace-42",
            "branch-9",
            "cleanup.mismatch",
            "boundary-mismatch-foreign",
        ),
    );
    let mismatch = server
        .compat_http()
        .cleanup_mismatched_binary_ingress(&mismatched_request, &mismatched_session)
        .expect("mismatched session cleanup should succeed");
    let interrupted_after_cleanup_denial = match server
        .compat_http()
        .verify_binary_ingress(interrupted_session)
    {
        worth_proof::TransitionOutcome::Denied(value) => value,
        other => panic!("expected inactive-session denial, got {other:?}"),
    };

    assert_cleanup_receipt(&interrupted, worth_server::WorthServerUploadCleanupReason::Interrupted);
    assert_cleanup_receipt(&expired, worth_server::WorthServerUploadCleanupReason::Expired);
    assert_cleanup_receipt(&abandoned, worth_server::WorthServerUploadCleanupReason::Abandoned);
    assert_cleanup_receipt(&mismatch, worth_server::WorthServerUploadCleanupReason::OwnershipMismatch);
    assert_counter(
        interrupted.performance_receipt(),
        "compat_http.upload.cleanup_operations",
        1,
    );
    assert_counter(
        interrupted.performance_receipt(),
        "compat_http.upload.cleanup_staged_bytes",
        8,
    );
    assert_upload_denial(
        &interrupted_after_cleanup_denial,
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
        "no longer active",
    );
    assert_eq!(
        attempted_writes.load(std::sync::atomic::Ordering::Relaxed),
        0,
        "cleanup paths must not advance authoritative metadata truth"
    );
}
