pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_query::facade::runtime::WorthQueryAuthorityLane, worth_query::facade::runtime::WorthQueryInspection, worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily, worth_query::facade::runtime::WorthQueryRuntimeFamilySupport, worth_query::facade::runtime::WorthQueryRuntimeStateKind, worth_query::facade::runtime::WorthQueryRuntimeSupportProfile};
pub(crate) use {worth_server::WorthServerDirectInspectionOutcome, worth_server::WorthServerDirectReadOutcome, worth_server::WorthServerDirectStateOutcome, worth_server::WorthServerQueryHandoffDenialCode, worth_server::WorthServerQueryHandoffInput, worth_server::WorthServerQueryHandoffOperation, worth_server::WorthServerSuccessKind, worth_server::WorthServerSurfaceFamily, worth_server::WorthServerTransportClass};

pub(crate) use {worth_testkit_server::shared_support_query_handoff_fixture::admit_read_posture, worth_testkit_server::shared_support_query_handoff_fixture::request_input, worth_testkit_server::shared_support_query_handoff_fixture::resolve_request_context, worth_testkit_server::shared_support_query_handoff_fixture::success, worth_testkit_server::shared_support_worth_native_assertions::admitted_named_read, worth_testkit_server::shared_support_worth_native_assertions::family_contract_digest, worth_testkit_server::shared_support_worth_native_assertions::worth_native_session, worth_testkit_server::shared_support_worth_native_runtime::build_server, worth_testkit_server::shared_support_worth_native_runtime::build_server_with_profiled_workspace};

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

pub(crate) fn direct_state_denied(
    outcome: WorthServerDirectStateOutcome,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied direct state, got {other:?}"),
    }
}

pub(crate) fn direct_inspection_denied(
    outcome: WorthServerDirectInspectionOutcome,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied direct inspection, got {other:?}"),
    }
}

pub(crate) fn direct_read_denied(
    outcome: WorthServerDirectReadOutcome,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied direct read, got {other:?}"),
    }
}

pub(crate) fn build_server_with_workspace_provider() -> worth_server::WorthServer {
    build_server_with_profiled_workspace(worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile())
}