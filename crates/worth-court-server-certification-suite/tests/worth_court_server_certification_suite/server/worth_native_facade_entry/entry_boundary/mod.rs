use worth_proof::TransitionOutcome;
use {
    worth_server::request_context::DiagnosticRichnessProfile, worth_server::WorthServerDenialCode,
    worth_server::WorthServerPipelineInput, worth_server::WorthServerPipelineIntent,
    worth_server::WorthServerPreparedQueryHandoffKind,
    worth_server::WorthServerRequestContextDenialCode,
    worth_server::WorthServerWorthNativeSessionDenialCode,
};

use {
    worth_testkit_server::shared_support_worth_native_runtime::build_server,
    worth_testkit_server::shared_support_worth_native_runtime::build_server_with_disabled_worth_native,
    worth_testkit_server::shared_support_worth_native_runtime::build_server_with_preview_denial,
    worth_testkit_server::shared_support_worth_native_runtime::denied_prepared_session,
    worth_testkit_server::shared_support_worth_native_runtime::denied_session,
    worth_testkit_server::shared_support_worth_native_runtime::expect_preview_access_denial,
    worth_testkit_server::shared_support_worth_native_runtime::resolve_worth_native_request_context,
    worth_testkit_server::shared_support_worth_native_runtime::server_with_request_context_default,
    worth_testkit_server::shared_support_worth_native_runtime::worth_native_session_input_builder,
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
        worth_proof::TransitionOutcome::Success(prepared) => prepared,
        other => panic!("expected prepared session, got {other:?}"),
    };

    let lower_lane_context = resolve_worth_native_request_context(&server);
    let lower_lane_admission =
        match server
            .middleware()
            .admit(worth_server::WorthServerPipelineInput::new(
                lower_lane_context.clone(),
                worth_server::WorthServerPipelineIntent::worth_native_session(
                    "worth_native.session",
                ),
            )) {
            worth_proof::TransitionOutcome::Success(admission) => admission,
            other => panic!("expected lower-lane admission, got {other:?}"),
        };

    assert_eq!(prepared.resolved_request_context(), &lower_lane_context);
    assert_eq!(prepared.admission(), &lower_lane_admission);
    assert_eq!(
        prepared.admission().query_handoff_intent().kind(),
        worth_server::WorthServerPreparedQueryHandoffKind::WorthNativeSession
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
        worth_proof::TransitionOutcome::Success(session) => session,
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
        worth_server::WorthServerPreparedQueryHandoffKind::WorthNativeSession
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
        worth_server::WorthServerWorthNativeSessionDenialCode::WorthNativeSurfaceAbsent
    );
    assert_eq!(
        denial.diagnostics_profile(),
        worth_server::request_context::DiagnosticRichnessProfile::Standard
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
        worth_server::WorthServerWorthNativeSessionDenialCode::WorthNativeSurfaceDisabled
    );
    assert_eq!(
        denial.diagnostics_profile(),
        worth_server::request_context::DiagnosticRichnessProfile::Standard
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
        worth_server::WorthServerWorthNativeSessionDenialCode::RequestContextDenied
    );
    assert_eq!(
        denial
            .request_context_denial()
            .expect("request-context denial should be preserved")
            .code(),
        worth_server::WorthServerRequestContextDenialCode::InvalidBranchTarget
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
        worth_server::WorthServerWorthNativeSessionDenialCode::MiddlewareDenied
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
                .with_diagnostics_profile(
                    worth_server::request_context::DiagnosticRichnessProfile::Forensic,
                )
                .build()
                .expect("Worth-native session input should validate"),
        ),
    );

    assert_eq!(
        denial.code(),
        worth_server::WorthServerWorthNativeSessionDenialCode::WorthNativeSurfaceAbsent
    );
    assert_eq!(
        denial.diagnostics_profile(),
        worth_server::request_context::DiagnosticRichnessProfile::Forensic
    );
}

#[test]
fn worth_native_session_wrapper_preserves_denial_topology_without_reclassification() {
    let request_context_denial = denied_session(
        server_with_request_context_default(
            worth_server::request_context::DiagnosticRichnessProfile::OperationalMinimal,
        )
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
        worth_server::WorthServerWorthNativeSessionDenialCode::RequestContextDenied
    );
    assert_eq!(
        request_context_denial.diagnostics_profile(),
        worth_server::request_context::DiagnosticRichnessProfile::OperationalMinimal
    );
    assert_eq!(
        request_context_denial
            .request_context_denial()
            .expect("request-context denial should survive session wrapping")
            .code(),
        worth_server::WorthServerRequestContextDenialCode::InvalidBranchTarget
    );
    assert!(request_context_denial.middleware_denial().is_none());

    assert_eq!(
        middleware_denial.code(),
        worth_server::WorthServerWorthNativeSessionDenialCode::MiddlewareDenied
    );
    assert_eq!(
        middleware_denial
            .middleware_denial()
            .expect("middleware denial should survive session wrapping")
            .code(),
        worth_server::WorthServerDenialCode::PreviewBranchAccessDenied
    );
    assert!(middleware_denial.request_context_denial().is_none());
}
