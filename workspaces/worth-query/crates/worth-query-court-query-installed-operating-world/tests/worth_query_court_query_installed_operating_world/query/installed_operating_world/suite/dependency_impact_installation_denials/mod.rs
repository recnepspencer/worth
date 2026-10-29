pub(crate) use worth_query_testkit_query::facade::query::suite::dependency_impact_installation_denials::*;

#[test]
fn cyclic_workflow_is_denied_by_installation_before_dependency_impact_work() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_installation_denial(
        workspace_for(
            request_execution,
            "dependency-impact-cycle",
            Scenario::Cycle,
        ),
        "cyclic-or-unreachable-workflow-stage",
    );
}

#[test]
fn multiple_root_unreachable_workflow_is_denied_by_installation() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_installation_denial(
        workspace_for(
            request_execution,
            "dependency-impact-multiple-root",
            Scenario::MultipleRoot,
        ),
        "workflow-non-entry-root",
    );
}

#[test]
fn missing_predecessor_is_denied_by_installation() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_installation_denial(
        workspace_for(
            request_execution,
            "dependency-impact-missing-predecessor",
            Scenario::MissingPredecessor,
        ),
        "missing-workflow-predecessor",
    );
}

#[test]
fn graph_read_closure_mismatch_is_denied_by_installation() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_installation_denial(
        workspace_for(
            request_execution,
            "dependency-impact-graph-read-closure",
            Scenario::GraphReadClosure,
        ),
        "workflow-graph-read-closure-mismatch",
    );
}

#[test]
fn touch_closure_mismatch_is_denied_by_installation() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_installation_denial(
        workspace_for(
            request_execution,
            "dependency-impact-touch-closure",
            Scenario::TouchClosure,
        ),
        "workflow-touch-closure-mismatch",
    );
}

#[test]
fn effect_closure_mismatch_is_denied_by_installation() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_installation_denial(
        workspace_for(
            request_execution,
            "dependency-impact-effect-closure",
            Scenario::EffectClosure,
        ),
        "workflow-effect-closure-mismatch",
    );
}

#[test]
fn invariant_closure_mismatch_is_denied_by_installation() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_installation_denial(
        workspace_for(
            request_execution,
            "dependency-impact-invariant-closure",
            Scenario::InvariantClosure,
        ),
        "workflow-invariant-closure-mismatch",
    );
}
