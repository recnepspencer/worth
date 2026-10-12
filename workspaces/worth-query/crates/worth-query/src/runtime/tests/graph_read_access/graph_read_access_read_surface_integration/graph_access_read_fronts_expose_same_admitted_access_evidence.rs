//! graph access read fronts expose same admitted access evidence.
use super::*;

#[test]
fn graph_access_read_fronts_expose_same_admitted_access_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut execute_workspace =
        workspace(request_execution, "graph-read-access.read-surface.execute");
    let execute_family = graph_access_family(&mut execute_workspace, "read-surface-parity");
    let execute_result = execute_workspace
        .execute_read_family(request_execution, &execute_family)
        .expect("execute_read_family should execute through admitted access plan");
    let execute_summary = execute_result
        .receipt()
        .graph_read_access_summary()
        .expect("execute_read_family receipt should expose access summary");

    let mut intent_workspace =
        workspace(request_execution, "graph-read-access.read-surface.intent");
    let intent_family = graph_access_family(&mut intent_workspace, "read-surface-parity");
    let intent_result = intent_workspace
        .read_family_intent(&intent_family)
        .execute(request_execution)
        .expect("intent helper should execute through admitted access plan");
    let intent_summary = intent_result
        .receipt()
        .graph_read_access_summary()
        .expect("intent receipt should expose access summary");

    let mut compose_workspace =
        workspace(request_execution, "graph-read-access.read-surface.compose");
    let compose_result =
        execute_graph_access_compose_read(request_execution, &mut compose_workspace)
            .expect("compose_read should execute through admitted access plan");
    let compose_summary = compose_result
        .receipt()
        .graph_read_access_summary()
        .expect("compose_read receipt should expose access summary");

    assert_admitted_summary(execute_summary);
    assert_admitted_summary(intent_summary);
    assert_admitted_summary(compose_summary);
    assert_eq!(
        execute_summary.admission_digest(),
        intent_summary.admission_digest()
    );
    assert_eq!(
        execute_summary.admission_digest(),
        compose_summary.admission_digest()
    );
    assert_eq!(execute_summary.plan_digest(), intent_summary.plan_digest());
    assert_eq!(execute_summary.plan_digest(), compose_summary.plan_digest());
    assert_eq!(
        execute_summary.requirement_set_digest(),
        compose_summary.requirement_set_digest()
    );
    assert_eq!(
        execute_summary.graph_index_inventory_match_report_digest(),
        compose_summary.graph_index_inventory_match_report_digest()
    );
    assert_eq!(
        execute_summary.execution_strategy(),
        compose_summary.execution_strategy()
    );
}
