use worth_foundational::facade::{
    DiagnosticRichnessProfile, FoundationalBoundaryEvidenceSupportTruthKind,
};
use worth_server::{
    WorthServerDenialBoundary, WorthServerMiddlewareConfig, WorthServerOperatorEvidenceConfig,
    WorthServerQueryHandoffDenialCode, WorthServerRequestContextDenialCode,
    WorthServerResponseInput, WorthServerResponseTransform,
};

use crate::shared_support_query_handoff_runtime::TestWorkspaceProvider;
use crate::shared_support_response_fixture::{
    middleware_mutation_denial, operator_evidence_record, query_handoff_durable_denial,
    resolve_blank_principal_denial, resolve_blank_workspace_denial, resolve_preview_denial,
    test_server, test_server_with_request_context_and_operator_evidence_config,
};

#[test]
fn shape_keeps_request_context_middleware_and_query_handoff_denials_distinct() {
    let server = test_server(
        TestWorkspaceProvider,
        WorthServerMiddlewareConfig::builder()
            .build()
            .expect("middleware config should validate"),
    );

    let request_context = server.responses().shape(
        WorthServerResponseInput::request_context_denied(resolve_preview_denial(&server)),
        WorthServerResponseTransform::compat_http(),
    );
    let middleware = server.responses().shape(
        WorthServerResponseInput::middleware_denied(middleware_mutation_denial(&server)),
        WorthServerResponseTransform::compat_http(),
    );
    let query_handoff = server.responses().shape(
        WorthServerResponseInput::query_handoff_denied(query_handoff_durable_denial(&server)),
        WorthServerResponseTransform::compat_http(),
    );

    let request_context_denial = request_context.denial().expect("request context denial");
    assert_eq!(
        request_context_denial.cause().boundary(),
        WorthServerDenialBoundary::RequestContext
    );
    assert_eq!(
        request_context_denial.request_context_code(),
        Some(WorthServerRequestContextDenialCode::PreviewTargetingDisabled)
    );
    assert_eq!(
        request_context_denial.diagnostics_profile(),
        DiagnosticRichnessProfile::Standard
    );

    let middleware_denial = middleware.denial().expect("middleware denial");
    assert_eq!(
        middleware_denial.cause().boundary(),
        WorthServerDenialBoundary::Middleware
    );
    assert_eq!(
        middleware_denial.middleware_code(),
        Some(worth_server::WorthServerDenialCode::QueryMutationDisabled)
    );
    assert_eq!(
        middleware_denial.diagnostics_profile(),
        DiagnosticRichnessProfile::Standard
    );
    assert!(middleware_denial.middleware_priority().is_some());
    assert!(middleware_denial.middleware_step().is_some());

    let query_handoff_denial = query_handoff.denial().expect("query handoff denial");
    assert_eq!(
        query_handoff_denial.cause().boundary(),
        WorthServerDenialBoundary::QueryHandoff
    );
    assert_eq!(
        query_handoff_denial.query_handoff_code(),
        Some(WorthServerQueryHandoffDenialCode::DurableResumeDeferred)
    );
    assert_eq!(
        query_handoff_denial.diagnostics_profile(),
        DiagnosticRichnessProfile::Standard
    );

    assert_ne!(
        request_context.canonical_digest(),
        middleware.canonical_digest()
    );
    assert_ne!(
        middleware.canonical_digest(),
        query_handoff.canonical_digest()
    );
    assert_ne!(
        request_context.canonical_digest(),
        query_handoff.canonical_digest()
    );
}

#[test]
fn shape_keeps_auth_workspace_and_branch_request_context_denials_distinct() {
    let server = test_server(
        TestWorkspaceProvider,
        WorthServerMiddlewareConfig::builder()
            .build()
            .expect("middleware config should validate"),
    );

    let auth_denial = server.responses().shape(
        WorthServerResponseInput::request_context_denied(resolve_blank_principal_denial(&server)),
        WorthServerResponseTransform::compat_http(),
    );
    let workspace_denial = server.responses().shape(
        WorthServerResponseInput::request_context_denied(resolve_blank_workspace_denial(&server)),
        WorthServerResponseTransform::compat_http(),
    );
    let branch_denial = server.responses().shape(
        WorthServerResponseInput::request_context_denied(resolve_preview_denial(&server)),
        WorthServerResponseTransform::compat_http(),
    );

    let auth_denial = auth_denial.denial().expect("auth denial");
    let workspace_denial = workspace_denial.denial().expect("workspace denial");
    let branch_denial = branch_denial.denial().expect("branch denial");

    assert_eq!(
        auth_denial.cause().boundary(),
        WorthServerDenialBoundary::RequestContext
    );
    assert_eq!(
        auth_denial.request_context_code(),
        Some(WorthServerRequestContextDenialCode::InvalidAuthenticatedPrincipal)
    );
    assert_eq!(
        workspace_denial.request_context_code(),
        Some(WorthServerRequestContextDenialCode::InvalidWorkspaceTarget)
    );
    assert_eq!(
        branch_denial.request_context_code(),
        Some(WorthServerRequestContextDenialCode::PreviewTargetingDisabled)
    );

    assert_ne!(
        auth_denial.canonical_digest(),
        workspace_denial.canonical_digest()
    );
    assert_ne!(
        auth_denial.canonical_digest(),
        branch_denial.canonical_digest()
    );
    assert_ne!(
        workspace_denial.canonical_digest(),
        branch_denial.canonical_digest()
    );
}

