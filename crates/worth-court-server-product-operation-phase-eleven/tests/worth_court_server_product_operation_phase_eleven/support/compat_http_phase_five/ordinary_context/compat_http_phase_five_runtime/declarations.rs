pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {serde_json::json, serde_json::Value};
pub(crate) use {
    worth_server::request_context::DiagnosticRichnessProfile,
    worth_server::surfaces::CompatHttpSurface, worth_server::surfaces::WorthNativeSurface,
    worth_server::WorthServer, worth_server::WorthServerCompatHttpRouteFamily,
    worth_server::WorthServerCompatibilityMutationExecutionInput,
    worth_server::WorthServerCompatibilityPreparedRequest,
    worth_server::WorthServerCompatibilityRequestInput,
    worth_server::WorthServerCompatibilityUploadExecutionInput, worth_server::WorthServerConfig,
    worth_server::WorthServerMiddlewareConfig, worth_server::WorthServerMultipartUpload,
    worth_server::WorthServerQueryHandoffConfig, worth_server::WorthServerQueryWorkspaceProvider,
    worth_server::WorthServerRequestContextConfig, worth_server::WorthServerUploadExpectation,
    worth_server::WorthServerUploadManifest, worth_server::WorthServerUploadPart,
};

pub(crate) fn build_phase_five_server() -> WorthServer {
    build_phase_five_server_with_workspace_provider(
        worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::TestWorkspaceProvider,
    )
}

