#[allow(unused_imports)]
use crate::shared_support_compat_http_phase_three_runtime as compat_http_phase_three_runtime;
#[allow(unused_imports)]
use crate::shared_support_query_handoff_runtime as query_handoff_runtime;
#[allow(unused_imports)]
use crate::shared_support_test_bootstrap_policy as test_bootstrap_policy;

#[allow(unused_imports)]
use crate::shared_support_worth_native_assertions as worth_native_assertions;
#[allow(unused_imports)]
use crate::shared_support_worth_native_runtime as worth_native_runtime;

use worth_proof::TransitionOutcome;
use worth_server::{
    request_context::DiagnosticRichnessProfile,
    surfaces::{CompatHttpSurface, WorthNativeSurface},
    WorthServer, WorthServerCompatHttpRouteFamily, WorthServerCompatibilityMutationExecutionInput,
    WorthServerCompatibilityRequestInput, WorthServerConfig, WorthServerMiddlewareConfig,
    WorthServerOperationAuthorityDeclaration, WorthServerOperationAuthorizationPolicy,
    WorthServerOperationFamily, WorthServerOperationRegistration, WorthServerQueryHandoffConfig,
    WorthServerQueryHandoffDenial, WorthServerQueryHandoffDenialCode, WorthServerQueryOperation,
    WorthServerRequestContextConfig, WorthServerSuccessKind, WorthServerSurfaceFamily,
};

use compat_http_phase_three_runtime::{compat_mutation_success, insert_task, single_insert_body};
use query_handoff_runtime::RealMutationWorkspaceProvider;
use worth_native_assertions::worth_native_session;

#[test]
fn successful_mutation_boundaries_expose_the_canonical_operation_request() {
    let server = build_operation_name_guarded_server();
    let compat = compat_mutation_success(server.compat_http().mutate(
        WorthServerCompatibilityMutationExecutionInput::new(
            prepared_mutation_request(&server, "Tasks.Insert"),
            "Tasks.Insert",
            single_insert_body("task-1"),
        ),
    ));
    let direct = direct_mutation_success(worth_native_session(&server).direct().mutate(
        &WorthServerQueryOperation::single_mutation("Tasks.Insert", insert_task("task-1")),
    ));

    assert_eq!(
        compat
            .envelope()
            .response_envelope()
            .success()
            .expect("compat response should succeed")
            .payload()
            .kind(),
        WorthServerSuccessKind::QueryMutation
    );
    assert_eq!(
        compat.operation_request().identity().operation_name(),
        "tasks.insert"
    );
    assert_eq!(
        direct.operation_request().identity().operation_name(),
        "tasks.insert"
    );
}

#[test]
fn compat_mutation_denial_reports_unknown_operation_name_as_structured_facts() {
    let server = build_operation_name_guarded_server();
    let denial = compat_mutation_denied(server.compat_http().mutate(
        WorthServerCompatibilityMutationExecutionInput::new(
            prepared_mutation_request(&server, "tasks.unknown"),
            "tasks.unknown",
            single_insert_body("task-1"),
        ),
    ));

    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::UnknownOperationName
    );
    assert_eq!(
        denial
            .facts()
            .and_then(|facts| facts.rejected_operation_name()),
        Some("tasks.unknown")
    );
}

#[test]
fn direct_mutation_denial_reports_unknown_operation_name_as_structured_facts() {
    let server = build_operation_name_guarded_server();
    let denial = direct_mutation_denied(worth_native_session(&server).direct().mutate(
        &WorthServerQueryOperation::single_mutation("tasks.unknown", insert_task("task-1")),
    ));

    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::UnknownOperationName
    );
    assert_eq!(
        denial
            .facts()
            .and_then(|facts| facts.rejected_operation_name()),
        Some("tasks.unknown")
    );
}

fn build_operation_name_guarded_server() -> WorthServer {
    WorthServer::builder()
        .with_config(
            WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(
                    WorthServerRequestContextConfig::builder()
                        .with_preview_targeting_enabled(true)
                        .with_default_diagnostics_profile(DiagnosticRichnessProfile::Standard)
                        .build()
                        .expect("request context config should validate"),
                )
                .with_middleware_config(
                    WorthServerMiddlewareConfig::builder()
                        .with_query_mutation_enabled(true)
                        .build()
                        .expect("middleware config should validate"),
                )
                .with_query_handoff_config(
                    WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(RealMutationWorkspaceProvider)
                        .build()
                        .expect("query handoff config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operations(operation_name_guarded_registrations())
        .register_surface(WorthNativeSurface::enabled())
        .register_surface(CompatHttpSurface::phase_one_enabled())
        .build()
        .expect("operation-name guarded server should build")
}

fn operation_name_guarded_registrations() -> Vec<WorthServerOperationRegistration> {
    WorthServerOperationRegistration::phase_two_defaults()
        .into_iter()
        .filter(|registration| {
            registration.family() != WorthServerOperationFamily::QueryDirectSubmission
        })
        .chain([WorthServerOperationRegistration::enabled(
            WorthServerOperationFamily::QueryDirectSubmission,
        )
        .exposed_on([
            WorthServerSurfaceFamily::WorthNative,
            WorthServerSurfaceFamily::CompatHttp,
        ])
        .admit_operation_names(["tasks.insert"])
        .with_authority_declaration(
            WorthServerOperationAuthorityDeclaration::deterministic_submission(
                "query-write",
                "query-write-review",
                "derive-from-request",
                "derive-from-request",
            ),
        )
        .with_authorization_policy(WorthServerOperationAuthorizationPolicy::allow_authenticated())])
        .collect()
}

fn prepared_mutation_request(
    server: &WorthServer,
    operation_name: &str,
) -> worth_server::WorthServerCompatibilityPreparedRequest {
    match server.compat_http().prepare_request(
        WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_branch_id("branch-9")
            .with_route_family(WorthServerCompatHttpRouteFamily::Mutation)
            .with_method("POST")
            .with_path(format!("/compat/mutations/{operation_name}"))
            .with_header("accept", "application/json")
            .with_body_content_type("application/json")
            .with_body_present(true)
            .build()
            .expect("compat mutation request should validate"),
    ) {
        TransitionOutcome::Success(prepared) => prepared,
        other => panic!("expected prepared compatibility mutation request, got {other:?}"),
    }
}

fn compat_mutation_denied(
    outcome: worth_server::WorthServerCompatibilityMutationOutcome<
        worth_server::WorthServerCompatibilityMutation,
    >,
) -> WorthServerQueryHandoffDenial {
    match outcome {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied compat mutation, got {other:?}"),
    }
}

fn direct_mutation_success(
    outcome: worth_server::WorthServerDirectMutationOutcome,
) -> worth_server::WorthServerDirectMutation {
    match outcome {
        TransitionOutcome::Success(mutation) => mutation,
        other => panic!("expected direct mutation success, got {other:?}"),
    }
}

fn direct_mutation_denied(
    outcome: worth_server::WorthServerDirectMutationOutcome,
) -> WorthServerQueryHandoffDenial {
    match outcome {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied direct mutation, got {other:?}"),
    }
}
