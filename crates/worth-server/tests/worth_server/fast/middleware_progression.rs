use worth_proof::{TransitionOutcome, TransitionReadiness};
use worth_server::{
    request_context::DiagnosticRichnessProfile,
    surfaces::{CompatHttpSurface, WorthNativeSurface},
    WorthServer, WorthServerAdmission, WorthServerConfig, WorthServerDenial, WorthServerDenialCode,
    WorthServerDenialPriority, WorthServerMiddlewareConfig, WorthServerPipelineInput,
    WorthServerPipelineIntent, WorthServerPipelineStep, WorthServerPreparedQueryHandoffKind,
    WorthServerRequestContextConfig, WorthServerRequestContextInput,
    WorthServerResolvedRequestContext, WorthServerSurfaceFamily, WorthServerTransportClass,
};

fn test_server(
    request_context: WorthServerRequestContextConfig,
    middleware: WorthServerMiddlewareConfig,
) -> WorthServer {
    WorthServer::builder()
        .with_config(
            WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(request_context)
                .with_middleware_config(middleware)
                .build()
                .expect("server config should validate"),
        )
        .register_surface(WorthNativeSurface::disabled())
        .register_surface(CompatHttpSurface::disabled())
        .build()
        .expect("server should build")
}

fn request_input_builder() -> worth_server::WorthServerRequestContextInputBuilder {
    WorthServerRequestContextInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
}

fn resolve_request_context(
    server: &WorthServer,
    input: WorthServerRequestContextInput,
) -> WorthServerResolvedRequestContext {
    match server.request_contexts().resolve(input) {
        TransitionReadiness::Ready(resolved) => resolved,
        other => panic!("expected resolved request context, got {other:?}"),
    }
}

fn denied(
    outcome: TransitionOutcome<
        WorthServerAdmission,
        WorthServerDenial,
        worth_server::WorthServerMiddlewareDeferred,
        worth_server::WorthServerMiddlewareStale,
        worth_server::WorthServerMiddlewareRebindRequired,
        worth_server::WorthServerMiddlewareFailure,
    >,
) -> WorthServerDenial {
    match outcome {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected middleware denial, got {other:?}"),
    }
}

#[test]
fn admit_returns_ready_admission_with_prepared_query_handoff_intent() {
    let server = test_server(
        WorthServerRequestContextConfig::builder()
            .with_preview_targeting_enabled(true)
            .build()
            .expect("request context config should validate"),
        WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
    );
    let resolved_request_context = resolve_request_context(
        &server,
        request_input_builder()
            .with_surface_family(WorthServerSurfaceFamily::WorthNative)
            .with_transport_class(WorthServerTransportClass::WorthNativeInProcess)
            .with_branch_id("branch-9")
            .build()
            .expect("request context input should validate"),
    );

    let admission = server.middleware().admit(WorthServerPipelineInput::new(
        resolved_request_context.clone(),
        WorthServerPipelineIntent::query_read("users.profile"),
    ));

    let admission = match admission {
        TransitionOutcome::Success(admitted) => admitted,
        other => panic!("expected admitted middleware result, got {other:?}"),
    };

    assert_eq!(
        admission.resolved_request_context(),
        &resolved_request_context
    );
    assert_eq!(
        admission.query_handoff_intent().kind(),
        WorthServerPreparedQueryHandoffKind::QueryRead
    );
    assert_eq!(
        admission.query_handoff_intent().operation_name(),
        "users.profile"
    );
}

#[test]
fn admit_preserves_validation_denial_across_surface_families() {
    let server = test_server(
        WorthServerRequestContextConfig::builder()
            .build()
            .expect("request context config should validate"),
        WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(false)
            .build()
            .expect("middleware config should validate"),
    );

    let worth_native = denied(
        server.middleware().admit(WorthServerPipelineInput::new(
            resolve_request_context(
                &server,
                request_input_builder()
                    .with_surface_family(WorthServerSurfaceFamily::WorthNative)
                    .with_transport_class(WorthServerTransportClass::WorthNativeInProcess)
                    .build()
                    .expect("request context input should validate"),
            ),
            WorthServerPipelineIntent::query_mutation("users.rename"),
        )),
    );
    let compat_http = denied(
        server.middleware().admit(WorthServerPipelineInput::new(
            resolve_request_context(
                &server,
                request_input_builder()
                    .with_surface_family(WorthServerSurfaceFamily::CompatHttp)
                    .with_transport_class(WorthServerTransportClass::CompatHttp)
                    .build()
                    .expect("request context input should validate"),
            ),
            WorthServerPipelineIntent::query_mutation("users.rename"),
        )),
    );

    assert_eq!(worth_native, compat_http);
    assert_eq!(
        worth_native.code(),
        WorthServerDenialCode::QueryMutationDisabled
    );
    assert_eq!(
        worth_native.priority(),
        WorthServerDenialPriority::Validation
    );
    assert_eq!(
        worth_native.step(),
        WorthServerPipelineStep::ValidationPosture
    );
}

