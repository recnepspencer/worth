#[allow(unused_imports)]
use crate::middleware_progression::ordinary_context::*;
use {worth_proof::TransitionOutcome, worth_proof::TransitionReadiness};
use {
    worth_server::request_context::DiagnosticRichnessProfile,
    worth_server::surfaces::CompatHttpSurface, worth_server::surfaces::WorthNativeSurface,
    worth_server::WorthServer, worth_server::WorthServerAdmission, worth_server::WorthServerConfig,
    worth_server::WorthServerDenial, worth_server::WorthServerDenialCode,
    worth_server::WorthServerDenialPriority, worth_server::WorthServerMiddlewareConfig,
    worth_server::WorthServerPipelineInput, worth_server::WorthServerPipelineIntent,
    worth_server::WorthServerPipelineStep, worth_server::WorthServerPreparedQueryHandoffKind,
    worth_server::WorthServerRequestContextConfig, worth_server::WorthServerRequestContextInput,
    worth_server::WorthServerResolvedRequestContext, worth_server::WorthServerSurfaceFamily,
    worth_server::WorthServerTransportClass,
};

#[test]
fn admit_returns_ready_admission_with_prepared_query_handoff_intent() {
    let server = test_server(
        worth_server::WorthServerRequestContextConfig::builder()
            .with_preview_targeting_enabled(true)
            .build()
            .expect("request context config should validate"),
        worth_server::WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
    );
    let resolved_request_context = resolve_request_context(
        &server,
        request_input_builder()
            .with_surface_family(worth_server::WorthServerSurfaceFamily::WorthNative)
            .with_transport_class(worth_server::WorthServerTransportClass::WorthNativeInProcess)
            .with_branch_id("branch-9")
            .build()
            .expect("request context input should validate"),
    );

    let admission = server
        .middleware()
        .admit(worth_server::WorthServerPipelineInput::new(
            resolved_request_context.clone(),
            worth_server::WorthServerPipelineIntent::query_read("users.profile"),
        ));

    let admission = match admission {
        worth_proof::TransitionOutcome::Success(admitted) => admitted,
        other => panic!("expected admitted middleware result, got {other:?}"),
    };

    assert_eq!(
        admission.resolved_request_context(),
        &resolved_request_context
    );
    assert_eq!(
        admission.query_handoff_intent().kind(),
        worth_server::WorthServerPreparedQueryHandoffKind::QueryRead
    );
    assert_eq!(
        admission.query_handoff_intent().operation_name(),
        "users.profile"
    );
}

#[test]
fn admit_preserves_validation_denial_across_surface_families() {
    let server = test_server(
        worth_server::WorthServerRequestContextConfig::builder()
            .build()
            .expect("request context config should validate"),
        worth_server::WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(false)
            .build()
            .expect("middleware config should validate"),
    );

    let worth_native = denied(
        server
            .middleware()
            .admit(worth_server::WorthServerPipelineInput::new(
                resolve_request_context(
                    &server,
                    request_input_builder()
                        .with_surface_family(worth_server::WorthServerSurfaceFamily::WorthNative)
                        .with_transport_class(
                            worth_server::WorthServerTransportClass::WorthNativeInProcess,
                        )
                        .build()
                        .expect("request context input should validate"),
                ),
                worth_server::WorthServerPipelineIntent::query_mutation("users.rename"),
            )),
    );
    let compat_http = denied(
        server
            .middleware()
            .admit(worth_server::WorthServerPipelineInput::new(
                resolve_request_context(
                    &server,
                    request_input_builder()
                        .with_surface_family(worth_server::WorthServerSurfaceFamily::CompatHttp)
                        .with_transport_class(worth_server::WorthServerTransportClass::CompatHttp)
                        .build()
                        .expect("request context input should validate"),
                ),
                worth_server::WorthServerPipelineIntent::query_mutation("users.rename"),
            )),
    );

    assert_eq!(worth_native, compat_http);
    assert_eq!(
        worth_native.code(),
        worth_server::WorthServerDenialCode::QueryMutationDisabled
    );
    assert_eq!(
        worth_native.priority(),
        worth_server::WorthServerDenialPriority::Validation
    );
    assert_eq!(
        worth_native.step(),
        worth_server::WorthServerPipelineStep::ValidationPosture
    );
}

