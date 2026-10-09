use super::schema_registry_scenarios::conflict_isolation_merge_schema_registry;
use crate::facade::{
    ConflictIsolationGranularity, ConflictIsolationSelectionBasis, NodeEvaluationResult,
    SignalGraph, SignalRuntime,
};
use crate::tests::support::{version_ab, ASPECT_A, ASPECT_B};

#[test]
fn runtime_merge_conflict_isolation_selection_flows_into_execution_counters() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let graph = SignalGraph::new().with_schema_registry(conflict_isolation_merge_schema_registry(
        Some("signal.conflict-isolation.per-node"),
    ));
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let shared = runtime
        .graph_mut()
        .node()
        .schema_name("signal.demo.conflict-isolation-owned")
        .expect("known schema")
        .produces_aspects([ASPECT_A, ASPECT_B])
        .build();
    let mut runtime_ctx = ();

    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(431, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let main = runtime.observe().current_branch();
    let feature = runtime
        .create_branch("feature-conflict-isolation-execution-counters")
        .unwrap();

    runtime.switch_branch(feature.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(432, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    runtime.switch_branch(main.clone()).unwrap();
    runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(shared, ASPECT_A)?;
            tx.read(shared, &|view| {
                Ok(view.finish(NodeEvaluationResult::from_version(version_ab(433, 0))))
            })?;
            Ok(())
        })
        .unwrap();

    let result = runtime
        .merge_raw()
        .from(feature)
        .into(main)
        .conflict_isolation_policy_named("signal.conflict-isolation.per-aspect")
        .run()
        .unwrap();

    assert_eq!(
        result.selected_conflict_isolation_name.as_str(),
        "signal.conflict-isolation.per-aspect"
    );
    assert_eq!(
        result.selected_conflict_isolation_basis,
        ConflictIsolationSelectionBasis::RequestNamed
    );
    assert_eq!(result.conflict_isolation_plan.records.len(), 1);
    assert_eq!(
        result.conflict_isolation_plan.records[0].granularity,
        ConflictIsolationGranularity::PerAspect
    );
    assert_eq!(
        result.conflict_isolation_plan.records[0].isolated_aspects,
        vec![ASPECT_A, ASPECT_B]
    );
    assert_eq!(result.counters.conflict_isolation_record_count, 1);
    assert_eq!(result.counters.conflict_isolation_expansion_breadth, 0);
}