#[test]
fn admit_uses_canonical_denial_priority_under_overlapping_failures() {
    let server = test_server(
        WorthServerRequestContextConfig::builder()
            .with_preview_targeting_enabled(true)
            .with_default_diagnostics_profile(DiagnosticRichnessProfile::Forensic)
            .with_maximum_diagnostics_profile(DiagnosticRichnessProfile::Forensic)
            .build()
            .expect("request context config should validate"),
        WorthServerMiddlewareConfig::builder()
            .with_compat_http_maximum_diagnostics_profile(
                DiagnosticRichnessProfile::OperationalMinimal,
            )
            .with_preview_branch_authorization_enabled(false)
            .with_query_mutation_enabled(false)
            .build()
            .expect("middleware config should validate"),
    );

    let overlapping_inputs = [
        WorthServerPipelineInput::new(
            resolve_request_context(
                &server,
                request_input_builder()
                    .with_surface_family(WorthServerSurfaceFamily::CompatHttp)
                    .with_transport_class(WorthServerTransportClass::CompatHttp)
                    .with_preview_id("preview-1")
                    .with_diagnostics_profile(DiagnosticRichnessProfile::Forensic)
                    .build()
                    .expect("request context input should validate"),
            ),
            WorthServerPipelineIntent::query_mutation("users.rename"),
        ),
        WorthServerPipelineInput::new(
            resolve_request_context(
                &server,
                request_input_builder()
                    .with_surface_family(WorthServerSurfaceFamily::WorthNative)
                    .with_transport_class(WorthServerTransportClass::WorthNativeInProcess)
                    .with_preview_id("preview-2")
                    .with_diagnostics_profile(DiagnosticRichnessProfile::Forensic)
                    .build()
                    .expect("request context input should validate"),
            ),
            WorthServerPipelineIntent::query_mutation("users.rename"),
        ),
        WorthServerPipelineInput::new(
            resolve_request_context(
                &server,
                request_input_builder()
                    .with_surface_family(WorthServerSurfaceFamily::CompatHttp)
                    .with_transport_class(WorthServerTransportClass::CompatHttp)
                    .with_preview_id("preview-3")
                    .with_diagnostics_profile(DiagnosticRichnessProfile::Standard)
                    .build()
                    .expect("request context input should validate"),
            ),
            WorthServerPipelineIntent::query_mutation("users.rename"),
        ),
    ];

    for overlapping_input in overlapping_inputs {
        let denial = denied(server.middleware().admit(overlapping_input));
        assert_eq!(
            denial.code(),
            WorthServerDenialCode::PreviewBranchAccessDenied
        );
        assert_eq!(denial.priority(), WorthServerDenialPriority::Authorization);
        assert_eq!(denial.step(), WorthServerPipelineStep::AuthorizationPosture);
        assert_eq!(
            denial.detail(),
            "preview branch access is denied by middleware authorization posture"
        );
    }
}

#[test]
fn admit_denies_compat_http_diagnostics_budget_before_validation_when_authorization_allows() {
    let server = test_server(
        WorthServerRequestContextConfig::builder()
            .with_default_diagnostics_profile(DiagnosticRichnessProfile::Forensic)
            .with_maximum_diagnostics_profile(DiagnosticRichnessProfile::Forensic)
            .build()
            .expect("request context config should validate"),
        WorthServerMiddlewareConfig::builder()
            .with_compat_http_maximum_diagnostics_profile(
                DiagnosticRichnessProfile::OperationalMinimal,
            )
            .with_preview_branch_authorization_enabled(true)
            .with_query_mutation_enabled(false)
            .build()
            .expect("middleware config should validate"),
    );

    let denial = denied(
        server.middleware().admit(WorthServerPipelineInput::new(
            resolve_request_context(
                &server,
                request_input_builder()
                    .with_surface_family(WorthServerSurfaceFamily::CompatHttp)
                    .with_transport_class(WorthServerTransportClass::CompatHttp)
                    .with_diagnostics_profile(DiagnosticRichnessProfile::Forensic)
                    .build()
                    .expect("request context input should validate"),
            ),
            WorthServerPipelineIntent::query_mutation("users.rename"),
        )),
    );

    assert_eq!(
        denial.code(),
        WorthServerDenialCode::CompatHttpDiagnosticsBudgetExceeded
    );
    assert_eq!(denial.priority(), WorthServerDenialPriority::Budget);
    assert_eq!(denial.step(), WorthServerPipelineStep::BudgetPosture);
}
