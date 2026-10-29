use std::sync::atomic::Ordering;

use {
    crate::shared_support_compat_http_phase_thirteen_assertions::assert_denial_contains,
    crate::shared_support_compat_http_phase_thirteen_runtime::{
        canonical_download_success, compat_read_with_diagnostics, idempotent_mutation,
        idempotent_mutation_conflict, phase_thirteen_counting_server, phase_thirteen_server,
        resumed_download_success,
    },
};

#[test]
fn compat_http_phase_thirteen_retry_and_precondition_matrix_keeps_authority_and_resume_honest() {
    let (server, attempted_writes) = phase_thirteen_counting_server();
    let first = idempotent_mutation(&server, "phase-thirteen-idem", "idem-phase-13");
    let retry = idempotent_mutation(&server, "phase-thirteen-idem", "idem-phase-13");
    let conflict =
        idempotent_mutation_conflict(&server, "phase-thirteen-conflict", "idem-phase-13");

    assert!(!first.envelope().retry_receipt().is_previously_completed());
    assert!(retry.envelope().retry_receipt().is_previously_completed());
    assert_eq!(
        first.mutation_result().result_digest(),
        retry.mutation_result().result_digest(),
    );
    assert_denial_contains(
        &conflict,
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityIdempotencyConflict,
        "cannot be reused",
    );

    let read_server = phase_thirteen_server();
    let baseline = compat_read_with_diagnostics(
        &read_server,
        "users.profile",
        worth_foundational::facade::DiagnosticRichnessProfile::Standard,
    );
    let precondition_denial = match read_server.compat_http().read(
        worth_server::WorthServerCompatibilityExecutionInput::new(
            crate::shared_support_compat_http_phase_two_runtime::prepared_read_request(
                &read_server,
                crate::shared_support_compat_http_phase_two_runtime::read_input("users.profile")
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
        attempted_writes.load(Ordering::Relaxed),
        1,
        "idempotent retry must not create duplicate authority effects",
    );
}
