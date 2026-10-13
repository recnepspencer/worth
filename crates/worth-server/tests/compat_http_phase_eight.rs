#[path = "support/compat_http/phase_eight/assertions.rs"]
mod compat_http_phase_eight_assertions;
#[path = "support/compat_http/phase_eight/runtime.rs"]
mod compat_http_phase_eight_runtime;
#[path = "support/query_handoff/runtime.rs"]
mod query_handoff_runtime;
#[path = "support/test_bootstrap_policy.rs"]
mod test_bootstrap_policy;

use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
use worth_server::{WorthServerBinaryDownloadRequest, WorthServerBinaryResumeRequest};

use compat_http_phase_eight_assertions::{
    assert_counter, assert_integrity_matches, assert_metadata_parity,
};
use compat_http_phase_eight_runtime::{
    build_phase_eight_server_with_workspace_provider, compat_download_execution_input,
    compat_download_success, compat_resume_success, download_input, plan_integrity,
    prepared_request,
};
use query_handoff_runtime::ProfiledTestWorkspaceProvider;

#[test]
fn compat_http_resumed_downloads_stay_distinct_from_restart_stable_claims() {
    let server =
        build_phase_eight_server_with_workspace_provider(ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));
    let body = b"phase-eight-download-body".to_vec();
    let first = compat_download_success(
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
                    .expect("initial range request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(body.clone()),
            )),
    );
    let resume = compat_resume_success(server.compat_http().plan_binary_resume(first.session()));
    let resumed = compat_download_success(
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
                    .expect("resumed range request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(body).with_resume_request(
                    WorthServerBinaryResumeRequest::resume_from(resume)
                        .with_expected_integrity(first.integrity_digest().clone()),
                ),
            )),
    );

    assert!(!first.session().retry_posture().is_resume());
    assert!(resumed.session().retry_posture().is_resume());
    assert!(!resumed.session().retry_posture().restart_stable());
    assert_eq!(
        resumed.session().retry_posture().expected_next_start(),
        Some(11)
    );
    assert_integrity_matches(first.integrity_digest(), resumed.integrity_digest());
    assert_metadata_parity(&first, &resumed);
    assert_counter(
        resumed.performance_receipt(),
        "compat_http.download.resume_requests",
        1,
    );
    assert_counter(
        resumed.performance_receipt(),
        "compat_http.download.resumed_requests_admitted",
        1,
    );
    assert_counter(
        resumed.performance_receipt(),
        "compat_http.download.integrity_verifications",
        1,
    );
}

#[test]
fn compat_http_integrity_projection_stays_explicit_for_full_range_and_head_delivery() {
    let server =
        build_phase_eight_server_with_workspace_provider(ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));
    let body = b"phase-eight-download-body".to_vec();
    let full = compat_download_success(server.compat_http().download(
        compat_download_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            "files.asset.download",
            WorthServerBinaryDownloadRequest::new(body.clone()),
        ),
    ));
    let range = compat_download_success(
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
                    .with_header("range", "bytes=6-12")
                    .build()
                    .expect("range request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(body.clone()),
            )),
    );
    let head = compat_download_success(
        server
            .compat_http()
            .download(worth_server::WorthServerBinaryDownloadExecutionInput::new(
                prepared_request(
                    &server,
                    worth_server::WorthServerCompatibilityRequestInput::builder()
                        .with_authenticated_principal_id("principal-8")
                        .with_tenant_id("tenant-a")
                        .with_workspace_id("workspace-42")
                        .with_branch_id("branch-9")
                        .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Download)
                        .with_method("HEAD")
                        .with_path("/compat/downloads/files.asset.download")
                        .with_header("accept", "application/octet-stream")
                        .with_header("range", "bytes=0-4")
                        .build()
                        .expect("HEAD request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(body),
            )),
    );

    assert_integrity_matches(full.integrity_digest(), range.integrity_digest());
    assert_eq!(range.payload_bytes(), b"eight-d");
    assert_ne!(
        full.integrity_digest().selected_representation_digest(),
        range.integrity_digest().selected_representation_digest()
    );
    assert_eq!(head.payload_bytes(), b"");
    assert!(head.integrity_digest().head_only());
    assert_eq!(head.integrity_digest().selected_start(), 0);
    assert_eq!(head.integrity_digest().selected_end_exclusive(), 5);
    assert_eq!(
        plan_integrity(&server, head.session()).canonical_digest(),
        head.integrity_digest().canonical_digest()
    );
}
