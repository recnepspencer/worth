use worth_foundational::facade::DiagnosticRichnessProfile;
use {
    worth_server::WorthServerBinaryDownloadAuthorization,
    worth_server::WorthServerBinaryDownloadRequest,
    worth_server::WorthServerQueryHandoffDenialCode,
};

use {
    crate::shared_support_compat_http_phase_thirteen_assertions::assert_binary_counter,
    crate::shared_support_compat_http_phase_thirteen_assertions::assert_bundle_digests_equal,
    crate::shared_support_compat_http_phase_thirteen_assertions::assert_denial_contains,
    crate::shared_support_compat_http_phase_thirteen_assertions::assert_external_counters_zero,
    crate::shared_support_compat_http_phase_thirteen_bundle::WorthServerPhaseThirteenBundle,
    crate::shared_support_compat_http_phase_thirteen_bundle::FILE_ENVELOPE_DIGEST,
    crate::shared_support_compat_http_phase_thirteen_bundle::METADATA_IDENTITY_DIGEST,
    crate::shared_support_compat_http_phase_thirteen_bundle::POLICY_DIGEST,
    crate::shared_support_compat_http_phase_thirteen_runtime::phase_thirteen_server,
    crate::shared_support_compat_http_phase_thirteen_runtime::upload_with_payload,
    worth_testkit_server::shared_support_compat_http_phase_nine_runtime as compat_http_phase_nine_runtime,
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime as compat_http_phase_ten_runtime,
};

#[test]
fn compat_http_multipart_and_range_miserable_path_matrix_stays_localized() {
    let alpha_server = phase_thirteen_server();
    let beta_server = phase_thirteen_server();
    let malformed_server = phase_thirteen_server();
    let range_server = phase_thirteen_server();

    let alpha =
        worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::outcomes::compat_upload_success(alpha_server.compat_http().upload(
            worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::requests::compat_upload_execution_input(
                &alpha_server,
                "tenant-a",
                "workspace-42",
                "branch-9",
                "files.asset",
                worth_foundational::facade::DiagnosticRichnessProfile::Standard,
                worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::fixtures::canonical_upload("phase-thirteen-alpha"),
            ),
        ));
    let beta =
        worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::outcomes::compat_upload_success(beta_server.compat_http().upload(
            worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::requests::compat_upload_execution_input(
                &beta_server,
                "tenant-a",
                "workspace-42",
                "branch-9",
                "files.asset",
                worth_foundational::facade::DiagnosticRichnessProfile::Standard,
                worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::fixtures::reordered_canonical_upload("phase-thirteen-alpha"),
            ),
        ));
    let malformed = worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::outcomes::compat_upload_denied(upload_with_payload(
        &malformed_server,
        "files.asset",
        worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::fixtures::malformed_upload(),
    ));
    let unauthorized_range = worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::outcomes::compat_download_denied(
        range_server.compat_http().download(
            worth_server::WorthServerBinaryDownloadExecutionInput::new(
                worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::requests::prepared_request(
                    &range_server,
                    worth_server::WorthServerCompatibilityRequestInput::builder()
                        .with_authenticated_principal_id("principal-13")
                        .with_tenant_id("tenant-a")
                        .with_workspace_id("workspace-42")
                        .with_branch_id("branch-9")
                        .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Download)
                        .with_method("GET")
                        .with_path("/compat/downloads/files.asset")
                        .with_header("accept", "application/octet-stream")
                        .with_header("range", "bytes=6-10")
                        .build()
                        .expect("range hostility request should validate"),
                ),
                "files.asset",
                worth_server::WorthServerBinaryDownloadRequest::new(b"hello-world".to_vec()).with_authorization(
                    worth_server::WorthServerBinaryDownloadAuthorization::admitted_window(0, 5),
                ),
            ),
        ),
    );

    let alpha_bundle = crate::shared_support_compat_http_phase_thirteen_bundle::WorthServerPhaseThirteenBundle::new()
        .with_digest(
            METADATA_IDENTITY_DIGEST,
            alpha.file_envelope().metadata_receipt().metadata_identity(),
        )
        .with_digest(
            FILE_ENVELOPE_DIGEST,
            alpha.file_envelope().canonical_digest(),
        )
        .with_digest(
            POLICY_DIGEST,
            alpha.file_envelope().policy_decision().canonical_digest(),
        );
    let beta_bundle = crate::shared_support_compat_http_phase_thirteen_bundle::WorthServerPhaseThirteenBundle::new()
        .with_digest(
            METADATA_IDENTITY_DIGEST,
            beta.file_envelope().metadata_receipt().metadata_identity(),
        )
        .with_digest(
            FILE_ENVELOPE_DIGEST,
            beta.file_envelope().canonical_digest(),
        )
        .with_digest(
            POLICY_DIGEST,
            beta.file_envelope().policy_decision().canonical_digest(),
        );

    assert_bundle_digests_equal(
        &alpha_bundle,
        &beta_bundle,
        &[
            METADATA_IDENTITY_DIGEST,
            FILE_ENVELOPE_DIGEST,
            POLICY_DIGEST,
        ],
    );
    assert_denial_contains(
        &malformed,
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
        "collection",
    );
    assert_denial_contains(
        &unauthorized_range,
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityDownloadRequestInvalid,
        "outside the admitted authorization window",
    );

    assert_binary_counter(
        alpha.certification_bundle().binary_counters(),
        "compat_http.upload.cleanup_operations",
        0,
    );
    assert_external_counters_zero(
        alpha.certification_bundle().external_counters(),
        &[
            "compat_http.external.download.successes",
            "compat_http.external.buffered_export.successes",
        ],
    );
}
