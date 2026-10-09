use crate::facade::{
    mark_dirty, mark_dirty_with_regions, AspectVersion, ChangedRegion, EvaluationRequestMode,
    NodeEvaluationResult, NodeId, SignalGraph,
};
use crate::tests::leased_execution::support::{authority, request};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_A};

use super::canonical_artifact_oracle::canonical_runtime_artifacts;
use super::executor_policy::{bounded_contract, worker_matrix};

#[test]
fn full_parallel_policy_matrix_preserves_semantic_artifacts_on_tolerance_heavy_partition_graph() {
    fn build_graph() -> (SignalGraph, NodeId, NodeId, NodeId) {
        let mut graph = SignalGraph::new();
        let source = graph.node().with_contract(bounded_contract(&[])).build();
        let branch_a = graph
            .node()
            .with_contract(bounded_contract(&[source]))
            .tolerance(1)
            .build();
        let branch_b = graph
            .node()
            .with_contract(bounded_contract(&[source]))
            .tolerance(1)
            .build();
        graph
            .append_partition_dependency(branch_a, source, ASPECT_A, "shell")
            .unwrap();
        graph
            .append_partition_dependency(branch_b, source, ASPECT_A, "core")
            .unwrap();
        (graph, source, branch_a, branch_b)
    }

    fn bootstrap(graph: &mut SignalGraph, source: NodeId, branch_a: NodeId, branch_b: NodeId) {
        let plan = graph
            .build_evaluation_plan(
                &[source, branch_a, branch_b],
                EvaluationRequestMode::ForceOnDemand,
            )
            .unwrap();
        graph
            .execute_prepared_plan(&plan, &(), &move |ctx| {
                let node = ctx.node();
                let result = if node == source {
                    ctx.finish(
                        NodeEvaluationResult::from_version(version_ab(10, 0))
                            .with_changed_region(ChangedRegion::new("shell"))
                            .with_changed_region(ChangedRegion::new("core")),
                    )
                } else {
                    let version = ctx.read_aspect_version(source, ASPECT_A)?;
                    ctx.finish(NodeEvaluationResult::from_version(version))
                };
                Ok(result)
            })
            .unwrap();
    }

    fn run_with_lease(
        mut graph: SignalGraph,
        source: NodeId,
        target: NodeId,
        workers: usize,
    ) -> serde_json::Value {
        mark_dirty_with_regions(
            &mut graph,
            source,
            ASPECT_A,
            &[ChangedRegion::new("core"), ChangedRegion::new("shell")],
        )
        .unwrap();
        let plan = graph
            .build_evaluation_plan(&[target], EvaluationRequestMode::Default)
            .unwrap();
        let lease = authority()
            .request_lease(request(workers, 1_000_000))
            .unwrap();
        graph
            .execute_prepared_plan_checked(
                &plan,
                &(),
                &move |ctx| {
                    let node = ctx.node();
                    let result = if node == source {
                        ctx.finish(
                            NodeEvaluationResult::from_version(version_ab(12, 0))
                                .with_changed_region(ChangedRegion::new("shell"))
                                .with_changed_region(ChangedRegion::new("core")),
                        )
                    } else {
                        let version = version_ab(ctx.read(source, ASPECT_A)?, 0);
                        ctx.finish(NodeEvaluationResult::from_version(version))
                    };
                    Ok(result)
                },
                worth_execution::ExecutionRequest::leased(&lease),
            )
            .unwrap();
        canonical_runtime_artifacts(&graph, target)
    }

    let (base_graph, source, branch_a, branch_b) = build_graph();
    let mut seed_graph = base_graph.clone();
    bootstrap(&mut seed_graph, source, branch_a, branch_b);
    let baseline = run_with_lease(seed_graph.clone(), source, branch_b, 1);

    for (label, workers) in worker_matrix() {
        let observed = run_with_lease(seed_graph.clone(), source, branch_b, workers);
        assert_eq!(
            baseline, observed,
            "lease {label} drifted semantic artifacts"
        );
    }
}
#[test]
fn repeated_executor_policy_churn_keeps_tolerance_boundary_artifacts_stable() {
    fn build_graph() -> (SignalGraph, NodeId, NodeId, NodeId, NodeId) {
        let mut graph = SignalGraph::new();
        let source = graph.node().with_contract(bounded_contract(&[])).build();
        let shell = graph
            .node()
            .with_contract(bounded_contract(&[source]))
            .tolerance(2)
            .build();
        let core = graph
            .node()
            .with_contract(bounded_contract(&[source]))
            .tolerance(2)
            .build();
        let target = graph
            .node()
            .with_contract(bounded_contract(&[shell, core]))
            .output_identity()
            .build();
        graph.append_dependency(shell, source, ASPECT_A).unwrap();
        graph.append_dependency(core, source, ASPECT_A).unwrap();
        graph.append_dependency(target, shell, ASPECT_A).unwrap();
        graph.append_dependency(target, core, ASPECT_A).unwrap();
        (graph, source, shell, core, target)
    }

    fn bootstrap(
        graph: &mut SignalGraph,
        source: NodeId,
        shell: NodeId,
        core: NodeId,
        target: NodeId,
    ) {
        let plan = graph
            .build_evaluation_plan(
                &[source, shell, core, target],
                EvaluationRequestMode::ForceOnDemand,
            )
            .unwrap();
        graph
            .execute_prepared_plan(&plan, &(), &move |ctx| {
                let node = ctx.node();
                let result = if node == source {
                    ctx.finish(NodeEvaluationResult::from_version(version_ab(100, 0)))
                } else if node == shell || node == core {
                    let version = ctx.read_aspect_version(source, ASPECT_A)?;
                    ctx.finish(NodeEvaluationResult::from_version(version))
                } else {
                    let shell_v = ctx.read_aspect_version(shell, ASPECT_A)?;
                    let core_v = ctx.read_aspect_version(core, ASPECT_A)?;
                    ctx.finish(
                        NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                            ASPECT_A,
                            shell_v.get(ASPECT_A) + core_v.get(ASPECT_A),
                        )]))
                        .with_output_identity("topology-target"),
                    )
                };
                Ok(result)
            })
            .unwrap();
    }

    fn run_once(
        graph: &mut SignalGraph,
        source: NodeId,
        shell: NodeId,
        core: NodeId,
        target: NodeId,
        next_version: u64,
        workers: usize,
    ) -> serde_json::Value {
        mark_dirty(&mut *graph, source, ASPECT_A).unwrap();
        let plan = graph
            .build_evaluation_plan(&[target], EvaluationRequestMode::Default)
            .unwrap();
        let lease = authority()
            .request_lease(request(workers, 1_000_000))
            .unwrap();
        graph
            .execute_prepared_plan_checked(
                &plan,
                &(),
                &move |ctx| {
                    let node = ctx.node();
                    let result = if node == source {
                        ctx.finish(NodeEvaluationResult::from_version(version_ab(
                            next_version,
                            0,
                        )))
                    } else if node == shell || node == core {
                        let version = version_ab(ctx.read(source, ASPECT_A)?, 0);
                        ctx.finish(NodeEvaluationResult::from_version(version))
                    } else {
                        let shell_v = version_ab(ctx.read(shell, ASPECT_A)?, 0);
                        let core_v = version_ab(ctx.read(core, ASPECT_A)?, 0);
                        ctx.finish(
                            NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                                ASPECT_A,
                                shell_v.get(ASPECT_A) + core_v.get(ASPECT_A),
                            )]))
                            .with_output_identity("topology-target"),
                        )
                    };
                    Ok(result)
                },
                worth_execution::ExecutionRequest::leased(&lease),
            )
            .unwrap();
        canonical_runtime_artifacts(graph, target)
    }

    let (mut seed_graph, source, shell, core, target) = build_graph();
    bootstrap(&mut seed_graph, source, shell, core, target);

    for next_version in [101_u64, 102, 101, 102] {
        let mut baseline_graph = seed_graph.clone();
        let baseline = run_once(
            &mut baseline_graph,
            source,
            shell,
            core,
            target,
            next_version,
            1,
        );
        for (label, workers) in worker_matrix() {
            let mut candidate_graph = seed_graph.clone();
            let observed = run_once(
                &mut candidate_graph,
                source,
                shell,
                core,
                target,
                next_version,
                workers,
            );
            assert_eq!(
                baseline, observed,
                "lease {label} drifted at tolerance boundary {next_version}"
            );
        }
    }
}