#[test]
fn operator_evidence_reconstructs_denial_classification_without_logs() {
    let server = test_server(
        TestWorkspaceProvider,
        WorthServerMiddlewareConfig::builder()
            .build()
            .expect("middleware config should validate"),
    );

    let request_context_evidence = operator_evidence_record(
        &server,
        server.responses().shape(
            WorthServerResponseInput::request_context_denied(resolve_blank_principal_denial(
                &server,
            )),
            WorthServerResponseTransform::compat_http(),
        ),
    );
    let middleware_evidence = operator_evidence_record(
        &server,
        server.responses().shape(
            WorthServerResponseInput::middleware_denied(middleware_mutation_denial(&server)),
            WorthServerResponseTransform::compat_http(),
        ),
    );
    let query_handoff_evidence = operator_evidence_record(
        &server,
        server.responses().shape(
            WorthServerResponseInput::query_handoff_denied(query_handoff_durable_denial(&server)),
            WorthServerResponseTransform::compat_http(),
        ),
    );

    assert!(matches!(
        request_context_evidence.classification(),
        worth_server::WorthServerOperatorEvidenceClass::RequestContextDenied(
            WorthServerRequestContextDenialCode::InvalidAuthenticatedPrincipal
        )
    ));
    assert!(matches!(
        middleware_evidence.classification(),
        worth_server::WorthServerOperatorEvidenceClass::MiddlewareDenied(
            worth_server::WorthServerDenialCode::QueryMutationDisabled
        )
    ));
    assert!(matches!(
        query_handoff_evidence.classification(),
        worth_server::WorthServerOperatorEvidenceClass::QueryHandoffDenied(
            WorthServerQueryHandoffDenialCode::DurableResumeDeferred
        )
    ));

    assert_eq!(
        request_context_evidence
            .counter_receipt()
            .counter("response.request_context_denial.count")
            .expect("request-context counter")
            .exact_value(),
        1
    );
    assert_eq!(
        middleware_evidence
            .counter_receipt()
            .counter("response.middleware_denial.count")
            .expect("middleware counter")
            .exact_value(),
        1
    );
    assert_eq!(
        query_handoff_evidence
            .counter_receipt()
            .counter("response.query_handoff_denial.count")
            .expect("query-handoff counter")
            .exact_value(),
        1
    );
    assert_eq!(
        query_handoff_evidence
            .counter_receipt()
            .counter("response.unsupported_capability.count")
            .expect("unsupported capability counter")
            .exact_value(),
        1
    );
    assert!(request_context_evidence
        .materialized_attachment_bundle()
        .support()
        .is_some());
}

#[test]
fn operator_evidence_preserves_denial_support_truth_when_operational_minimal_elides_support() {
    let server = test_server_with_request_context_and_operator_evidence_config(
        TestWorkspaceProvider,
        WorthServerMiddlewareConfig::builder()
            .build()
            .expect("middleware config should validate"),
        worth_server::WorthServerRequestContextConfig::builder()
            .with_default_diagnostics_profile(DiagnosticRichnessProfile::OperationalMinimal)
            .build()
            .expect("request context config should validate"),
        worth_server::WorthServerResponseConfig::builder()
            .with_denial_minimum_diagnostics_profile(DiagnosticRichnessProfile::OperationalMinimal)
            .build()
            .expect("response config should validate"),
        WorthServerOperatorEvidenceConfig::builder()
            .with_minimum_diagnostics_profile(DiagnosticRichnessProfile::OperationalMinimal)
            .build()
            .expect("operator evidence config should validate"),
    );

    let evidence = operator_evidence_record(
        &server,
        server.responses().shape(
            WorthServerResponseInput::request_context_denied(resolve_blank_principal_denial(
                &server,
            )),
            WorthServerResponseTransform::compat_http(),
        ),
    );

    assert_eq!(
        evidence.support_truth_kind(),
        FoundationalBoundaryEvidenceSupportTruthKind::DegradedRecoveryReport
    );
    assert!(evidence
        .materialized_attachment_bundle()
        .support()
        .is_none());
}
