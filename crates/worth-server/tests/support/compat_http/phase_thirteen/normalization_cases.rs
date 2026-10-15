use {
    crate::shared_support_compat_http_phase_ten_runtime as compat_http_phase_ten_runtime,
    crate::shared_support_compat_http_phase_thirteen_assertions::{
        assert_bundle_digests_equal, assert_compatibility_denial_contains, assert_denial_contains,
    },
    crate::shared_support_compat_http_phase_thirteen_bundle::{
        WorthServerPhaseThirteenBundle, CACHEABILITY_DIGEST, POLICY_DIGEST, REQUEST_CONTRACT_DIGEST,
    },
    crate::shared_support_compat_http_phase_thirteen_runtime::{
        canonical_download_success, phase_thirteen_server,
    },
};

#[test]
fn compat_http_phase_thirteen_edge_normalization_matrix_preserves_one_request_and_cache_story() {
    let server = phase_thirteen_server();
    let trimmed = compat_http_phase_ten_runtime::prepared_request(
        &server,
        worth_server::WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-13")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_branch_id("branch-9")
            .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Read)
            .with_method("GET")
            .with_path("/compat/reads/files.asset")
            .with_header("accept", "application/json")
            .with_header("x-Worth-api-version", " 1 ")
            .with_query_pair("mode", " table ")
            .build()
            .expect("trimmed request should validate"),
    );
    let canonical = compat_http_phase_ten_runtime::prepared_request(
        &server,
        worth_server::WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-13")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_branch_id("branch-9")
            .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Read)
            .with_method("GET")
            .with_path("/compat/reads/files.asset")
            .with_header("accept", "application/json")
            .with_header("x-Worth-api-version", "1")
            .with_query_pair("mode", "table")
            .build()
            .expect("canonical request should validate"),
    );
    let repeated_query_left = compat_http_phase_ten_runtime::prepared_request(
        &server,
        worth_server::WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-13")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_branch_id("branch-9")
            .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Read)
            .with_method("GET")
            .with_path("/compat/reads/files.asset")
            .with_header("accept", "application/json")
            .with_query_pair("tag", "beta")
            .with_query_pair("tag", "alpha")
            .build()
            .expect("left repeated-query request should validate"),
    );
    let repeated_query_right = compat_http_phase_ten_runtime::prepared_request(
        &server,
        worth_server::WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-13")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_branch_id("branch-9")
            .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Read)
            .with_method("GET")
            .with_path("/compat/reads/files.asset")
            .with_header("accept", "application/json")
            .with_query_pair("tag", "alpha")
            .with_query_pair("tag", "beta")
            .build()
            .expect("right repeated-query request should validate"),
    );
    let preflight = compat_http_phase_ten_runtime::prepared_request(
        &server,
        worth_server::WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-13")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_branch_id("branch-9")
            .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Preflight)
            .with_method("OPTIONS")
            .with_path("/compat/reads/files.asset")
            .with_header("origin", "https://example.com")
            .build()
            .expect("preflight request should validate"),
    );
    let forwarded_host_denial = match server.compat_http().prepare_request(
        worth_server::WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-13")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_branch_id("branch-9")
            .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Read)
            .with_method("GET")
            .with_path("/compat/reads/files.asset")
            .with_header("accept", "application/json")
            .with_header("x-forwarded-host", "public.one.example")
            .with_header("x-forwarded-host", "public.two.example")
            .build()
            .expect("forwarded host hostility request should validate"),
    ) {
        worth_proof::TransitionOutcome::Denied(value) => value,
        other => panic!("expected forwarded host denial, got {other:?}"),
    };
    let representation_denial = match server.compat_http().prepare_request(
        worth_server::WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-13")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_branch_id("branch-9")
            .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Download)
            .with_method("GET")
            .with_path("/compat/downloads/files.asset.download")
            .with_header("accept", "application/json")
            .build()
            .expect("unsupported representation request should validate"),
    ) {
        worth_proof::TransitionOutcome::Denied(value) => value,
        other => panic!("expected unsupported representation denial, got {other:?}"),
    };
    let metadata_hostility =
        compat_http_phase_ten_runtime::compat_upload_denied(server.compat_http().upload(
            compat_http_phase_ten_runtime::compat_upload_execution_input(
                &server,
                "tenant-a",
                "workspace-42",
                "branch-9",
                "files.asset",
                worth_foundational::facade::DiagnosticRichnessProfile::Standard,
                compat_http_phase_ten_runtime::non_ascii_metadata_upload(),
            ),
        ));

    let trimmed_bundle = WorthServerPhaseThirteenBundle::new().with_digest(
        REQUEST_CONTRACT_DIGEST,
        trimmed.request_contract().canonical_digest(),
    );
    let canonical_bundle = WorthServerPhaseThirteenBundle::new().with_digest(
        REQUEST_CONTRACT_DIGEST,
        canonical.request_contract().canonical_digest(),
    );
    let repeated_query_left_bundle = WorthServerPhaseThirteenBundle::new().with_digest(
        REQUEST_CONTRACT_DIGEST,
        repeated_query_left.request_contract().canonical_digest(),
    );
    let repeated_query_right_bundle = WorthServerPhaseThirteenBundle::new().with_digest(
        REQUEST_CONTRACT_DIGEST,
        repeated_query_right.request_contract().canonical_digest(),
    );
    assert_bundle_digests_equal(
        &trimmed_bundle,
        &canonical_bundle,
        &[REQUEST_CONTRACT_DIGEST],
    );
    assert_bundle_digests_equal(
        &repeated_query_left_bundle,
        &repeated_query_right_bundle,
        &[REQUEST_CONTRACT_DIGEST],
    );
    assert_eq!(
        preflight.request_contract().route_family(),
        worth_server::WorthServerCompatHttpRouteFamily::Preflight
    );
    assert_compatibility_denial_contains(
        &forwarded_host_denial,
        worth_server::WorthServerCompatibilityDenialCode::AmbiguousForwardingHeaders,
        "x-forwarded-host",
    );
    assert_compatibility_denial_contains(
        &representation_denial,
        worth_server::WorthServerCompatibilityDenialCode::UnsupportedRepresentation,
        "application/json",
    );
    assert_denial_contains(
        &metadata_hostility,
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
        "ASCII-printable",
    );

    let download = canonical_download_success(&server, "files.asset");
    let download_bundle = WorthServerPhaseThirteenBundle::new()
        .with_digest(
            CACHEABILITY_DIGEST,
            download
                .file_envelope()
                .cacheability_policy()
                .canonical_digest(),
        )
        .with_digest(
            POLICY_DIGEST,
            download
                .file_envelope()
                .policy_decision()
                .canonical_digest(),
        );
    assert_eq!(
        download
            .file_envelope()
            .cacheability_policy()
            .cache_control(),
        "private, no-store"
    );
    assert!(download_bundle
        .digest(CACHEABILITY_DIGEST)
        .expect("download should preserve cacheability digest")
        .contains("private"),);
    assert!(download_bundle
        .digest(POLICY_DIGEST)
        .expect("download should preserve policy digest")
        .contains("branch"),);
}
