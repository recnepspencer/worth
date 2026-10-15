use {crate::shared_support_compat_http_phase_thirteen_assertions::{
        assert_binary_counter, assert_bundle_digests_not_equal, assert_external_counter,
    }, crate::shared_support_compat_http_phase_thirteen_bundle::{
        WorthServerPhaseThirteenBundle, AUDIT_EVIDENCE_DIGEST, FAILURE_DIGEST, RESPONSE_DIGEST,
    }, crate::shared_support_compat_http_phase_thirteen_runtime::{
        canonical_download_success, canonical_upload_success, malformed_upload_denial,
        phase_thirteen_server,
    }};

#[test]
fn compat_http_phase_thirteen_operator_evidence_reconstructs_external_outcomes_without_logs() {
    let server = phase_thirteen_server();
    let upload = canonical_upload_success(&server, "files.asset", "phase-thirteen-evidence");
    let download = canonical_download_success(&server, "files.asset");
    let malformed_denial = malformed_upload_denial(&server);

    let upload_bundle = WorthServerPhaseThirteenBundle::new()
        .with_digest(
            AUDIT_EVIDENCE_DIGEST,
            upload
                .certification_bundle()
                .operator_evidence_record()
                .canonical_digest(),
        )
        .with_digest(
            RESPONSE_DIGEST,
            upload
                .certification_bundle()
                .operator_evidence_record()
                .operator_record()
                .response_digest(),
        );
    let download_bundle = WorthServerPhaseThirteenBundle::new()
        .with_digest(
            AUDIT_EVIDENCE_DIGEST,
            download
                .certification_bundle()
                .operator_evidence_record()
                .canonical_digest(),
        )
        .with_digest(
            RESPONSE_DIGEST,
            download
                .certification_bundle()
                .operator_evidence_record()
                .operator_record()
                .response_digest(),
        );
    let denial_bundle = WorthServerPhaseThirteenBundle::new().with_digest(
        FAILURE_DIGEST,
        format!(
            "{:?}:{}",
            malformed_denial.code(),
            malformed_denial.detail()
        ),
    );

    assert_eq!(
        upload
            .certification_bundle()
            .operator_evidence_record()
            .classification_label(),
        "compatibility_upload_succeeded"
    );
    assert_eq!(
        download
            .certification_bundle()
            .operator_evidence_record()
            .classification_label(),
        "compatibility_download_succeeded"
    );
    assert_external_counter(
        upload.certification_bundle().external_counters(),
        "compat_http.external.upload.successes",
        1,
    );
    assert_binary_counter(
        download.certification_bundle().binary_counters(),
        "compat_http.download.requests",
        1,
    );
    assert_ne!(
        upload.certification_bundle().canonical_digest(),
        download.certification_bundle().canonical_digest(),
        "canonical certification bundles must stay lane-specific enough to reconstruct the outcome class",
    );
    assert_bundle_digests_not_equal(
        &upload_bundle,
        &download_bundle,
        &[AUDIT_EVIDENCE_DIGEST, RESPONSE_DIGEST],
    );
    assert!(denial_bundle
        .digest(FAILURE_DIGEST)
        .expect("denial bundle should preserve failure digest")
        .contains("CompatibilityMutationRequestInvalid"),);
}
