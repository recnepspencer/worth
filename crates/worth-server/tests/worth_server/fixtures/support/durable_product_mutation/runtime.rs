use worth_proof::TransitionOutcome;
use worth_server::{
    surfaces::{CompatHttpSurface, WorthNativeSurface},
    WorthServer, WorthServerCompatibilityPreparedRequest, WorthServerOperationAuthorizationPolicy,
    WorthServerOperationFamily, WorthServerOperationRegistration,
    WorthServerProductApplicationAdapterRegistration, WorthServerWorthNativeSession,
    WorthServerWorthNativeSessionInput,
};

use super::{registration, TestDurableProductExecutor};

#[allow(unused_imports)]
use crate::shared_support_product_adapter_phase_nine_fixture as product_adapter_fixture;

pub use product_adapter_fixture::direct_session;
pub use product_adapter_fixture::schema_bound_json::publish_schema_bound_json;

pub fn build_server(executor: &TestDurableProductExecutor) -> WorthServer {
    product_adapter_fixture::build_server(vec![registration(executor.clone())])
}

pub fn build_server_with_registration(
    registration: WorthServerProductApplicationAdapterRegistration,
) -> WorthServer {
    product_adapter_fixture::build_server(vec![registration])
}

pub fn build_server_with_mutation_policy(
    executor: &TestDurableProductExecutor,
    policy: WorthServerOperationAuthorizationPolicy,
) -> WorthServer {
    let operation_registrations = WorthServerOperationRegistration::phase_two_defaults()
        .into_iter()
        .map(|registration| {
            if registration.family() == WorthServerOperationFamily::ProductApplicationMutation {
                registration.with_authorization_policy(policy.clone())
            } else {
                registration
            }
        })
        .collect::<Vec<_>>();
    WorthServer::builder()
        .with_config(product_adapter_fixture::base_config())
        .register_operations(operation_registrations)
        .register_surface(WorthNativeSurface::enabled())
        .register_surface(CompatHttpSurface::phase_one_enabled())
        .register_product_adapter(registration(executor.clone()))
        .build()
        .expect("policy-specific durable product server should build")
}

pub fn session(
    server: &WorthServer,
    tenant_id: &str,
    workspace_id: &str,
) -> WorthServerWorthNativeSession {
    session_with_principal(server, tenant_id, workspace_id, "principal-7")
}

pub fn session_with_principal(
    server: &WorthServer,
    tenant_id: &str,
    workspace_id: &str,
    principal_id: &str,
) -> WorthServerWorthNativeSession {
    match server.worth_native().session(
        WorthServerWorthNativeSessionInput::builder()
            .with_authenticated_principal_id(principal_id)
            .with_tenant_id(tenant_id)
            .with_workspace_id(workspace_id)
            .with_branch_id("branch-9")
            .build()
            .expect("test session input should validate"),
    ) {
        TransitionOutcome::Success(session) => session,
        other => panic!("expected Worth-native session, got {other:?}"),
    }
}

pub fn prepared_mutation_request(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCompatibilityPreparedRequest {
    product_adapter_fixture::prepared_mutation_request(server, operation_name, None)
}
