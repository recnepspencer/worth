use crate::runtime::tests::graph_read_access::support::graph_read_access::read_surface_assertions::read_composition_denial;
use crate::runtime::tests::graph_read_access::support::graph_read_access::read_surface_declarations::{
    graph_access_family, unregistered_domain_operation_family,
};
use crate::runtime::tests::graph_read_access::support::public_bridge_runtime::PublicBridgeRuntimeHarness;

#[test]
fn closeout_access_plan_receipts_replay_to_stable_digests() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let first = execute_replayed_read(request_execution, "first");
    let second = execute_replayed_read(request_execution, "second");

    assert_eq!(first.plan_digest, second.plan_digest);
    assert_eq!(first.admission_digest, second.admission_digest);
    assert_eq!(first.requirement_set_digest, second.requirement_set_digest);
    assert_eq!(first.inventory_match_digest, second.inventory_match_digest);
    assert_eq!(first.summary_digest, second.summary_digest);
    assert_eq!(
        first.plan_consumption_digest,
        second.plan_consumption_digest
    );
    assert_eq!(
        first.execution_counter_envelope,
        second.execution_counter_envelope
    );
}

#[test]
fn closeout_replay_denies_wrong_family_plan_with_typed_digest_context() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut source = workspace(
        request_execution,
        "graph-read-access.closeout.replay.source",
    );
    let source_family = graph_access_family(&mut source, "closeout-replay-source");
    let plan = source
        .read_family_intent(&source_family)
        .review()
        .expect("source read should review")
        .graph_read_access_plan()
        .expect("source read should produce plan");
    let plan_digest = plan.digest().to_string();

    let mut target = workspace(
        request_execution,
        "graph-read-access.closeout.replay.target",
    );
    let target_family = unregistered_domain_operation_family(&mut target, "closeout-replay-target");
    let denial = read_composition_denial(
        target
            .execute_read_family_with_access_plan(request_execution, &target_family, plan)
            .expect_err("wrong family plan should deny before execution"),
    );
    let mismatch = denial
        .access_plan_binding_mismatch()
        .expect("wrong family plan should carry typed mismatch proof");

    assert_eq!(mismatch.provided_plan_digest(), plan_digest);
    assert_eq!(
        mismatch.execution_read_graph_digest(),
        target_family.read_graph().digest()
    );
}

struct ReplayReceiptDigestSet {
    plan_digest: String,
    admission_digest: String,
    requirement_set_digest: String,
    inventory_match_digest: String,
    summary_digest: String,
    plan_consumption_digest: String,
    execution_counter_envelope: ReplayExecutionCounterEnvelope,
}

fn execute_replayed_read(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    label: &str,
) -> ReplayReceiptDigestSet {
    let mut workspace = workspace(
        request_execution,
        &format!("graph-read-access.closeout.replay.{label}"),
    );
    let family = graph_access_family(&mut workspace, "closeout-replay-family");
    let result = workspace
        .execute_read_family(request_execution, &family)
        .expect("replayed read should execute");
    let summary = result
        .receipt()
        .graph_read_access_summary()
        .expect("receipt should expose access summary");
    let consumption = result
        .receipt()
        .graph_read_access_plan_consumption()
        .expect("receipt should expose plan consumption");

    ReplayReceiptDigestSet {
        plan_digest: summary.plan_digest().to_string(),
        admission_digest: summary.admission_digest().to_string(),
        requirement_set_digest: summary.requirement_set_digest().to_string(),
        inventory_match_digest: summary
            .graph_index_inventory_match_report_digest()
            .to_string(),
        summary_digest: summary.digest().to_string(),
        plan_consumption_digest: consumption.digest().to_string(),
        execution_counter_envelope: ReplayExecutionCounterEnvelope::from_consumption(consumption),
    }
}

#[derive(Debug, Eq, PartialEq)]
struct ReplayExecutionCounterEnvelope {
    executor_entry_count: usize,
    strategy_recompute_count: usize,
    ephemeral_index_allocation_count: usize,
    edge_scan_count: usize,
    per_result_neighbor_lookup_count: usize,
    persistent_artifact_bypass_count: usize,
    materialized_row_count: usize,
}

impl ReplayExecutionCounterEnvelope {
    fn from_consumption(
        consumption: &crate::runtime::WorthQueryGraphReadAccessPlanConsumption,
    ) -> Self {
        let counters = consumption.execution_counters();
        Self {
            executor_entry_count: counters.executor_entry_count(),
            strategy_recompute_count: counters.strategy_recompute_count(),
            ephemeral_index_allocation_count: counters.ephemeral_index_allocation_count(),
            edge_scan_count: counters.edge_scan_count(),
            per_result_neighbor_lookup_count: counters.per_result_neighbor_lookup_count(),
            persistent_artifact_bypass_count: counters.persistent_artifact_bypass_count(),
            materialized_row_count: counters.materialized_row_count(),
        }
    }
}

fn workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> crate::runtime::WorthQueryWorkspace {
    PublicBridgeRuntimeHarness::new()
        .bridge_backed_runtime(request_execution)
        .workspace(name)
        .expect("closeout replay workspace should open")
}
