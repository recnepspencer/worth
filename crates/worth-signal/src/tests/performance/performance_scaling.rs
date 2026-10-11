use super::performance_workload_support::{
    build_chain_graph, evaluate_chain_bulk, with_perf_topology_asserts_disabled,
};
use crate::facade::{
    mark_dirty, EvaluationCondition, EvaluationRequestMode, NodeId, NodeState, SignalGraph,
    SignalRuntime,
};
use crate::tests::support::{evaluate, version_ab, GraphDependencyBatchExt, ASPECT_B};
use std::time::Instant;

#[test]
fn chain_1000_source_seed_does_not_walk_the_subscriber_closure() {
    let mut graph = SignalGraph::new();
    let mut chain: Vec<NodeId> = Vec::with_capacity(1000);

    let first = graph.create_node();
    chain.push(first);

    for i in 1..1000 {
        let node = graph.create_node();
        graph
            .append_dependency(node, chain[i - 1], ASPECT_B)
            .unwrap();
        chain.push(node);
    }

    let mut compute = |_id, _g: &SignalGraph| Ok(version_ab(0, 1));
    for node in &chain {
        evaluate(&mut graph, *node, &mut compute).unwrap();
    }

    mark_dirty(&mut graph, chain[0], ASPECT_B).unwrap();

    let state_first = graph.get_state(chain[0]).unwrap();
    assert_eq!(state_first, NodeState::Dirty);

    let state_second = graph.get_state(chain[1]).unwrap();
    assert_eq!(state_second, NodeState::Clean);

    let state_last = graph.get_state(chain[999]).unwrap();
    assert_eq!(state_last, NodeState::Clean);
}

#[test]
fn transaction_chain_1000_source_seed_stages_only_the_source() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let (mut graph, chain) = with_perf_topology_asserts_disabled(|| build_chain_graph(1_000));
    evaluate_chain_bulk(&mut graph, &chain);
    let mut runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();

    runtime
        .transaction(request_execution, &mut (), |tx| {
            assert!(!tx.has_config_rollback_baseline_for_test());
            tx.mark_changed(chain[0], ASPECT_B)?;
            assert!(!tx.has_config_rollback_baseline_for_test());
            Ok(())
        })
        .unwrap();

    let metrics = runtime.observe().metrics();
    assert_eq!(
        metrics.transaction.transaction_mark_dirty_candidate_visits,
        1
    );
    assert_eq!(
        runtime.graph().get_state(chain[0]).unwrap(),
        NodeState::Dirty
    );
    assert_eq!(
        runtime.graph().get_state(chain[1]).unwrap(),
        NodeState::Clean
    );
    assert_eq!(
        runtime.graph().get_state(chain[999]).unwrap(),
        NodeState::Clean
    );
}

#[test]
fn push_perf_10k_nodes() {
    let (mut graph, chain) = with_perf_topology_asserts_disabled(|| build_chain_graph(10_000));
    evaluate_chain_bulk(&mut graph, &chain);

    let start = Instant::now();
    mark_dirty(&mut graph, chain[0], ASPECT_B).unwrap();
    let elapsed = start.elapsed();

    let max_push_ms: u128 = 500;
    assert!(
        elapsed.as_millis() < max_push_ms,
        "Push propagation took {}ms, expected < {}ms",
        elapsed.as_millis(),
        max_push_ms
    );
}

#[test]
#[ignore = "requires a quiet timing run; debug trial exceeded the 500 ms defer budget"]
fn ondemand_defer_perf_10k_nodes() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let mut nodes: Vec<NodeId> = Vec::with_capacity(10_000);
    {
        let mut graph = runtime.graph_mut();
        for _ in 0..10_000 {
            nodes.push(
                graph
                    .node()
                    .condition(EvaluationCondition::OnDemand)
                    .build(),
            );
        }
    }

    let start = std::time::Instant::now();
    let plan = runtime
        .build_evaluation_plan(&nodes, EvaluationRequestMode::Default)
        .unwrap();
    assert_eq!(plan.summary.task_count, 10_000);
    runtime
        .execute_prepared_plan(&plan, &(), &|_ctx| Ok(version_ab(0, 1)))
        .unwrap();
    let elapsed = start.elapsed();

    let max_eval_ms: u128 = 500;
    assert!(
        elapsed.as_millis() < max_eval_ms,
        "On-demand defer path took {}ms, expected < {}ms",
        elapsed.as_millis(),
        max_eval_ms
    );
}
