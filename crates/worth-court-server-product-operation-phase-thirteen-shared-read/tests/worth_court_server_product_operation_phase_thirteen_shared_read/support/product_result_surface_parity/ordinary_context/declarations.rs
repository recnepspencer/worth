#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy};
pub(crate) use {serde_json::json};
pub(crate) use {worth_server::request_context::DiagnosticRichnessProfile, worth_server::WorthServerProductIdempotencyKey, worth_server::WorthServerProductOperationInput, worth_server::WorthServerProductOperationPayload, worth_server::WorthServerWorthNativeSession, worth_server::WorthServerWorthNativeSessionInput};

#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_durable_product_mutation as durable_support};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_route_assembly_phase_twelve_request_driver as request_driver};

pub(crate) fn durable_host_input(idempotency_key: &str) -> WorthServerProductOperationInput {
    worth_server::WorthServerProductOperationInput::new(
        "product.host_connection.upsert",
        worth_server::WorthServerProductOperationPayload::json(
            "product.host-connection.upsert.v1",
            json!({ "connection_id": "host-7" }),
        ),
    )
    .with_basis_digest("basis:0")
    .with_idempotency_key(
        worth_server::WorthServerProductIdempotencyKey::new(idempotency_key)
            .expect("test idempotency key should validate"),
    )
}

pub(crate) fn session_with_diagnostics(
    server: &worth_server::WorthServer,
    diagnostics: DiagnosticRichnessProfile,
) -> WorthServerWorthNativeSession {
    use {worth_proof::TransitionOutcome};
    match server.worth_native().session(
        worth_server::WorthServerWorthNativeSessionInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_branch_id("branch-9")
            .with_diagnostics_profile(diagnostics)
            .build()
            .expect("diagnostic session input should validate"),
    ) {
        TransitionOutcome::Success(session) => session,
        other => panic!("expected diagnostic session, got {other:?}"),
    }
}