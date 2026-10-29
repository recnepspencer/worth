use {std::sync::atomic::Ordering};

use {serde_json::json};
use {worth_query::facade::runtime::WorthQueryRuntimeSupportProfile};
use {worth_server::WorthServerCompatHttpRouteFamily, worth_server::WorthServerCompatibilityRequestInput, worth_server::WorthServerMultipartUpload, worth_server::WorthServerQueryHandoffDenialCode, worth_server::WorthServerUploadExpectation, worth_server::WorthServerUploadManifest, worth_server::WorthServerUploadPart};

use {crate::compat_http_phase_five::ordinary_context::compat_http_phase_five_runtime::build_phase_five_server, crate::compat_http_phase_five::ordinary_context::compat_http_phase_five_runtime::build_phase_five_server_with_mutation_disabled, crate::compat_http_phase_five::ordinary_context::compat_http_phase_five_runtime::build_phase_five_server_with_workspace_provider, crate::compat_http_phase_five::ordinary_context::compat_http_phase_five_runtime::compat_upload_denied, crate::compat_http_phase_five::ordinary_context::compat_http_phase_five_runtime::prepared_request, crate::compat_http_phase_five::ordinary_context::compat_http_phase_five_runtime::prepared_upload_denied, crate::compat_http_phase_five::ordinary_context::compat_http_phase_five_runtime::upload_order_alpha};
use {worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledCountingTestWorkspaceProvider};
use {crate::compat_http_phase_five::ordinary_context::upload_input::upload_input_with_content_type};

#[test]
fn compat_http_upload_prepare_rejects_expect_continue_before_any_write_executes() {
    let attempted_writes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = build_phase_five_server_with_workspace_provider(
        worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledCountingTestWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            attempted_writes.clone(),
        ),
    );
    let denial = prepared_upload_denied(
        server.compat_http().prepare_upload(
            worth_server::WorthServerCompatibilityUploadExecutionInput::new(
                prepared_request(
                    &server,
                    upload_input_with_content_type(
                        "files.avatar.upload",
                        "multipart/form-data; boundary=expect",
                    )
                    .with_header("expect", "100-continue")
                    .build()
                    .expect("expect-continue upload input should validate structurally"),
                ),
                "files.avatar.upload",
                worth_server::WorthServerMultipartUpload::new(worth_server::WorthServerUploadManifest::new(json!(null)))
                    .with_expectation(worth_server::WorthServerUploadExpectation::continue_required())
                    .with_part(
                        worth_server::WorthServerUploadPart::file("avatar")
                            .with_content_type("image/png")
                            .with_declared_length(128),
                    ),
            ),
        ),
    );

    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid
    );
    assert!(denial
        .detail()
        .contains("manifest metadata must be a JSON object"));
    assert_eq!(
        attempted_writes.load(std::sync::atomic::Ordering::Relaxed),
        0,
        "prepare_upload denials must occur before any authoritative write executes"
    );
}

#[test]
fn compat_http_upload_expect_continue_denies_unauthorized_requests_before_any_body_progression() {
    let server = build_phase_five_server_with_mutation_disabled();
    let denial = match server.compat_http().prepare_request(
        upload_input_with_content_type(
            "files.avatar.upload",
            "multipart/form-data; boundary=authz",
        )
        .with_header("expect", "100-continue")
        .build()
        .expect("unauthorized expect-continue upload input should validate structurally"),
    ) {
        worth_proof::TransitionOutcome::Denied(value) => value,
        other => panic!("expected upload request denial, got {other:?}"),
    };

    assert_eq!(
        denial.code(),
        worth_server::WorthServerCompatibilityDenialCode::MiddlewareDenied
    );
    assert!(denial
        .detail()
        .contains("query mutation intent is disabled"));
}

#[test]
fn compat_http_upload_requires_the_upload_route_family() {
    let server = build_phase_five_server();
    let denial = compat_upload_denied(
        server.compat_http().upload(
            worth_server::WorthServerCompatibilityUploadExecutionInput::new(
                prepared_request(
                    &server,
                    worth_server::WorthServerCompatibilityRequestInput::builder()
                        .with_authenticated_principal_id("principal-7")
                        .with_tenant_id("tenant-a")
                        .with_workspace_id("workspace-42")
                        .with_branch_id("branch-9")
                        .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Mutation)
                        .with_method("POST")
                        .with_path("/compat/mutations/files.avatar.upload")
                        .with_header("accept", "application/json")
                        .with_body_content_type("application/json")
                        .with_body_present(true)
                        .build()
                        .expect("mutation-shaped request should validate structurally"),
                ),
                "files.avatar.upload",
                upload_order_alpha("task-1"),
            ),
        ),
    );

    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid
    );
    assert!(denial.detail().contains("requires the upload route family"));
}
