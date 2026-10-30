#[allow(unused_imports)]
use crate::operation_request_boundaries::ordinary_context::*;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_compat_http_phase_three_runtime as compat_http_phase_three_runtime;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_query_handoff_runtime as query_handoff_runtime;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy;

#[allow(unused_imports)]
use worth_testkit_server::shared_support_worth_native_assertions as worth_native_assertions;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_worth_native_runtime as worth_native_runtime;

use worth_testkit_server::shared_support_worth_native_assertions::worth_native_session;
use {
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::compat_mutation_success,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::insert_task,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::single_insert_body,
};

#[test]
fn successful_mutation_boundaries_expose_the_canonical_operation_request() {
    let server = build_operation_name_guarded_server();
    let compat = compat_mutation_success(server.compat_http().mutate(
        worth_server::WorthServerCompatibilityMutationExecutionInput::new(
            prepared_mutation_request(&server, "Tasks.Insert"),
            "Tasks.Insert",
            single_insert_body("task-1"),
        ),
    ));
    let direct = direct_mutation_success(worth_native_session(&server).direct().mutate(
        &worth_server::WorthServerQueryOperation::single_mutation(
            "Tasks.Insert",
            insert_task("task-1"),
        ),
    ));

    assert_eq!(
        compat
            .envelope()
            .response_envelope()
            .success()
            .expect("compat response should succeed")
            .payload()
            .kind(),
        worth_server::WorthServerSuccessKind::QueryMutation
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
        worth_server::WorthServerCompatibilityMutationExecutionInput::new(
            prepared_mutation_request(&server, "tasks.unknown"),
            "tasks.unknown",
            single_insert_body("task-1"),
        ),
    ));

    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::UnknownOperationName
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
        &worth_server::WorthServerQueryOperation::single_mutation(
            "tasks.unknown",
            insert_task("task-1"),
        ),
    ));

    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::UnknownOperationName
    );
    assert_eq!(
        denial
            .facts()
            .and_then(|facts| facts.rejected_operation_name()),
        Some("tasks.unknown")
    );
}