#[test]
fn admit_uses_canonical_denial_priority_under_overlapping_failures() {
    let server = test_server(
        worth_server::WorthServerRequestContextConfig::builder()
            .with_preview_targeting_enabled(true)
            .with_default_diagnostics_profile(
                worth_server::request_context::DiagnosticRichnessProfile::Forensic,
            )
            .with_maximum_diagnostics_profile(
                worth_server::request_context::DiagnosticRichnessProfile::Forensic,
            )
            .build()
            .expect("request context config should validate"),
        worth_server::WorthServerMiddlewareConfig::builder()
            .with_compat_http_maximum_diagnostics_profile(
                worth_server::request_context::DiagnosticRichnessProfile::OperationalMinimal,
            )
            .with_preview_branch_authorization_enabled(false)
            .with_query_mutation_enabled(false)
            .build()
            .expect("middleware config should validate"),
    );

    let overlapping_inputs = [
        worth_server::WorthServerPipelineInput::new(
            resolve_request_context(
                &server,
                request_input_builder()
                    .with_surface_family(worth_server::WorthServerSurfaceFamily::CompatHttp)
                    .with_transport_class(worth_server::WorthServerTransportClass::CompatHttp)
                    .with_preview_id("preview-1")
                    .with_diagnostics_profile(
                        worth_server::request_context::DiagnosticRichnessProfile::Forensic,
                    )
                    .build()
                    .expect("request context input should validate"),
            ),
            worth_server::WorthServerPipelineIntent::query_mutation("users.rename"),
        ),
        worth_server::WorthServerPipelineInput::new(
            resolve_request_context(
                &server,
                request_input_builder()
                    .with_surface_family(worth_server::WorthServerSurfaceFamily::WorthNative)
                    .with_transport_class(
                        worth_server::WorthServerTransportClass::WorthNativeInProcess,
                    )
                    .with_preview_id("preview-2")
                    .with_diagnostics_profile(
                        worth_server::request_context::DiagnosticRichnessProfile::Forensic,
                    )
                    .build()
                    .expect("request context input should validate"),
            ),
            worth_server::WorthServerPipelineIntent::query_mutation("users.rename"),
        ),
        worth_server::WorthServerPipelineInput::new(
            resolve_request_context(
                &server,
                request_input_builder()
                    .with_surface_family(worth_server::WorthServerSurfaceFamily::CompatHttp)
                    .with_transport_class(worth_server::WorthServerTransportClass::CompatHttp)
                    .with_preview_id("preview-3")
                    .with_diagnostics_profile(
                        worth_server::request_context::DiagnosticRichnessProfile::Standard,
                    )
                    .build()
                    .expect("request context input should validate"),
            ),
            worth_server::WorthServerPipelineIntent::query_mutation("users.rename"),
        ),
    ];

    for overlapping_input in overlapping_inputs {
        let denial = denied(server.middleware().admit(overlapping_input));
        assert_eq!(
            denial.code(),
            worth_server::WorthServerDenialCode::PreviewBranchAccessDenied
        );
        assert_eq!(
            denial.priority(),
            worth_server::WorthServerDenialPriority::Authorization
        );
        assert_eq!(
            denial.step(),
            worth_server::WorthServerPipelineStep::AuthorizationPosture
        );
        assert_eq!(
            denial.detail(),
            "preview branch access is denied by middleware authorization posture"
        );
    }
}

#[test]
fn admit_denies_compat_http_diagnostics_budget_before_validation_when_authorization_allows() {
    let server = test_server(
        worth_server::WorthServerRequestContextConfig::builder()
            .with_default_diagnostics_profile(
                worth_server::request_context::DiagnosticRichnessProfile::Forensic,
            )
            .with_maximum_diagnostics_profile(
                worth_server::request_context::DiagnosticRichnessProfile::Forensic,
            )
            .build()
            .expect("request context config should validate"),
        worth_server::WorthServerMiddlewareConfig::builder()
            .with_compat_http_maximum_diagnostics_profile(
                worth_server::request_context::DiagnosticRichnessProfile::OperationalMinimal,
            )
            .with_preview_branch_authorization_enabled(true)
            .with_query_mutation_enabled(false)
            .build()
            .expect("middleware config should validate"),
    );

    let denial = denied(
        server
            .middleware()
            .admit(worth_server::WorthServerPipelineInput::new(
                resolve_request_context(
                    &server,
                    request_input_builder()
                        .with_surface_family(worth_server::WorthServerSurfaceFamily::CompatHttp)
                        .with_transport_class(worth_server::WorthServerTransportClass::CompatHttp)
                        .with_diagnostics_profile(
                            worth_server::request_context::DiagnosticRichnessProfile::Forensic,
                        )
                        .build()
                        .expect("request context input should validate"),
                ),
                worth_server::WorthServerPipelineIntent::query_mutation("users.rename"),
            )),
    );

    assert_eq!(
        denial.code(),
        worth_server::WorthServerDenialCode::CompatHttpDiagnosticsBudgetExceeded
    );
    assert_eq!(
        denial.priority(),
        worth_server::WorthServerDenialPriority::Budget
    );
    assert_eq!(
        denial.step(),
        worth_server::WorthServerPipelineStep::BudgetPosture
    );
}
