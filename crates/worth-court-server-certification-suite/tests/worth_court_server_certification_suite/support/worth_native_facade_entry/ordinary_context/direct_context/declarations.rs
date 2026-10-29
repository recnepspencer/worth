pub(crate) use {worth_foundational::facade::DiagnosticRichnessProfile};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_query::facade::runtime::WorthQueryInspection, worth_query::facade::runtime::WorthQueryRuntimeRemaskProjection, worth_query::facade::runtime::WorthQueryRuntimeRemaskReasonKind, worth_query::facade::runtime::WorthQueryRuntimeStateKind};
pub(crate) use {worth_server::WorthServerDirectInspectionOutcome, worth_server::WorthServerDirectReadOutcome, worth_server::WorthServerDirectRemaskDisposition, worth_server::WorthServerDirectStateOutcome, worth_server::WorthServerResponseInput, worth_server::WorthServerSurfaceFamily, worth_server::WorthServerTransportClass};

pub(crate) use {worth_testkit_server::shared_support_direct_context_runtime::RemaskWorkspaceProvider, worth_testkit_server::shared_support_query_handoff_fixture::admit_read_posture, worth_testkit_server::shared_support_query_handoff_fixture::request_input, worth_testkit_server::shared_support_query_handoff_fixture::resolve_request_context, worth_testkit_server::shared_support_query_handoff_fixture::success, worth_testkit_server::shared_support_worth_native_assertions::admitted_named_read, worth_testkit_server::shared_support_worth_native_assertions::direct_provenance_digest, worth_testkit_server::shared_support_worth_native_assertions::response_provenance_digest, worth_testkit_server::shared_support_worth_native_assertions::worth_native_session, worth_testkit_server::shared_support_worth_native_runtime::build_server, worth_testkit_server::shared_support_worth_native_runtime::build_server_with_workspace_provider, worth_testkit_server::shared_support_worth_native_runtime::worth_native_session_input_builder};

pub(crate) fn branch_session(
    server: &worth_server::WorthServer,
    branch_id: &str,
) -> worth_server::WorthServerWorthNativeSession {
    match server.worth_native().session(
        worth_native_session_input_builder()
            .with_branch_id(branch_id)
            .build()
            .expect("branch session input should validate"),
    ) {
        worth_proof::TransitionOutcome::Success(session) => session,
        other => panic!("expected branch session, got {other:?}"),
    }
}

pub(crate) fn session_with_diagnostics(
    server: &worth_server::WorthServer,
    diagnostics_profile: DiagnosticRichnessProfile,
) -> worth_server::WorthServerWorthNativeSession {
    match server.worth_native().session(
        worth_native_session_input_builder()
            .with_diagnostics_profile(diagnostics_profile)
            .build()
            .expect("diagnostics session input should validate"),
    ) {
        worth_proof::TransitionOutcome::Success(session) => session,
        other => panic!("expected diagnostics session, got {other:?}"),
    }
}

pub(crate) fn direct_read_success(
    outcome: WorthServerDirectReadOutcome,
) -> worth_server::WorthServerDirectRead {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct read, got {other:?}"),
    }
}

pub(crate) fn direct_state_success(
    outcome: WorthServerDirectStateOutcome,
) -> worth_server::WorthServerDirectState {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct state, got {other:?}"),
    }
}

pub(crate) fn direct_inspection_success(
    outcome: WorthServerDirectInspectionOutcome,
) -> worth_server::WorthServerDirectInspection {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct inspection, got {other:?}"),
    }
}

pub(crate) fn compat_read_handoff(
    server: &worth_server::WorthServer,
) -> worth_server::WorthServerQueryHandoff {
    success(
        server
            .query_handoff()
            .prepare(worth_server::WorthServerQueryHandoffInput::new(
                admit_read_posture(
                    server,
                    resolve_request_context(
                        server,
                        request_input(
                            worth_server::WorthServerSurfaceFamily::CompatHttp,
                            worth_server::WorthServerTransportClass::CompatHttp,
                        ),
                    ),
                ),
                worth_server::WorthServerQueryHandoffOperation::query_read("users.profile"),
            )),
    )
}