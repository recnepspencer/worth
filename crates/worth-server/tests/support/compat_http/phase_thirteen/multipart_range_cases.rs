use worth_foundational::facade::DiagnosticRichnessProfile;
use worth_server::{
    WorthServerBinaryDownloadAuthorization, WorthServerBinaryDownloadRequest,
    WorthServerQueryHandoffDenialCode,
};

use {crate::shared_support_compat_http_phase_nine_runtime as compat_http_phase_nine_runtime, crate::shared_support_compat_http_phase_ten_runtime as compat_http_phase_ten_runtime, crate::shared_support_compat_http_phase_thirteen_assertions::{
        assert_binary_counter, assert_bundle_digests_equal, assert_denial_contains,
        assert_external_counters_zero,
    }, crate::shared_support_compat_http_phase_thirteen_bundle::{
        WorthServerPhaseThirteenBundle, FILE_ENVELOPE_DIGEST, METADATA_IDENTITY_DIGEST,
        POLICY_DIGEST,
    }, crate::shared_support_compat_http_phase_thirteen_runtime::{phase_thirteen_server, upload_with_payload}};

#[test]
fn compat_http_phase_thirteen_multipart_and_range_miserable_path_matrix_stays_localized() {
    let alpha_server = phase_thirteen_server();
    let beta_server = phase_thirteen_server();
    let malformed_server = phase_thirteen_server();
    let range_server = phase_thirteen_server();

    let alpha =
        compat_http_phase_ten_runtime::compat_upload_success(alpha_server.compat_http().upload(
            compat_http_phase_ten_runtime::compat_upload_execution_input(
                &alpha_server,
                "tenant-a",
                "workspace-42",
                "branch-9",
                "files.asset",
                DiagnosticRichnessProfile::Standard,
                compat_http_phase_ten_runtime::canonical_upload("phase-thirteen-alpha"),
            ),
        ));
    let beta =
        compat_http_phase_ten_runtime::compat_upload_success(beta_server.compat_http().upload(
            compat_http_phase_ten_runtime::compat_upload_execution_input(
                &beta_server,
                "tenant-a",
                "workspace-42",
                "branch-9",
                "files.asset",
                DiagnosticRichnessProfile::Standard,
                compat_http_phase_ten_runtime::reordered_canonical_upload("phase-thirteen-alpha"),
            ),
        ));
    let malformed = compat_http_phase_nine_runtime::compat_upload_denied(upload_with_payload(
        &malformed_server,
        "files.asset",
        compat_http_phase_nine_runtime::malformed_upload(),
    ));
    let unauthorized_range = compat_http_phase_ten_runtime::compat_download_denied(
        range_server.compat_http().download(
            worth_server::WorthServerBinaryDownloadExecutionInput::new(
                compat_http_phase_ten_runtime::prepared_request(
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
                WorthServerBinaryDownloadRequest::new(b"hello-world".to_vec()).with_authorization(
                    WorthServerBinaryDownloadAuthorization::admitted_window(0, 5),
                ),
            ),
        ),
    );

    let alpha_bundle = WorthServerPhaseThirteenBundle::new()
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
    let beta_bundle = WorthServerPhaseThirteenBundle::new()
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
        WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
        "collection",
    );
    assert_denial_contains(
        &unauthorized_range,
        WorthServerQueryHandoffDenialCode::CompatibilityDownloadRequestInvalid,
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
