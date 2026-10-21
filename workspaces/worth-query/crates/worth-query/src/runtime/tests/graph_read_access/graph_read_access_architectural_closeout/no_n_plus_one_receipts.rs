use crate::runtime::WorthQueryGraphReadAccessRequirementKind;

use crate::runtime::tests::graph_read_access::support::graph_index_inventory::runtime_profiles::{
    profile_with_ephemeral_graph_support, workspace_with_graph_support,
};
use crate::runtime::tests::graph_read_access::support::graph_read_access::read_surface_assertions::assert_success_counters_are_executor_observed;
use crate::runtime::tests::graph_read_access::support::graph_read_access::read_surface_declarations::graph_access_family;

#[test]
fn closeout_covered_read_receipt_proves_zero_caller_owned_n_plus_one_paths() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace_with_graph_support(
        request_execution,
        "graph-read-access.closeout.no-n-plus-one.ephemeral",
        profile_with_ephemeral_graph_support(
            WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
        ),
    );
    let family = graph_access_family(&mut workspace, "closeout-no-n-plus-one");
    let result = workspace
        .execute_read_family(request_execution, &family)
        .expect("covered read should execute through admitted access plan");
    let counters = result
        .receipt()
        .graph_read_access_complexity_counters()
        .expect("receipt should expose access complexity counters");
    let consumption = result
        .receipt()
        .graph_read_access_plan_consumption()
        .expect("receipt should expose access-plan consumption");

    assert_success_counters_are_executor_observed(counters);
    assert_eq!(consumption.execution_counters().edge_scan_count(), 0);
    assert_eq!(
        consumption
            .execution_counters()
            .per_result_neighbor_lookup_count(),
        0
    );
    assert_eq!(
        consumption
            .execution_counters()
            .persistent_artifact_bypass_count(),
        0
    );
}
