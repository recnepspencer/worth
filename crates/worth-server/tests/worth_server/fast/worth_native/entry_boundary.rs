use worth_proof::TransitionOutcome;
use worth_server::{
    request_context::DiagnosticRichnessProfile, WorthServerDenialCode, WorthServerPipelineInput,
    WorthServerPipelineIntent, WorthServerPreparedQueryHandoffKind,
    WorthServerRequestContextDenialCode, WorthServerWorthNativeSessionDenialCode,
};

use crate::shared_support_worth_native_runtime::{
    build_server, build_server_with_disabled_worth_native, build_server_with_preview_denial,
    denied_prepared_session, denied_session, expect_preview_access_denial,
    resolve_worth_native_request_context, server_with_request_context_default,
    worth_native_session_input_builder,
};

#[test]
fn worth_native_prepare_session_matches_lower_lane_request_context_and_admission() {
    let server = build_server(true);

    let prepared = match server.worth_native().prepare_session(
        worth_native_session_input_builder()
            .with_branch_id("branch-9")
            .build()
            .expect("Worth-native session input should validate"),
    ) {
        TransitionOutcome::Success(prepared) => prepared,
        other => panic!("expected prepared session, got {other:?}"),
    };

    let lower_lane_context = resolve_worth_native_request_context(&server);
    let lower_lane_admission = match server.middleware().admit(WorthServerPipelineInput::new(
        lower_lane_context.clone(),
        WorthServerPipelineIntent::worth_native_session("worth_native.session"),
    )) {
        TransitionOutcome::Success(admission) => admission,
        other => panic!("expected lower-lane admission, got {other:?}"),
    };

    assert_eq!(prepared.resolved_request_context(), &lower_lane_context);
    assert_eq!(prepared.admission(), &lower_lane_admission);
    assert_eq!(
        prepared.admission().query_handoff_intent().kind(),
        WorthServerPreparedQueryHandoffKind::WorthNativeSession
    );
    assert_eq!(
        prepared.admission().query_handoff_intent().operation_name(),
        "worth_native.session"
    );
}

#[test]
fn worth_native_common_session_lane_wraps_prepared_session_without_semantic_drift() {
    let server = build_server(true);

    let session = match server.worth_native().session(
        worth_native_session_input_builder()
            .with_branch_id("branch-9")
            .build()
            .expect("Worth-native session input should validate"),
    ) {
        TransitionOutcome::Success(session) => session,
        other => panic!("expected admitted session, got {other:?}"),
    };

    assert_eq!(
        session
            .resolved_request_context()
            .request_context()
            .workspace_target()
            .workspace_id(),
        "workspace-42"
    );
    assert_eq!(
        session.admission().query_handoff_intent().kind(),
        WorthServerPreparedQueryHandoffKind::WorthNativeSession
    );
}

#[test]
fn worth_native_prepare_session_denies_when_surface_family_is_absent() {
    let server = build_server(false);

    let denial = denied_prepared_session(
        server.worth_native().prepare_session(
            worth_native_session_input_builder()
                .build()
                .expect("Worth-native session input should validate"),
        ),
    );

    assert_eq!(
        denial.code(),
        WorthServerWorthNativeSessionDenialCode::WorthNativeSurfaceAbsent
    );
    assert_eq!(
        denial.diagnostics_profile(),
        DiagnosticRichnessProfile::Standard
    );
    assert_eq!(
        denial.detail(),
        "Worth-native surface family is not registered on this server"
    );
    assert!(denial.request_context_denial().is_none());
    assert!(denial.middleware_denial().is_none());
}

