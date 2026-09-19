use crate::data::retained_storage::{
    btree_structure_charge, RetainedStorageMeasurement, RetainedStoragePreparation as Work,
};
use crate::diagnostics::flow::FlowSummary;
use crate::diagnostics::profile::DiagnosticsTier;
use crate::diagnostics::summary::GraphSummary;
use crate::facade::{EvaluationContext, EvaluationRequestMode, SignalGraph, SignalRuntimePolicy};
use crate::logic::planner::{StageExecutionOutcome, TaskExecutionOutcome, TaskReason};
use crate::tests::support::version_ab;

fn summaries() -> (FlowSummary, GraphSummary) {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::forensic());
    let node = graph.node().build();
    let compute = |ctx: &mut EvaluationContext<'_, ()>| Ok(ctx.finish(version_ab(7, 0)));
    let plan = graph
        .build_evaluation_plan(&[node], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    graph.execute_prepared_plan(&plan, &(), &compute).unwrap();
    let flow = graph
        .observe()
        .latest_flow_diagnostics()
        .unwrap()
        .to_owned_summary();
    let summary = graph
        .observe()
        .diagnostics_summary(DiagnosticsTier::Forensic);
    (flow, summary)
}
#[test]
fn diagnostic_count_maps_and_plan_widths_match_their_retained_representations() {
    let (flow, _) = summaries();
    let report = flow.apply.report;
    assert!(!report.task_outcome_counts.is_empty());
    assert!(!report.stage_outcome_counts.is_empty());
    let expected =
        btree_structure_charge::<TaskExecutionOutcome, u32>(report.task_outcome_counts.len())
            .unwrap()
            .checked_add(
                btree_structure_charge::<StageExecutionOutcome, u32>(
                    report.stage_outcome_counts.len(),
                )
                .unwrap(),
            )
            .unwrap();
    assert_eq!(
        report.retained_heap_charge(&mut Work::new(10000)).unwrap(),
        expected
    );
    let plan = flow.planning.plan;
    assert!(!plan.task_reason_counts.is_empty());
    let expected = btree_structure_charge::<TaskReason, u32>(plan.task_reason_counts.len())
        .unwrap()
        .bytes()
        + (plan.stage_widths.capacity() * std::mem::size_of::<u32>()) as u64;
    assert_eq!(
        plan.retained_heap_charge(&mut Work::new(10000))
            .unwrap()
            .bytes(),
        expected
    );
}
#[test]
fn diagnostic_graph_samples_charge_capacity_while_metrics_remain_inline() {
    let (_, mut graph) = summaries();
    let expected = (graph.sample_dirty_nodes.capacity()
        + graph.sample_nodes_with_execution_record.capacity())
        * std::mem::size_of::<crate::data::handle::NodeId>();
    assert_eq!(
        graph
            .retained_heap_charge(&mut Work::new(10000))
            .unwrap()
            .bytes(),
        expected as u64
    );
    let original = graph.clone();
    let before = graph.retained_heap_charge(&mut Work::new(10000)).unwrap();
    let old_capacity = graph.sample_dirty_nodes.capacity();
    graph.sample_dirty_nodes.reserve_exact(128);
    let delta = (graph.sample_dirty_nodes.capacity() - old_capacity)
        * std::mem::size_of::<crate::data::handle::NodeId>();
    assert_eq!(
        graph
            .retained_heap_charge(&mut Work::new(10000))
            .unwrap()
            .bytes()
            - before.bytes(),
        delta as u64
    );
    assert_eq!(graph, original);
}
