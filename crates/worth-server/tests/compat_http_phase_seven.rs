#[path = "support/compat_http/phase_seven/assertions.rs"]
mod compat_http_phase_seven_assertions;
#[path = "support/compat_http/phase_seven/runtime.rs"]
mod compat_http_phase_seven_runtime;
#[path = "support/query_handoff/runtime.rs"]
mod query_handoff_runtime;
#[path = "support/test_bootstrap_policy.rs"]
mod test_bootstrap_policy;

use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
use worth_server::{
    WorthServerBinaryDownloadAuthorization, WorthServerBinaryDownloadRequest,
    WorthServerCompatibilityRequestInput, WorthServerQueryHandoffDenialCode,
};

use compat_http_phase_seven_assertions::{
    assert_counter, assert_download_denial, assert_metadata_parity,
};
use compat_http_phase_seven_runtime::{
    build_phase_seven_server_with_workspace_provider, compat_download_denied,
    compat_download_execution_input, compat_download_success, download_input, prepared_request,
};
use query_handoff_runtime::ProfiledTestWorkspaceProvider;

#[test]
fn compat_http_range_assembled_downloads_preserve_canonical_metadata_truth() {
    let server =
        build_phase_seven_server_with_workspace_provider(ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));
    let body = b"phase-seven-download-body".to_vec();
    let full = compat_download_success(
        server
            .compat_http()
            .download(compat_download_execution_input(
                &server,
                "tenant-a",
                "workspace-42",
                "branch-9",
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(body.clone())
                    .with_content_type("application/pdf"),
            )),
    );
    let first_range = compat_download_success(
        server
            .compat_http()
            .download(worth_server::WorthServerBinaryDownloadExecutionInput::new(
                prepared_request(
                    &server,
                    download_input(
                        "tenant-a",
                        "workspace-42",
                        "branch-9",
                        "files.asset.download",
                    )
                    .with_header("range", "bytes=0-10")
                    .build()
                    .expect("range request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(body.clone())
                    .with_content_type("application/pdf"),
            )),
    );
    let second_range = compat_download_success(
        server
            .compat_http()
            .download(worth_server::WorthServerBinaryDownloadExecutionInput::new(
                prepared_request(
                    &server,
                    download_input(
                        "tenant-a",
                        "workspace-42",
                        "branch-9",
                        "files.asset.download",
                    )
                    .with_header("range", "bytes=11-")
                    .build()
                    .expect("range request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(body.clone())
                    .with_content_type("application/pdf"),
            )),
    );

    assert_eq!(
        [first_range.payload_bytes(), second_range.payload_bytes()].concat(),
        full.payload_bytes()
    );
    assert_metadata_parity(&full, &first_range);
    assert_metadata_parity(&full, &second_range);
    assert_counter(
        full.performance_receipt(),
        "compat_http.download.full_buffer_materializations",
        1,
    );
    assert_eq!(
        first_range.session().content_range().as_deref(),
        Some("bytes 0-10/25")
    );
    assert_eq!(
        second_range.session().content_range().as_deref(),
        Some("bytes 11-24/25")
    );
}

#[test]
fn compat_http_hostile_range_authorization_denies_out_of_bounds_and_unauthorized_spans() {
    let server =
        build_phase_seven_server_with_workspace_provider(ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));
    let body = b"phase-seven-download-body".to_vec();
    let unauthorized = compat_download_denied(
        server
            .compat_http()
            .download(worth_server::WorthServerBinaryDownloadExecutionInput::new(
                prepared_request(
                    &server,
                    download_input(
                        "tenant-a",
                        "workspace-42",
                        "branch-9",
                        "files.asset.download",
                    )
                    .with_header("range", "bytes=0-10")
                    .build()
                    .expect("range request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(body.clone()).with_authorization(
                    WorthServerBinaryDownloadAuthorization::admitted_window(4, 9),
                ),
            )),
    );
    let out_of_bounds = compat_download_denied(
        server
            .compat_http()
            .download(worth_server::WorthServerBinaryDownloadExecutionInput::new(
                prepared_request(
                    &server,
                    download_input(
                        "tenant-a",
                        "workspace-42",
                        "branch-9",
                        "files.asset.download",
                    )
                    .with_header("range", "bytes=0-99")
                    .build()
                    .expect("range request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(body),
            )),
    );

    assert_download_denial(
        &unauthorized,
        WorthServerQueryHandoffDenialCode::CompatibilityDownloadRequestInvalid,
        "authorization window",
    );
    assert_download_denial(
        &out_of_bounds,
        WorthServerQueryHandoffDenialCode::CompatibilityDownloadRequestInvalid,
        "available binary representation length",
    );
}

#[test]
fn compat_http_range_shape_hostility_rejects_multi_range_and_honors_canonical_if_range_fallback() {
    let server =
        build_phase_seven_server_with_workspace_provider(ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));
    let body = b"phase-seven-download-body".to_vec();
    let baseline = compat_download_success(server.compat_http().download(
        compat_download_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            "files.asset.download",
            WorthServerBinaryDownloadRequest::new(body.clone()),
        ),
    ));
    let multi_range = compat_download_denied(
        server
            .compat_http()
            .download(worth_server::WorthServerBinaryDownloadExecutionInput::new(
                prepared_request(
                    &server,
                    download_input(
                        "tenant-a",
                        "workspace-42",
                        "branch-9",
                        "files.asset.download",
                    )
                    .with_header("range", "bytes=0-2,1-3")
                    .build()
                    .expect("range request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(body.clone()),
            )),
    );
    let zero_suffix = compat_download_denied(
        server
            .compat_http()
            .download(worth_server::WorthServerBinaryDownloadExecutionInput::new(
                prepared_request(
                    &server,
                    download_input(
                        "tenant-a",
                        "workspace-42",
                        "branch-9",
                        "files.asset.download",
                    )
                    .with_header("range", "bytes=-0")
                    .build()
                    .expect("suffix request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(body.clone()),
            )),
    );
    let if_range_fallback = compat_download_success(
        server
            .compat_http()
            .download(worth_server::WorthServerBinaryDownloadExecutionInput::new(
                prepared_request(
                    &server,
                    download_input(
                        "tenant-a",
                        "workspace-42",
                        "branch-9",
                        "files.asset.download",
                    )
                    .with_header("range", "bytes=5-9")
                    .with_header("if-range", "\"stale-validator\"")
                    .build()
                    .expect("If-Range request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(body.clone()),
            )),
    );
    let if_range_exact = compat_download_success(
        server
            .compat_http()
            .download(worth_server::WorthServerBinaryDownloadExecutionInput::new(
                prepared_request(
                    &server,
                    download_input(
                        "tenant-a",
                        "workspace-42",
                        "branch-9",
                        "files.asset.download",
                    )
                    .with_header("range", "bytes=5-9")
                    .with_header("if-range", baseline.session().validator().entity_tag())
                    .build()
                    .expect("exact If-Range request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(body.clone()),
            )),
    );
    let suffix_range = compat_download_success(
        server
            .compat_http()
            .download(worth_server::WorthServerBinaryDownloadExecutionInput::new(
                prepared_request(
                    &server,
                    download_input(
                        "tenant-a",
                        "workspace-42",
                        "branch-9",
                        "files.asset.download",
                    )
                    .with_header("range", "bytes=-4")
                    .build()
                    .expect("suffix request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(body.clone()),
            )),
    );
    let head_range = compat_download_success(
        server
            .compat_http()
            .download(worth_server::WorthServerBinaryDownloadExecutionInput::new(
                prepared_request(
                    &server,
                    WorthServerCompatibilityRequestInput::builder()
                        .with_authenticated_principal_id("principal-7")
                        .with_tenant_id("tenant-a")
                        .with_workspace_id("workspace-42")
                        .with_branch_id("branch-9")
                        .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Download)
                        .with_method("HEAD")
                        .with_path("/compat/downloads/files.asset.download")
                        .with_header("accept", "application/octet-stream")
                        .with_header("range", "bytes=5-9")
                        .build()
                        .expect("HEAD download request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(body),
            )),
    );

    assert_download_denial(
        &multi_range,
        WorthServerQueryHandoffDenialCode::CompatibilityDownloadRequestInvalid,
        "multi-range requests",
    );
    assert_download_denial(
        &zero_suffix,
        WorthServerQueryHandoffDenialCode::CompatibilityDownloadRequestInvalid,
        "at least one byte",
    );
    assert_eq!(
        if_range_fallback.payload_bytes(),
        b"phase-seven-download-body"
    );
    assert!(!if_range_fallback.session().range_honored());
    assert_eq!(if_range_exact.payload_bytes(), b"-seve");
    assert!(if_range_exact.session().range_honored());
    assert_eq!(suffix_range.payload_bytes(), b"body");
    assert!(suffix_range.session().range_honored());
    assert_eq!(head_range.payload_bytes(), b"");
    assert!(head_range.session().range_honored());
    assert_counter(
        head_range.performance_receipt(),
        "compat_http.download.head_requests",
        1,
    );
}
