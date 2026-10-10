#[path = "support/compat_http/phase_eight/assertions.rs"]
mod compat_http_phase_eight_assertions;
#[path = "support/compat_http/phase_eight/resume_boundary_cases.rs"]
mod compat_http_phase_eight_resume_boundary_cases;
#[path = "support/compat_http/phase_eight/runtime.rs"]
mod compat_http_phase_eight_runtime;
#[path = "support/query_handoff/runtime.rs"]
mod query_handoff_runtime;
#[path = "support/test_execution_policy.rs"]
mod test_execution_policy;

use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
use worth_server::{
    WorthServerBinaryDownloadRequest, WorthServerBinaryResumeRequest,
    WorthServerQueryHandoffDenialCode,
};

use compat_http_phase_eight_assertions::assert_download_denial;
use compat_http_phase_eight_runtime::{
    build_phase_eight_server_with_workspace_provider, compat_download_denied,
    compat_download_success, compat_resume_success, download_input, prepared_request,
};
use query_handoff_runtime::ProfiledTestWorkspaceProvider;

#[test]
fn compat_http_restart_stable_resume_claims_fail_typed_without_a_real_contract() {
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
    let denial = compat_download_denied(
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
                    .expect("restart-stable resume request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(body).with_resume_request(
                    WorthServerBinaryResumeRequest::resume_from(resume).require_restart_stable(),
                ),
            )),
    );

    assert_download_denial(
        &denial,
        WorthServerQueryHandoffDenialCode::CompatibilityDownloadRequestInvalid,
        "restart-stable resume is not admitted",
    );
}

#[test]
fn compat_http_resume_denies_byte_drift_and_validator_drift() {
    let server =
        build_phase_eight_server_with_workspace_provider(ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));
    let baseline_body = b"phase-eight-download-body".to_vec();
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
                WorthServerBinaryDownloadRequest::new(baseline_body.clone()),
            )),
    );
    let resume = compat_resume_success(server.compat_http().plan_binary_resume(first.session()));
    let digest_drift = compat_download_denied(
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
                    .expect("digest drift request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(b"phase-eight-download-bodz".to_vec())
                    .with_resume_request(WorthServerBinaryResumeRequest::resume_from(
                        resume.clone(),
                    )),
            )),
    );
    let validator_drift = compat_download_denied(
        server
            .compat_http()
            .download(worth_server::WorthServerBinaryDownloadExecutionInput::new(
                prepared_request(
                    &server,
                    download_input(
                        "tenant-a",
                        "workspace-99",
                        "branch-9",
                        "files.asset.download",
                    )
                    .with_header("range", "bytes=11-")
                    .build()
                    .expect("workspace drift request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(baseline_body)
                    .with_resume_request(WorthServerBinaryResumeRequest::resume_from(resume)),
            )),
    );

    assert_download_denial(
        &digest_drift,
        WorthServerQueryHandoffDenialCode::CompatibilityDownloadRequestInvalid,
        "resume integrity digest mismatch",
    );
    assert_download_denial(
        &validator_drift,
        WorthServerQueryHandoffDenialCode::CompatibilityDownloadRequestInvalid,
        "different workspace delivery context",
    );
}

#[test]
fn compat_http_resume_denies_explicit_integrity_expectation_mismatch() {
    let server =
        build_phase_eight_server_with_workspace_provider(ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));
    let baseline_body = b"phase-eight-download-body".to_vec();
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
                WorthServerBinaryDownloadRequest::new(baseline_body.clone()),
            )),
    );
    let mismatched = compat_download_success(
        server
            .compat_http()
            .download(worth_server::WorthServerBinaryDownloadExecutionInput::new(
                prepared_request(
                    &server,
                    download_input(
                        "tenant-a",
                        "workspace-99",
                        "branch-9",
                        "files.asset.download",
                    )
                    .build()
                    .expect("mismatched download should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(b"other-download-body".to_vec()),
            )),
    );
    let resume = compat_resume_success(server.compat_http().plan_binary_resume(first.session()));
    let denial = compat_download_denied(
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
                WorthServerBinaryDownloadRequest::new(baseline_body).with_resume_request(
                    WorthServerBinaryResumeRequest::resume_from(resume)
                        .with_expected_integrity(mismatched.integrity_digest().clone()),
                ),
            )),
    );

    assert_download_denial(
        &denial,
        WorthServerQueryHandoffDenialCode::CompatibilityDownloadRequestInvalid,
        "resume integrity digest mismatch",
    );
}

#[test]
fn compat_http_resume_denies_byte_position_drift() {
    let server =
        build_phase_eight_server_with_workspace_provider(ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));
    let baseline_body = b"phase-eight-download-body".to_vec();
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
                WorthServerBinaryDownloadRequest::new(baseline_body.clone()),
            )),
    );
    let resume = compat_resume_success(server.compat_http().plan_binary_resume(first.session()));
    let denial = compat_download_denied(
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
                    .with_header("range", "bytes=10-")
                    .build()
                    .expect("boundary drift request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(baseline_body)
                    .with_resume_request(WorthServerBinaryResumeRequest::resume_from(resume)),
            )),
    );

    assert_download_denial(
        &denial,
        WorthServerQueryHandoffDenialCode::CompatibilityDownloadRequestInvalid,
        "resume request expected byte 11 but selected byte 10",
    );
}
