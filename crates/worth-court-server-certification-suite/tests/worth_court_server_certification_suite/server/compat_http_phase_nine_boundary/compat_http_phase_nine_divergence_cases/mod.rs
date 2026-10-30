use {worth_testkit_server::shared_support_compat_http_phase_nine_assertions::assert_denial};
use {worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::server::build_phase_nine_server_with_workspace_provider, worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::fixtures::canonical_upload, worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::outcomes::compat_download_denied, worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::requests::compat_download_execution_input, worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::outcomes::compat_upload_denied, worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::requests::compat_upload_execution_input, worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::fixtures::malformed_upload};

#[test]
fn compat_http_denied_authorization_and_malformed_metadata_do_not_create_truth_transfer_fog() {
    let attempted_writes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = build_phase_nine_server_with_workspace_provider(
        worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledCountingTestWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
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
            worth_foundational::facade::DiagnosticRichnessProfile::Standard,
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
                worth_foundational::facade::DiagnosticRichnessProfile::Standard,
                worth_server::WorthServerBinaryDownloadRequest::new(b"hello-world".to_vec())
                    .with_authorization(
                        worth_server::WorthServerBinaryDownloadAuthorization::admitted_window(
                            20, 24,
                        ),
                    ),
            )),
    );

    assert_denial(
        &malformed,
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
        "collection",
    );
    assert_denial(
        &denied_download,
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityDownloadRequestInvalid,
        "outside the admitted authorization window",
    );
    assert_eq!(
        attempted_writes.load(std::sync::atomic::Ordering::Relaxed),
        0,
        "malformed metadata must deny before authoritative truth motion"
    );

    let successful_upload = server.compat_http().upload(compat_upload_execution_input(
        &server,
        "tenant-a",
        "workspace-42",
        "branch-9",
        "files.asset",
        worth_foundational::facade::DiagnosticRichnessProfile::Standard,
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
    let attempted_writes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = build_phase_nine_server_with_workspace_provider(
        worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledCountingTestWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
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
            worth_foundational::facade::DiagnosticRichnessProfile::Standard,
            canonical_upload("phase-nine-file"),
        )) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected prepared upload success, got {other:?}"),
    };
    let ingress_session = match server.compat_http().begin_binary_ingress(prepared_upload) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected binary ingress session success, got {other:?}"),
    };
    let cleanup = server
        .compat_http()
        .interrupt_binary_ingress(&ingress_session)
        .expect("interrupted session cleanup should succeed");
    let verification_denial = match server.compat_http().verify_binary_ingress(ingress_session) {
        worth_proof::TransitionOutcome::Denied(value) => value,
        other => panic!("expected verification denial after interruption, got {other:?}"),
    };

    assert_eq!(
        cleanup.reason(),
        worth_server::WorthServerUploadCleanupReason::Interrupted
    );
    assert_denial(
        &verification_denial,
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
        "no longer active",
    );
    assert_eq!(
        attempted_writes.load(std::sync::atomic::Ordering::Relaxed),
        0,
        "interrupted blob motion must not advance authoritative truth"
    );
}
