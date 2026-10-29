#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_compat_http_phase_three_runtime as compat_http_phase_three_runtime};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_query_handoff_runtime as query_handoff_runtime};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy};

#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_worth_native_assertions as worth_native_assertions};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_worth_native_runtime as worth_native_runtime};

pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_server::request_context::DiagnosticRichnessProfile, worth_server::surfaces::CompatHttpSurface, worth_server::surfaces::WorthNativeSurface, worth_server::WorthServer, worth_server::WorthServerCompatHttpRouteFamily, worth_server::WorthServerCompatibilityMutationExecutionInput, worth_server::WorthServerCompatibilityRequestInput, worth_server::WorthServerConfig, worth_server::WorthServerMiddlewareConfig, worth_server::WorthServerOperationAuthorityDeclaration, worth_server::WorthServerOperationAuthorizationPolicy, worth_server::WorthServerOperationFamily, worth_server::WorthServerOperationRegistration, worth_server::WorthServerQueryHandoffConfig, worth_server::WorthServerQueryHandoffDenial, worth_server::WorthServerQueryHandoffDenialCode, worth_server::WorthServerQueryOperation, worth_server::WorthServerRequestContextConfig, worth_server::WorthServerSuccessKind, worth_server::WorthServerSurfaceFamily};

pub(crate) use {worth_testkit_server::shared_support_compat_http_phase_three_runtime::compat_mutation_success, worth_testkit_server::shared_support_compat_http_phase_three_runtime::insert_task, worth_testkit_server::shared_support_compat_http_phase_three_runtime::single_insert_body};
pub(crate) use {worth_testkit_server::shared_support_query_handoff_runtime::runtime_real_mutation::RealMutationWorkspaceProvider};
pub(crate) use {worth_testkit_server::shared_support_worth_native_assertions::worth_native_session};

pub(crate) fn build_operation_name_guarded_server() -> WorthServer {
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
                        .with_query_mutation_enabled(true)
                        .build()
                        .expect("middleware config should validate"),
                )
                .with_query_handoff_config(
                    worth_server::WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(RealMutationWorkspaceProvider)
                        .build()
                        .expect("query handoff config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operations(operation_name_guarded_registrations())
        .register_surface(worth_server::surfaces::WorthNativeSurface::enabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::phase_one_enabled())
        .build()
        .expect("operation-name guarded server should build")
}

pub(crate) fn operation_name_guarded_registrations() -> Vec<WorthServerOperationRegistration> {
    worth_server::WorthServerOperationRegistration::phase_two_defaults()
        .into_iter()
        .filter(|registration| {
            registration.family() != worth_server::WorthServerOperationFamily::QueryDirectSubmission
        })
        .chain([worth_server::WorthServerOperationRegistration::enabled(
            worth_server::WorthServerOperationFamily::QueryDirectSubmission,
        )
        .exposed_on([
            worth_server::WorthServerSurfaceFamily::WorthNative,
            worth_server::WorthServerSurfaceFamily::CompatHttp,
        ])
        .admit_operation_names(["tasks.insert"])
        .with_authority_declaration(
            worth_server::WorthServerOperationAuthorityDeclaration::deterministic_submission(
                "query-write",
                "query-write-review",
                "derive-from-request",
                "derive-from-request",
            ),
        )
        .with_authorization_policy(worth_server::WorthServerOperationAuthorizationPolicy::allow_authenticated())])
        .collect()
}

pub(crate) fn prepared_mutation_request(
    server: &WorthServer,
    operation_name: &str,
) -> worth_server::WorthServerCompatibilityPreparedRequest {
    match server.compat_http().prepare_request(
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
            .build()
            .expect("compat mutation request should validate"),
    ) {
        worth_proof::TransitionOutcome::Success(prepared) => prepared,
        other => panic!("expected prepared compatibility mutation request, got {other:?}"),
    }
}

pub(crate) fn compat_mutation_denied(
    outcome: worth_server::WorthServerCompatibilityMutationOutcome<
        worth_server::WorthServerCompatibilityMutation,
    >,
) -> WorthServerQueryHandoffDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied compat mutation, got {other:?}"),
    }
}

pub(crate) fn direct_mutation_success(
    outcome: worth_server::WorthServerDirectMutationOutcome,
) -> worth_server::WorthServerDirectMutation {
    match outcome {
        worth_proof::TransitionOutcome::Success(mutation) => mutation,
        other => panic!("expected direct mutation success, got {other:?}"),
    }
}

pub(crate) fn direct_mutation_denied(
    outcome: worth_server::WorthServerDirectMutationOutcome,
) -> WorthServerQueryHandoffDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied direct mutation, got {other:?}"),
    }
}