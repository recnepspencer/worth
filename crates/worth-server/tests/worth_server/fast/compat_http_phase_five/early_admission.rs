use std::sync::atomic::Ordering;

use serde_json::json;
use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
use worth_server::{
    WorthServerCompatHttpRouteFamily, WorthServerCompatibilityRequestInput,
    WorthServerMultipartUpload, WorthServerQueryHandoffDenialCode, WorthServerUploadExpectation,
    WorthServerUploadManifest, WorthServerUploadPart,
};

use super::compat_http_phase_five_runtime::{
    build_phase_five_server, build_phase_five_server_with_mutation_disabled,
    build_phase_five_server_with_workspace_provider, compat_upload_denied, prepared_request,
    prepared_upload_denied, upload_order_alpha,
};
use super::query_handoff_runtime::ProfiledCountingTestWorkspaceProvider;
use super::upload_input::upload_input_with_content_type;

#[test]
fn compat_http_upload_prepare_rejects_expect_continue_before_any_write_executes() {
    let attempted_writes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = build_phase_five_server_with_workspace_provider(
        ProfiledCountingTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
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
                WorthServerMultipartUpload::new(WorthServerUploadManifest::new(json!(null)))
                    .with_expectation(WorthServerUploadExpectation::continue_required())
                    .with_part(
                        WorthServerUploadPart::file("avatar")
                            .with_content_type("image/png")
                            .with_declared_length(128),
                    ),
            ),
        ),
    );

    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid
    );
    assert!(denial
        .detail()
        .contains("manifest metadata must be a JSON object"));
    assert_eq!(
        attempted_writes.load(Ordering::Relaxed),
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
                    WorthServerCompatibilityRequestInput::builder()
                        .with_authenticated_principal_id("principal-7")
                        .with_tenant_id("tenant-a")
                        .with_workspace_id("workspace-42")
                        .with_branch_id("branch-9")
                        .with_route_family(WorthServerCompatHttpRouteFamily::Mutation)
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
        WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid
    );
    assert!(denial.detail().contains("requires the upload route family"));
}
