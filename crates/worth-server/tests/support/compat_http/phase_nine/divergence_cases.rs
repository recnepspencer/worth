use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use worth_foundational::facade::DiagnosticRichnessProfile;
use worth_proof::TransitionOutcome;
use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
use worth_server::{
    WorthServerBinaryDownloadAuthorization, WorthServerQueryHandoffDenialCode,
    WorthServerUploadCleanupReason,
};

use crate::shared_support_compat_http_phase_nine_assertions::assert_denial;
use crate::shared_support_compat_http_phase_nine_runtime::{
    build_phase_nine_server_with_workspace_provider, canonical_upload, compat_download_denied,
    compat_download_execution_input, compat_upload_denied, compat_upload_execution_input,
    malformed_upload,
};
use crate::shared_support_query_handoff_runtime::ProfiledCountingTestWorkspaceProvider;

#[test]
fn compat_http_denied_authorization_and_malformed_metadata_do_not_create_truth_transfer_fog() {
    let attempted_writes = Arc::new(AtomicUsize::new(0));
    let server = build_phase_nine_server_with_workspace_provider(
        ProfiledCountingTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            attempted_writes.clone(),
        ),
    );

    let malformed =
        compat_upload_denied(server.compat_http().upload(compat_upload_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            "files.asset",
            DiagnosticRichnessProfile::Standard,
            malformed_upload(),
        )));
    let denied_download = compat_download_denied(
        server
            .compat_http()
            .download(compat_download_execution_input(
                &server,
                "tenant-a",
                "workspace-42",
                "branch-9",
                "files.asset",
                DiagnosticRichnessProfile::Standard,
                worth_server::WorthServerBinaryDownloadRequest::new(b"hello-world".to_vec())
                    .with_authorization(WorthServerBinaryDownloadAuthorization::admitted_window(
                        20, 24,
                    )),
            )),
    );

    assert_denial(
        &malformed,
        WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
        "collection",
    );
    assert_denial(
        &denied_download,
        WorthServerQueryHandoffDenialCode::CompatibilityDownloadRequestInvalid,
        "outside the admitted authorization window",
    );
    assert_eq!(
        attempted_writes.load(Ordering::Relaxed),
        0,
        "malformed metadata must deny before authoritative truth motion"
    );

    let successful_upload = server.compat_http().upload(compat_upload_execution_input(
        &server,
        "tenant-a",
        "workspace-42",
        "branch-9",
        "files.asset",
        DiagnosticRichnessProfile::Standard,
        canonical_upload("phase-nine-file"),
    ));
    assert!(
        matches!(
            successful_upload,
            worth_proof::TransitionOutcome::Success(_)
        ),
        "a healthy upload should still produce one canonical success story after hostile denials"
    );
}

#[test]
fn compat_http_interrupted_blob_motion_cannot_progress_into_truth_linkage_fog() {
    let attempted_writes = Arc::new(AtomicUsize::new(0));
    let server = build_phase_nine_server_with_workspace_provider(
        ProfiledCountingTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            attempted_writes.clone(),
        ),
    );

    let prepared_upload = match server
        .compat_http()
        .prepare_upload(compat_upload_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            "files.asset",
            DiagnosticRichnessProfile::Standard,
            canonical_upload("phase-nine-file"),
        )) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected prepared upload success, got {other:?}"),
    };
    let ingress_session = match server.compat_http().begin_binary_ingress(prepared_upload) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected binary ingress session success, got {other:?}"),
    };
    let cleanup = server
        .compat_http()
        .interrupt_binary_ingress(&ingress_session)
        .expect("interrupted session cleanup should succeed");
    let verification_denial = match server.compat_http().verify_binary_ingress(ingress_session) {
        TransitionOutcome::Denied(value) => value,
        other => panic!("expected verification denial after interruption, got {other:?}"),
    };

    assert_eq!(
        cleanup.reason(),
        WorthServerUploadCleanupReason::Interrupted
    );
    assert_denial(
        &verification_denial,
        WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
        "no longer active",
    );
    assert_eq!(
        attempted_writes.load(Ordering::Relaxed),
        0,
        "interrupted blob motion must not advance authoritative truth"
    );
}
