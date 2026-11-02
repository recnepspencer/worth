use {
    crate::shared_support_compat_http_phase_thirteen_assertions::assert_denial_contains,
    crate::shared_support_compat_http_phase_thirteen_runtime::canonical_download_success,
    crate::shared_support_compat_http_phase_thirteen_runtime::compat_read_with_diagnostics,
    crate::shared_support_compat_http_phase_thirteen_runtime::phase_thirteen_counting_server,
    crate::shared_support_compat_http_phase_thirteen_runtime::phase_thirteen_server,
    crate::shared_support_compat_http_phase_thirteen_runtime::resumed_download_success,
    crate::shared_support_compat_http_phase_thirteen_runtime::unconfigured_keyed_mutation_refusal,
};

#[test]
fn compat_http_retry_and_precondition_matrix_keeps_authority_and_resume_honest() {
    let (server, attempted_writes) = phase_thirteen_counting_server();
    let first =
        unconfigured_keyed_mutation_refusal(&server, "phase-thirteen-idem", "idem-phase-13");
    let retry =
        unconfigured_keyed_mutation_refusal(&server, "phase-thirteen-idem", "idem-phase-13");
    let conflict =
        unconfigured_keyed_mutation_refusal(&server, "phase-thirteen-conflict", "idem-phase-13");

    // D9 retires this process-memory backend's receipt authority. The real
    // filesystem Server courts separately require original receipt equality.
    for refusal in [&first, &retry, &conflict] {
        assert_denial_contains(
            refusal,
            worth_server::WorthServerQueryHandoffDenialCode::WorkspaceBindingFailed,
            "no Query application mutation binding is installed",
        );
    }

    let read_server = phase_thirteen_server();
    let baseline = compat_read_with_diagnostics(
        &read_server,
        "users.profile",
        worth_foundational::facade::DiagnosticRichnessProfile::Standard,
    );
    let precondition_denial = match read_server.compat_http().read(
        worth_server::WorthServerCompatibilityExecutionInput::new(
            worth_testkit_server::shared_support_compat_http_phase_two_runtime::prepared_read_request(
                &read_server,
                worth_testkit_server::shared_support_compat_http_phase_two_runtime::read_input("users.profile")
                    .with_header("if-match", "\"validator:wrong\"")
                    .build()
                    .expect("conditional request should validate"),
            ),
            "users.profile",
        ),
    ) {
        worth_proof::TransitionOutcome::Denied(value) => value,
        other => panic!("expected conditional read denial, got {other:?}"),
    };
    assert_eq!(
        baseline.validator().entity_tag(),
        format!("\"{}\"", baseline.validator().canonical_digest())
    );
    assert_denial_contains(
        &precondition_denial,
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityConditionalReadPreconditionFailed,
        "does not match the canonical read validator",
    );

    let resume_server = phase_thirteen_server();
    let resumed = resumed_download_success(&resume_server);
    let ordinary = canonical_download_success(&resume_server, "files.asset.download");
    let retry_evidence = resumed
        .transfer_cleanup_evidence()
        .expect("resumed downloads should emit retry lifecycle evidence");
    assert_eq!(
        retry_evidence.reason(),
        worth_server::WorthServerTransferCleanupReason::DownloadRetryAdmitted
    );
    assert!(
        ordinary.transfer_cleanup_evidence().is_none(),
        "ordinary downloads must not fabricate retry lifecycle evidence",
    );

    assert_eq!(
        attempted_writes.load(std::sync::atomic::Ordering::Relaxed),
        0,
        "a missing Query owner cannot fall back to raw mutation effects",
    );
}
