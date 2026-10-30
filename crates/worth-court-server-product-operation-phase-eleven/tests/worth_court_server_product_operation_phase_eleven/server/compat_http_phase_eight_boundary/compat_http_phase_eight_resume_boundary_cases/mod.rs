use crate::shared_support_compat_http_phase_eight_assertions::assert_download_denial;
use {
    worth_testkit_server::shared_support_compat_http_phase_eight_runtime::build_phase_eight_server_with_workspace_provider,
    worth_testkit_server::shared_support_compat_http_phase_eight_runtime::compat_download_denied,
    worth_testkit_server::shared_support_compat_http_phase_eight_runtime::compat_download_success,
    worth_testkit_server::shared_support_compat_http_phase_eight_runtime::compat_resume_success,
    worth_testkit_server::shared_support_compat_http_phase_eight_runtime::download_input,
    worth_testkit_server::shared_support_compat_http_phase_eight_runtime::prepared_request,
};

#[test]
fn compat_http_resume_denies_content_type_and_authorization_story_drift() {
    let server =
        build_phase_eight_server_with_workspace_provider(worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
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
                worth_server::WorthServerBinaryDownloadRequest::new(baseline_body.clone())
                    .with_content_type("application/pdf"),
            )),
    );
    let resume = compat_resume_success(server.compat_http().plan_binary_resume(first.session()));
    let content_type_drift = compat_download_denied(
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
                    .expect("content-type drift request should validate"),
                ),
                "files.asset.download",
                worth_server::WorthServerBinaryDownloadRequest::new(baseline_body.clone())
                    .with_content_type("application/octet-stream")
                    .with_resume_request(
                        worth_server::WorthServerBinaryResumeRequest::resume_from(resume.clone()),
                    ),
            )),
    );
    let authorization_drift = compat_download_denied(
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
                    .expect("authorization drift request should validate"),
                ),
                "files.asset.download",
                worth_server::WorthServerBinaryDownloadRequest::new(baseline_body)
                    .with_content_type("application/pdf")
                    .with_authorization(
                        worth_server::WorthServerBinaryDownloadAuthorization::admitted_window(
                            11, 25,
                        ),
                    )
                    .with_resume_request(
                        worth_server::WorthServerBinaryResumeRequest::resume_from(resume),
                    ),
            )),
    );

    assert_download_denial(
        &content_type_drift,
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityDownloadRequestInvalid,
        "canonical binary download artifact or policy story",
    );
    assert_download_denial(
        &authorization_drift,
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityDownloadRequestInvalid,
        "widened the admitted authorization window",
    );
}

#[test]
fn compat_http_head_requests_do_not_admit_resume_claims() {
    let server =
        build_phase_eight_server_with_workspace_provider(worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
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
                worth_server::WorthServerBinaryDownloadRequest::new(baseline_body.clone()),
            )),
    );
    let resume = compat_resume_success(server.compat_http().plan_binary_resume(first.session()));
    let denial = compat_download_denied(
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
                        .with_header("range", "bytes=11-")
                        .build()
                        .expect("HEAD resume request should validate structurally"),
                ),
                "files.asset.download",
                worth_server::WorthServerBinaryDownloadRequest::new(baseline_body)
                    .with_resume_request(
                        worth_server::WorthServerBinaryResumeRequest::resume_from(resume),
                    ),
            )),
    );

    assert_download_denial(
        &denial,
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityDownloadRequestInvalid,
        "HEAD binary egress does not admit resumed byte delivery claims",
    );
}
