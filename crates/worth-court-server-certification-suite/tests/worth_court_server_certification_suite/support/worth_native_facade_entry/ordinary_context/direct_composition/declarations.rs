pub(crate) use std::sync::atomic::Ordering;
pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
pub(crate) use {
    worth_server::WorthServerDirectDeclaration, worth_server::WorthServerDirectProjectionOutcome,
    worth_server::WorthServerDirectProjectionRequest, worth_server::WorthServerDirectReadOutcome,
    worth_server::WorthServerDirectStateOutcome, worth_server::WorthServerQueryHandoffInput,
    worth_server::WorthServerQueryHandoffOperation, worth_server::WorthServerSuccessKind,
    worth_server::WorthServerSurfaceFamily, worth_server::WorthServerTransportClass,
};

pub(crate) use {
    worth_testkit_server::shared_support_direct_context_runtime::RemaskWorkspaceProvider,
    worth_testkit_server::shared_support_query_handoff_fixture::admit_read_posture,
    worth_testkit_server::shared_support_query_handoff_fixture::request_input,
    worth_testkit_server::shared_support_query_handoff_fixture::resolve_request_context,
    worth_testkit_server::shared_support_query_handoff_fixture::success,
    worth_testkit_server::shared_support_worth_native_assertions::admitted_named_read,
    worth_testkit_server::shared_support_worth_native_assertions::direct_provenance_digest,
    worth_testkit_server::shared_support_worth_native_assertions::family_contract_digest,
    worth_testkit_server::shared_support_worth_native_assertions::response_provenance_digest,
    worth_testkit_server::shared_support_worth_native_assertions::worth_native_session,
    worth_testkit_server::shared_support_worth_native_runtime::build_server,
    worth_testkit_server::shared_support_worth_native_runtime::build_server_with_profiled_counting_workspace,
    worth_testkit_server::shared_support_worth_native_runtime::build_server_with_workspace_provider,
};

pub(crate) fn projection_request() -> WorthServerDirectProjectionRequest {
    worth_server::WorthServerDirectProjectionRequest::new(
        "authorized-projection:test",
        "narrowed-result-shape:test",
        "policy:test",
        "tenant-schema:test",
        ["identity.id", "profile.display_name"],
    )
}

pub(crate) fn direct_read_success(
    outcome: WorthServerDirectReadOutcome,
) -> worth_server::WorthServerDirectRead {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct read, got {other:?}"),
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

pub(crate) fn direct_state_success(
    outcome: WorthServerDirectStateOutcome,
) -> worth_server::WorthServerDirectState {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct state, got {other:?}"),
    }
}

pub(crate) fn direct_retained_posture_success(
    outcome: worth_proof::TransitionOutcome<
        worth_server::WorthServerDirectRetainedPosture,
        worth_server::WorthServerQueryHandoffDenial,
        worth_server::WorthServerQueryHandoffDeferred,
        worth_server::WorthServerQueryHandoffStale,
        worth_server::WorthServerQueryHandoffRebindRequired,
        worth_server::WorthServerQueryHandoffFailure,
    >,
) -> worth_server::WorthServerDirectRetainedPosture {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected successful retained posture, got {other:?}"),
    }
}

pub(crate) fn direct_projection_success(
    outcome: WorthServerDirectProjectionOutcome,
) -> worth_server::WorthServerDirectProjection {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct projection, got {other:?}"),
    }
}

pub(crate) struct ProjectionRemaskTestSupport;

impl ProjectionRemaskTestSupport {
    pub(crate) fn projection() -> worth_query::facade::runtime::WorthQueryRuntimeRemaskProjection {
        worth_query::facade::runtime::WorthQueryRuntimeRemaskProjection::remasked(
            worth_query::facade::runtime::WorthQueryRuntimeRemaskReasonKind::PolicyDrift,
            "policy:test",
            "tenant-truth:test",
            "tenant-schema:test",
            "relationship-proof:test",
            "schema-context:test",
        )
    }
}