pub(crate) fn build_phase_five_server_with_mutation_disabled() -> WorthServer {
    worth_server::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(
                    worth_server::WorthServerRequestContextConfig::builder()
                        .with_preview_targeting_enabled(true)
                        .with_default_diagnostics_profile(worth_server::request_context::DiagnosticRichnessProfile::Standard)
                        .build()
                        .expect("request context config should validate"),
                )
                .with_middleware_config(
                    worth_server::WorthServerMiddlewareConfig::builder()
                        .with_query_mutation_enabled(false)
                        .build()
                        .expect("middleware config should validate"),
                )
                .with_query_handoff_config(
                    worth_server::WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(
                            worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::TestWorkspaceProvider,
                        )
                        .build()
                        .expect("query handoff config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(worth_server::surfaces::WorthNativeSurface::enabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::phase_one_enabled())
        .build()
        .expect("server should build")
}

pub(crate) fn build_phase_five_server_with_workspace_provider(
    workspace_provider: impl WorthServerQueryWorkspaceProvider,
) -> WorthServer {
    worth_server::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(
                    worth_server::WorthServerRequestContextConfig::builder()
                        .with_preview_targeting_enabled(true)
                        .with_default_diagnostics_profile(
                            worth_server::request_context::DiagnosticRichnessProfile::Standard,
                        )
                        .build()
                        .expect("request context config should validate"),
                )
                .with_middleware_config(
                    worth_server::WorthServerMiddlewareConfig::builder()
                        .with_query_mutation_enabled(true)
                        .build()
                        .expect("middleware config should validate"),
                )
                .with_query_handoff_config(
                    worth_server::WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(workspace_provider)
                        .build()
                        .expect("query handoff config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(worth_server::surfaces::WorthNativeSurface::enabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::phase_one_enabled())
        .build()
        .expect("server should build")
}

pub(crate) fn upload_input(
    operation_name: &str,
    boundary: &str,
) -> worth_server::WorthServerCompatibilityRequestInputBuilder {
    worth_server::WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Upload)
        .with_method("POST")
        .with_path(format!("/compat/uploads/{operation_name}"))
        .with_header("accept", "application/json")
        .with_body_content_type(format!("multipart/form-data; boundary={boundary}"))
        .with_body_present(true)
}

pub(crate) fn mutation_input(
    operation_name: &str,
) -> worth_server::WorthServerCompatibilityRequestInputBuilder {
    worth_server::WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_branch_id("branch-9")
        .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Mutation)
        .with_method("POST")
        .with_path(format!("/compat/mutations/{operation_name}"))
        .with_header("accept", "application/json")
        .with_body_content_type("application/json")
        .with_body_present(true)
}

pub(crate) fn prepared_request(
    server: &WorthServer,
    request: WorthServerCompatibilityRequestInput,
) -> WorthServerCompatibilityPreparedRequest {
    match server.compat_http().prepare_request(request) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected prepared compatibility request, got {other:?}"),
    }
}

pub(crate) fn compat_upload_execution_input(
    server: &WorthServer,
    operation_name: &str,
    boundary: &str,
    upload: WorthServerMultipartUpload,
) -> WorthServerCompatibilityUploadExecutionInput {
    worth_server::WorthServerCompatibilityUploadExecutionInput::new(
        prepared_request(
            server,
            upload_input(operation_name, boundary)
                .build()
                .expect("compat upload input should validate structurally"),
        ),
        operation_name,
        upload,
    )
}

pub(crate) fn compat_mutation_execution_input(
    server: &WorthServer,
    operation_name: &str,
    body: Value,
) -> WorthServerCompatibilityMutationExecutionInput {
    worth_server::WorthServerCompatibilityMutationExecutionInput::new(
        prepared_request(
            server,
            mutation_input(operation_name)
                .build()
                .expect("compat mutation input should validate structurally"),
        ),
        operation_name,
        body,
    )
}

pub(crate) fn single_insert_body(identity: &str) -> Value {
    json!({
        "command": {
            "family": "insert",
            "collection": "Task",
            "aspects": {
                "identity.id": identity,
                "title.value": format!("Title for {identity}")
            }
        }
    })
}

pub(crate) fn manifest_for(identity: &str) -> WorthServerUploadManifest {
    worth_server::WorthServerUploadManifest::new(single_insert_body(identity))
        .with_file_part("avatar")
        .with_file_part("thumbnail")
}

pub(crate) fn upload_order_alpha(identity: &str) -> WorthServerMultipartUpload {
    worth_server::WorthServerMultipartUpload::new(manifest_for(identity))
        .with_expectation(worth_server::WorthServerUploadExpectation::continue_optional())
        .with_part(
            worth_server::WorthServerUploadPart::file("avatar")
                .with_content_type("image/png")
                .with_declared_length(128),
        )
        .with_part(
            worth_server::WorthServerUploadPart::file("thumbnail")
                .with_content_type("image/webp")
                .with_declared_length(64),
        )
}

pub(crate) fn upload_order_beta(identity: &str) -> WorthServerMultipartUpload {
    worth_server::WorthServerMultipartUpload::new(manifest_for(identity))
        .with_expectation(worth_server::WorthServerUploadExpectation::continue_optional())
        .with_part(
            worth_server::WorthServerUploadPart::file("thumbnail")
                .with_content_type("image/webp")
                .with_declared_length(64),
        )
        .with_part(
            worth_server::WorthServerUploadPart::file("avatar")
                .with_content_type("image/png")
                .with_declared_length(128),
        )
}

pub(crate) fn compat_upload_success(
    outcome: worth_server::WorthServerCompatibilityUploadOutcome<
        worth_server::WorthServerCompatibilityUpload,
    >,
) -> worth_server::WorthServerCompatibilityUpload {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility upload success, got {other:?}"),
    }
}

pub(crate) fn compat_upload_denied(
    outcome: worth_server::WorthServerCompatibilityUploadOutcome<
        worth_server::WorthServerCompatibilityUpload,
    >,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(value) => value,
        other => panic!("expected compatibility upload denial, got {other:?}"),
    }
}

pub(crate) fn compat_mutation_success(
    outcome: worth_server::WorthServerCompatibilityMutationOutcome<
        worth_server::WorthServerCompatibilityMutation,
    >,
) -> worth_server::WorthServerCompatibilityMutation {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility mutation success, got {other:?}"),
    }
}

pub(crate) fn prepared_upload_denied(
    outcome: worth_server::WorthServerCompatibilityUploadOutcome<
        worth_server::WorthServerPreparedMultipartUpload,
    >,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(value) => value,
        other => panic!("expected prepared compatibility upload denial, got {other:?}"),
    }
}
