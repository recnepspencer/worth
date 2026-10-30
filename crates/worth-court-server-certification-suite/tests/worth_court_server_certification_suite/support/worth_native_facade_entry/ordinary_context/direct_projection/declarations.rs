pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_query::facade::foundation::ProjectionAuthorityOutcome,
    worth_query::facade::foundation::ProjectionConsumptionBindingContext,
    worth_query::facade::foundation::ProjectionFactConsumptionPathError,
    worth_query::facade::foundation::WorthQueryConsumedProjectionAuthority,
};
pub(crate) use {
    worth_server::WorthServerDirectProjectionOutcome,
    worth_server::WorthServerDirectProjectionRequest,
    worth_server::WorthServerQueryHandoffDenialCode, worth_server::WorthServerSuccessKind,
};

pub(crate) use {
    worth_testkit_server::shared_support_direct_context_runtime::RemaskWorkspaceProvider,
    worth_testkit_server::shared_support_worth_native_assertions::admitted_named_read,
    worth_testkit_server::shared_support_worth_native_assertions::worth_native_session,
    worth_testkit_server::shared_support_worth_native_runtime::build_server,
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

pub(crate) fn query_binding(
    request: &WorthServerDirectProjectionRequest,
    receipt: &worth_query::facade::runtime::WorthQueryLiveReadReceipt,
) -> ProjectionConsumptionBindingContext {
    worth_query::facade::foundation::ProjectionConsumptionBindingContext::from_projection_metadata(
        receipt.view_shape_digest(),
        receipt.canonical_query_digest(),
        receipt.view_shape_digest(),
        request.authorized_projection_identity(),
        request.narrowed_result_shape_digest(),
        request.policy_digest(),
        request.tenant_schema_basis_digest(),
        request.visible_fields().to_vec(),
    )
}

pub(crate) fn direct_read_success(
    outcome: worth_server::WorthServerDirectReadOutcome,
) -> worth_server::WorthServerDirectRead {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct read, got {other:?}"),
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

pub(crate) fn direct_projection_denied(
    outcome: WorthServerDirectProjectionOutcome,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied direct projection, got {other:?}"),
    }
}

pub(crate) fn query_projection_success(
    outcome: Result<ProjectionAuthorityOutcome, ProjectionFactConsumptionPathError>,
) -> WorthQueryConsumedProjectionAuthority {
    outcome
        .expect("query projection path should stay typed")
        .into_admitted()
        .map(|(authority, _)| authority)
        .unwrap_or_else(|other| panic!("expected admitted query projection path, got {other:?}"))
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
