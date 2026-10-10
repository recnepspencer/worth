use super::executor_policy::{aggressive_parallel_runtime_policy, bounded_contract};
use crate::facade::{mark_dirty, EvaluationRequestMode, SignalGraph};
use crate::tests::leased_execution::support::{authority, request};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A};

#[test]
fn grouped_parallel_publishes_output_commits_in_global_task_order() {
    let mut baseline = SignalGraph::new();
    baseline.set_runtime_policy(aggressive_parallel_runtime_policy());
    let producers = (0..4)
        .map(|_| {
            baseline
                .node()
                .with_contract(bounded_contract(&[]))
                .produces_aspects(ASPECT_A)
                .build()
        })
        .collect::<Vec<_>>();
    let left_consumer = baseline
        .node()
        .with_contract(bounded_contract(&producers[..2]))
        .reads_aspects(ASPECT_A)
        .build();
    let right_consumer = baseline
        .node()
        .with_contract(bounded_contract(&producers[2..]))
        .reads_aspects(ASPECT_A)
        .build();
    for &producer in &producers[..2] {
        baseline
            .append_dependency(left_consumer, producer, ASPECT_A)
            .unwrap();
    }
    for &producer in &producers[2..] {
        baseline
            .append_dependency(right_consumer, producer, ASPECT_A)
            .unwrap();
    }
    let requested = producers
        .iter()
        .copied()
        .chain([left_consumer, right_consumer])
        .collect::<Vec<_>>();
    let bootstrap = baseline
        .build_evaluation_plan(&requested, EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    baseline
        .execute_prepared_plan(&bootstrap, &(), &|ctx| {
            Ok(ctx.finish(version_ab(ctx.node().index() as u64 + 1, 0)))
        })
        .unwrap();

    let run = |mut graph: SignalGraph, workers: usize| {
        let before_commit_count = graph.published_output_commit_order_for_test().len();
        let before_reductions = graph.telemetry().execution.reduction_group_count;
        for &producer in &producers {
            mark_dirty(&mut graph, producer, ASPECT_A).unwrap();
        }
        let plan = graph
            .build_evaluation_plan(&producers, EvaluationRequestMode::Default)
            .unwrap();
        let lease = authority()
            .request_lease(request(workers, 1_000_000))
            .unwrap();
        graph
            .execute_prepared_plan_checked(
                &plan,
                &(),
                &|ctx| Ok(ctx.finish(version_ab(ctx.node().index() as u64 + 100, 0))),
                worth_execution::ExecutionRequest::leased(&lease),
            )
            .unwrap();
        let order = graph.published_output_commit_order_for_test();
        (
            order[before_commit_count..].to_vec(),
            graph.telemetry().execution.reduction_group_count - before_reductions,
        )
    };

    let serial = run(baseline.clone(), 1);
    let parallel = run(baseline, 4);
    assert_eq!(serial.0, parallel.0);
    assert!(
        parallel.1 > 0,
        "the grouped-parallel reduction must execute"
    );
    assert_eq!(
        parallel.0.iter().map(|(_, node)| *node).collect::<Vec<_>>(),
        producers
    );
}
