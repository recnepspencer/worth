#[path = "support/test_bootstrap_policy.rs"]
mod test_bootstrap_policy;

use worth_proof::TransitionOutcome;
use worth_server::{
    WorthServerOperationFamily, WorthServerOperationReadinessDenialCode,
    WorthServerOperationRequestInput, WorthServerPipelineInput, WorthServerPipelineIntent,
    WorthServerQueryHandoffDenialFamily, WorthServerQueryHandoffInput,
    WorthServerQueryHandoffOperation,
};

#[path = "support/compat_http/phase_three_runtime.rs"]
mod compat_http_phase_three_runtime;
#[path = "support/operation_request/runtime.rs"]
mod operation_request_runtime;
#[path = "support/query_handoff/fixture.rs"]
mod query_handoff_fixture;
#[path = "support/query_handoff/runtime.rs"]
mod query_handoff_runtime;

use compat_http_phase_three_runtime::{
    build_phase_three_server, compat_mutation_denied, mutation_input, prepared_mutation_request,
    single_insert_body,
};
use operation_request_runtime::{operation_request_test_server, worth_native_resolved_context};
use query_handoff_fixture::{request_input, resolve_request_context, test_server};
use query_handoff_runtime::ProfiledTestWorkspaceProvider;

fn query_read_admission(server: &worth_server::WorthServer) -> worth_server::WorthServerAdmission {
    match server.middleware().admit(WorthServerPipelineInput::new(
        worth_native_resolved_context(server, None),
        WorthServerPipelineIntent::query_read("users.profile"),
    )) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected query read admission, got {other:?}"),
    }
}

#[test]
fn support_and_precondition_denials_do_not_fall_back_to_serialization() {
    let readiness_server = operation_request_test_server();
    let query_read_admission = query_read_admission(&readiness_server);
    let query_read_request = readiness_server
        .operation_requests()
        .admit_from_worth_native_admission(
            &query_read_admission,
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::QueryDirectRead)
                .with_operation_name("users.profile")
                .with_basis_digest("basis-users-profile")
                .build(),
        )
        .expect("query read request should admit");
    let query_read_posture = readiness_server
        .operation_admissions()
        .admit_declared(&query_read_admission, &query_read_request)
        .expect("query read posture should admit");

    let support_denial = readiness_server
        .operation_readiness()
        .compose_support(&query_read_posture, None)
        .expect_err("missing query support must deny instead of serialize");
    assert_eq!(
        support_denial.code(),
        WorthServerOperationReadinessDenialCode::MissingQuerySupport
    );

    let handoff_server = test_server(
        ProfiledTestWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile()
                .with_family_support(
                    worth_query::facade::runtime::WorthQueryRuntimeFamilySupport::unsupported(
                        worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily::Read,
                        "read support intentionally denied for phase-five hostility",
                    ),
                ),
        ),
        false,
    );
    let denial = match handoff_server
        .query_handoff()
        .prepare(WorthServerQueryHandoffInput::new(
            query_handoff_fixture::admit_read_posture(
                &handoff_server,
                resolve_request_context(
                    &handoff_server,
                    request_input(
                        worth_server::WorthServerSurfaceFamily::WorthNative,
                        worth_server::WorthServerTransportClass::WorthNativeInProcess,
                    ),
                ),
            ),
            WorthServerQueryHandoffOperation::query_read("users.profile"),
        )) {
        TransitionOutcome::Denied(value) => value,
        other => panic!("expected denied query handoff, got {other:?}"),
    };
    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily
    );
    assert_eq!(
        denial.family(),
        WorthServerQueryHandoffDenialFamily::Support
    );

    let compat_server = build_phase_three_server();
    let compat_denial = compat_mutation_denied(
        compat_server.compat_http().mutate(
            worth_server::WorthServerCompatibilityMutationExecutionInput::new(
                prepared_mutation_request(
                    &compat_server,
                    mutation_input("tasks.insert")
                        .with_query_pair("basis", "basis:drifted")
                        .build()
                        .expect("compat mutation input should validate"),
                ),
                "tasks.insert",
                single_insert_body("task-1"),
            ),
        ),
    );
    assert_eq!(
        compat_denial.family(),
        WorthServerQueryHandoffDenialFamily::Precondition
    );
}
