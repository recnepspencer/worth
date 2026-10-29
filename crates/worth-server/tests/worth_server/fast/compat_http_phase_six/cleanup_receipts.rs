use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
use worth_server::{WorthServerQueryHandoffDenialCode, WorthServerUploadCleanupReason};

use super::compat_http_phase_six_assertions::{
    assert_cleanup_receipt, assert_counter, assert_upload_denial,
};
use super::compat_http_phase_six_runtime::{
    build_phase_six_server_with_workspace_provider, prepared_request, prepared_upload, upload_input,
};
use super::query_handoff_runtime::ProfiledCountingTestWorkspaceProvider;
use super::upload_fixtures::single_insert_body;

#[test]
fn compat_http_staged_upload_cleanup_receipts_prevent_authoritative_truth_drift() {
    let attempted_writes = Arc::new(AtomicUsize::new(0));
    let server =
        build_phase_six_server_with_workspace_provider(ProfiledCountingTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
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

    assert_cleanup_receipt(&interrupted, WorthServerUploadCleanupReason::Interrupted);
    assert_cleanup_receipt(&expired, WorthServerUploadCleanupReason::Expired);
    assert_cleanup_receipt(&abandoned, WorthServerUploadCleanupReason::Abandoned);
    assert_cleanup_receipt(&mismatch, WorthServerUploadCleanupReason::OwnershipMismatch);
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
        WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
        "no longer active",
    );
    assert_eq!(
        attempted_writes.load(Ordering::Relaxed),
        0,
        "cleanup paths must not advance authoritative metadata truth"
    );
}

fn begin_session(
    server: &worth_server::WorthServer,
    tenant_id: &str,
    workspace_id: &str,
    branch_id: &str,
    operation_name: &str,
    boundary: &str,
) -> worth_server::WorthServerBinaryIngressSession {
    let prepared = prepared_upload(
        server,
        tenant_id,
        workspace_id,
        branch_id,
        operation_name,
        boundary,
        cleanup_upload(operation_name),
    );
    match server.compat_http().begin_binary_ingress(prepared) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected binary ingress session, got {other:?}"),
    }
}

fn cleanup_upload(identity: &str) -> worth_server::WorthServerMultipartUpload {
    worth_server::WorthServerMultipartUpload::new(
        worth_server::WorthServerUploadManifest::new(single_insert_body(identity))
            .with_file_part("avatar"),
    )
    .with_expectation(worth_server::WorthServerUploadExpectation::continue_optional())
    .with_part(
        worth_server::WorthServerUploadPart::file("avatar")
            .with_content_type("image/png")
            .with_declared_length(8)
            .with_body_bytes(b"cleanup!".to_vec()),
    )
}
