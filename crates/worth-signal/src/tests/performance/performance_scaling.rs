use super::performance_workload_support::{
    build_chain_graph, evaluate_chain, evaluate_chain_bulk, evaluate_chain_bulk_profile,
    median_u128, warm_up_bulk_evaluation_path, with_perf_topology_asserts_disabled,
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
#[ignore = "machine-sensitive structured performance profile"]
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
#[ignore = "machine-sensitive structured performance profile"]
fn push_build_perf_10k_nodes() {
    let (elapsed, chain_len) = with_perf_topology_asserts_disabled(|| {
        let start = Instant::now();
        let (_graph, chain) = build_chain_graph(10_000);
        (start.elapsed(), chain.len())
    });

    let max_build_ms: u128 = 1_500;
    assert_eq!(chain_len, 10_000);
    assert!(
        elapsed.as_millis() < max_build_ms,
        "10k-node graph build took {}ms, expected < {}ms",
        elapsed.as_millis(),
        max_build_ms
    );
}

#[test]
#[ignore = "diagnostic-only ad hoc profile; authoritative 10k bootstrap cert lives in performance_profiles"]
fn push_initial_eval_perf_10k_nodes() {
    warm_up_bulk_evaluation_path();
    let mut elapsed_samples = Vec::with_capacity(5);
    let mut plan_samples = Vec::with_capacity(5);
    let mut execute_samples = Vec::with_capacity(5);
    let mut stage_exec_samples = Vec::with_capacity(5);
    let mut plans_built = 0;
    let mut tasks_scheduled = 0;

    for _ in 0..5 {
        let (mut graph, chain) = with_perf_topology_asserts_disabled(|| build_chain_graph(10_000));
        let (plan_ms, execute_ms, sample_plans_built, sample_tasks_scheduled, stage_exec_ms) =
            evaluate_chain_bulk_profile(&mut graph, &chain);
        elapsed_samples.push(plan_ms + execute_ms);
        plan_samples.push(plan_ms);
        execute_samples.push(execute_ms);
        stage_exec_samples.push(stage_exec_ms);
        plans_built = sample_plans_built;
        tasks_scheduled = sample_tasks_scheduled;
    }

    let elapsed_ms = median_u128(&mut elapsed_samples);
    let plan_ms = median_u128(&mut plan_samples);
    let execute_ms = median_u128(&mut execute_samples);
    let stage_exec_ms = median_u128(&mut stage_exec_samples);

    eprintln!(
        "Initial 10k-node evaluation median={}ms plan={}ms execute={}ms plans_built={} tasks_scheduled={} stage_exec={}ms samples={:?}",
        elapsed_ms,
        plan_ms,
        execute_ms,
        plans_built,
        tasks_scheduled,
        stage_exec_ms,
        elapsed_samples,
    );
}

#[test]
#[ignore = "diagnostic comparison for 10k bootstrap path shape"]
fn push_initial_eval_compare_10k_nodes() {
    let count = 10_000;
    let (single_elapsed_ms, single_plans_built, single_tasks_scheduled, single_stage_exec_ms) =
        with_perf_topology_asserts_disabled(|| {
            let (mut graph, chain) = build_chain_graph(count);
            graph.reset_telemetry();
            let start = Instant::now();
            evaluate_chain(&mut graph, &chain);
            let elapsed = start.elapsed().as_millis();
            let telemetry = *graph.telemetry();
            (
                elapsed,
                telemetry.planner.plans_built,
                telemetry.planner.tasks_scheduled,
                telemetry.execution.stage_execution_nanos / 1_000_000,
            )
        });

    let (bulk_elapsed_ms, bulk_plans_built, bulk_tasks_scheduled, bulk_stage_exec_ms) =
        with_perf_topology_asserts_disabled(|| {
            let (mut graph, chain) = build_chain_graph(count);
            let (plan_ms, execute_ms, plans_built, tasks_scheduled, stage_exec_ms) =
                evaluate_chain_bulk_profile(&mut graph, &chain);
            (
                plan_ms + execute_ms,
                plans_built,
                tasks_scheduled,
                stage_exec_ms,
            )
        });

    eprintln!(
        "single_node_eval: elapsed_ms={single_elapsed_ms} plans_built={single_plans_built} tasks_scheduled={single_tasks_scheduled} stage_exec_ms={single_stage_exec_ms}"
    );
    eprintln!(
        "bulk_eval: elapsed_ms={bulk_elapsed_ms} plans_built={bulk_plans_built} tasks_scheduled={bulk_tasks_scheduled} stage_exec_ms={bulk_stage_exec_ms}"
    );
}

#[test]
#[ignore = "diagnostic comparison for repeated 10k bulk bootstrap in one process"]
fn push_initial_eval_bulk_twice_10k_nodes() {
    let count = 10_000;
    let (first_elapsed_ms, first_plans_built, first_tasks_scheduled, first_stage_exec_ms) =
        with_perf_topology_asserts_disabled(|| {
            let (mut graph, chain) = build_chain_graph(count);
            let (plan_ms, execute_ms, plans_built, tasks_scheduled, stage_exec_ms) =
                evaluate_chain_bulk_profile(&mut graph, &chain);
            (
                plan_ms + execute_ms,
                plans_built,
                tasks_scheduled,
                stage_exec_ms,
            )
        });

    let (second_elapsed_ms, second_plans_built, second_tasks_scheduled, second_stage_exec_ms) =
        with_perf_topology_asserts_disabled(|| {
            let (mut graph, chain) = build_chain_graph(count);
            let (plan_ms, execute_ms, plans_built, tasks_scheduled, stage_exec_ms) =
                evaluate_chain_bulk_profile(&mut graph, &chain);
            (
                plan_ms + execute_ms,
                plans_built,
                tasks_scheduled,
                stage_exec_ms,
            )
        });

    eprintln!(
        "bulk_first: elapsed_ms={first_elapsed_ms} plans_built={first_plans_built} tasks_scheduled={first_tasks_scheduled} stage_exec_ms={first_stage_exec_ms}"
    );
    eprintln!(
        "bulk_second: elapsed_ms={second_elapsed_ms} plans_built={second_plans_built} tasks_scheduled={second_tasks_scheduled} stage_exec_ms={second_stage_exec_ms}"
    );
}

#[test]
#[ignore = "machine-sensitive defer budget; tracked in performance_profiles baseline suite"]
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
