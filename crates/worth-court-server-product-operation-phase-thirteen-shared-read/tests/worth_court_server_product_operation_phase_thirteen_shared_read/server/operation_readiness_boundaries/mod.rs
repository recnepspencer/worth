#[allow(unused_imports)]
use crate::operation_readiness_boundaries::ordinary_context::*;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy;

#[allow(unused_imports)]
use worth_testkit_server::shared_support_compat_http_phase_three_runtime as compat_http_phase_three_runtime;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_operation_request_runtime as operation_request_runtime;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_query_handoff_fixture as query_handoff_fixture;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_query_handoff_runtime as query_handoff_runtime;

use worth_testkit_server::shared_support_operation_request_runtime::operation_request_test_server;
use {
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::build_phase_three_server,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::compat_mutation_denied,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::mutation_input,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::prepared_mutation_request,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::single_insert_body,
};
use {
    worth_testkit_server::shared_support_query_handoff_fixture::request_input,
    worth_testkit_server::shared_support_query_handoff_fixture::resolve_request_context,
    worth_testkit_server::shared_support_query_handoff_fixture::test_server,
};

#[test]
fn support_and_precondition_denials_do_not_fall_back_to_serialization() {
    let readiness_server = operation_request_test_server();
    let query_read_admission = query_read_admission(&readiness_server);
    let query_read_request = readiness_server
        .operation_requests()
        .admit_from_worth_native_admission(
            &query_read_admission,
            worth_server::WorthServerOperationRequestInput::builder()
                .with_operation_family(worth_server::WorthServerOperationFamily::QueryDirectRead)
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
        worth_server::WorthServerOperationReadinessDenialCode::MissingQuerySupport
    );

    let handoff_server = test_server(
        worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider::new(
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
    let denial = match handoff_server.query_handoff().prepare(
        worth_server::WorthServerQueryHandoffInput::new(
            worth_testkit_server::shared_support_query_handoff_fixture::admit_read_posture(
                &handoff_server,
                resolve_request_context(
                    &handoff_server,
                    request_input(
                        worth_server::WorthServerSurfaceFamily::WorthNative,
                        worth_server::WorthServerTransportClass::WorthNativeInProcess,
                    ),
                ),
            ),
            worth_server::WorthServerQueryHandoffOperation::query_read("users.profile"),
        ),
    ) {
        worth_proof::TransitionOutcome::Denied(value) => value,
        other => panic!("expected denied query handoff, got {other:?}"),
    };
    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily
    );
    assert_eq!(
        denial.family(),
        worth_server::WorthServerQueryHandoffDenialFamily::Support
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
        worth_server::WorthServerQueryHandoffDenialFamily::Precondition
    );
}
