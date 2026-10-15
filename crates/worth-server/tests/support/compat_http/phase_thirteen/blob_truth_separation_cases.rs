use {crate::shared_support_compat_http_phase_thirteen_assertions::{
        assert_binary_counter, assert_bundle_digests_not_equal, assert_external_counter,
    }, crate::shared_support_compat_http_phase_thirteen_bundle::{
        WorthServerPhaseThirteenBundle, FILE_ENVELOPE_DIGEST, METADATA_IDENTITY_DIGEST,
        MUTATION_RESULT_DIGEST, POLICY_DIGEST, RESPONSE_DIGEST,
    }, crate::shared_support_compat_http_phase_thirteen_runtime::{
        canonical_download_success, canonical_upload_success, direct_and_compat_mutation,
        phase_thirteen_server,
    }};

#[test]
fn compat_http_phase_thirteen_blob_truth_separation_stays_explicit_under_shared_product_flow() {
    let server = phase_thirteen_server();
    let upload = canonical_upload_success(&server, "files.asset", "phase-thirteen-file");
    let download = canonical_download_success(&server, "files.asset");
    let (_direct_mutation, compat_mutation) =
        direct_and_compat_mutation(&server, "phase-thirteen-structured");

    let upload_bundle = WorthServerPhaseThirteenBundle::new()
        .with_digest(
            METADATA_IDENTITY_DIGEST,
            upload
                .file_envelope()
                .metadata_receipt()
                .metadata_identity(),
        )
        .with_digest(
            FILE_ENVELOPE_DIGEST,
            upload.file_envelope().canonical_digest(),
        )
        .with_digest(
            POLICY_DIGEST,
            upload.file_envelope().policy_decision().canonical_digest(),
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
            METADATA_IDENTITY_DIGEST,
            download
                .file_envelope()
                .metadata_receipt()
                .metadata_identity(),
        )
        .with_digest(
            FILE_ENVELOPE_DIGEST,
            download.file_envelope().canonical_digest(),
        )
        .with_digest(
            POLICY_DIGEST,
            download
                .file_envelope()
                .policy_decision()
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
    let mutation_bundle = WorthServerPhaseThirteenBundle::new()
        .with_digest(
            MUTATION_RESULT_DIGEST,
            compat_mutation.mutation_result().result_digest(),
        )
        .with_digest(
            RESPONSE_DIGEST,
            compat_mutation
                .envelope()
                .response_envelope()
                .canonical_digest(),
        );

    assert_eq!(
        upload_bundle.digest(METADATA_IDENTITY_DIGEST),
        download_bundle.digest(METADATA_IDENTITY_DIGEST),
        "binary ingress and egress should share one canonical metadata identity",
    );
    assert_bundle_digests_not_equal(
        &upload_bundle,
        &mutation_bundle,
        &[FILE_ENVELOPE_DIGEST, RESPONSE_DIGEST],
    );
    assert_bundle_digests_not_equal(
        &download_bundle,
        &mutation_bundle,
        &[FILE_ENVELOPE_DIGEST, RESPONSE_DIGEST],
    );

    assert!(upload.file_envelope().metadata_receipt().truth_committed());
    assert!(download.file_envelope().metadata_receipt().truth_observed());
    assert!(upload
        .file_envelope()
        .transfer_provenance()
        .byte_motion_observed());
    assert!(download
        .file_envelope()
        .transfer_provenance()
        .byte_motion_observed());
    assert_ne!(
        upload.file_envelope().policy_decision().support_posture_digest(),
        download.file_envelope().policy_decision().support_posture_digest(),
        "binary ingress and egress must keep their support lanes explicit even when metadata identity stays linked",
    );

    assert_external_counter(
        upload.certification_bundle().external_counters(),
        "compat_http.external.upload.successes",
        1,
    );
    assert_binary_counter(
        download.certification_bundle().binary_counters(),
        "compat_http.download.forbidden_fallbacks",
        0,
    );
}