#[test]
fn worth_native_prepare_session_denies_when_surface_family_is_disabled() {
    let server = build_server_with_disabled_worth_native();

    let denial = denied_prepared_session(
        server.worth_native().prepare_session(
            worth_native_session_input_builder()
                .with_branch_id("branch-9")
                .build()
                .expect("Worth-native session input should validate"),
        ),
    );

    assert_eq!(
        denial.code(),
        WorthServerWorthNativeSessionDenialCode::WorthNativeSurfaceDisabled
    );
    assert_eq!(
        denial.diagnostics_profile(),
        DiagnosticRichnessProfile::Standard
    );
    assert_eq!(
        denial.detail(),
        "Worth-native surface family is registered but disabled on this server"
    );
    assert!(denial.request_context_denial().is_none());
    assert!(denial.middleware_denial().is_none());
}

#[test]
fn worth_native_prepare_session_preserves_request_context_denial_artifact() {
    let server = build_server(true);

    let denial = denied_prepared_session(
        server.worth_native().prepare_session(
            worth_native_session_input_builder()
                .with_branch_id("")
                .build()
                .expect("Worth-native session input should validate"),
        ),
    );

    assert_eq!(
        denial.code(),
        WorthServerWorthNativeSessionDenialCode::RequestContextDenied
    );
    assert_eq!(
        denial
            .request_context_denial()
            .expect("request-context denial should be preserved")
            .code(),
        WorthServerRequestContextDenialCode::InvalidBranchTarget
    );
    assert!(denial.middleware_denial().is_none());
}

#[test]
fn worth_native_prepare_session_preserves_middleware_denial_artifact() {
    let server = build_server_with_preview_denial();

    let denial = denied_prepared_session(
        server.worth_native().prepare_session(
            worth_native_session_input_builder()
                .with_preview_id("preview-7")
                .build()
                .expect("Worth-native session input should validate"),
        ),
    );

    assert_eq!(
        denial.code(),
        WorthServerWorthNativeSessionDenialCode::MiddlewareDenied
    );
    expect_preview_access_denial(&denial);
    assert!(denial.request_context_denial().is_none());
}

#[test]
fn worth_native_prepare_session_honors_requested_diagnostics_profile_for_surface_absence() {
    let server = build_server(false);

    let denial = denied_prepared_session(
        server.worth_native().prepare_session(
            worth_native_session_input_builder()
                .with_diagnostics_profile(DiagnosticRichnessProfile::Forensic)
                .build()
                .expect("Worth-native session input should validate"),
        ),
    );

    assert_eq!(
        denial.code(),
        WorthServerWorthNativeSessionDenialCode::WorthNativeSurfaceAbsent
    );
    assert_eq!(
        denial.diagnostics_profile(),
        DiagnosticRichnessProfile::Forensic
    );
}

#[test]
fn worth_native_session_wrapper_preserves_denial_topology_without_reclassification() {
    let request_context_denial = denied_session(
        server_with_request_context_default(DiagnosticRichnessProfile::OperationalMinimal)
            .worth_native()
            .session(
                worth_native_session_input_builder()
                    .with_branch_id("")
                    .build()
                    .expect("Worth-native session input should validate"),
            ),
    );
    let middleware_denial = denied_session(
        build_server_with_preview_denial().worth_native().session(
            worth_native_session_input_builder()
                .with_preview_id("preview-7")
                .build()
                .expect("Worth-native session input should validate"),
        ),
    );

    assert_eq!(
        request_context_denial.code(),
        WorthServerWorthNativeSessionDenialCode::RequestContextDenied
    );
    assert_eq!(
        request_context_denial.diagnostics_profile(),
        DiagnosticRichnessProfile::OperationalMinimal
    );
    assert_eq!(
        request_context_denial
            .request_context_denial()
            .expect("request-context denial should survive session wrapping")
            .code(),
        WorthServerRequestContextDenialCode::InvalidBranchTarget
    );
    assert!(request_context_denial.middleware_denial().is_none());

    assert_eq!(
        middleware_denial.code(),
        WorthServerWorthNativeSessionDenialCode::MiddlewareDenied
    );
    assert_eq!(
        middleware_denial
            .middleware_denial()
            .expect("middleware denial should survive session wrapping")
            .code(),
        WorthServerDenialCode::PreviewBranchAccessDenied
    );
    assert!(middleware_denial.request_context_denial().is_none());
}
