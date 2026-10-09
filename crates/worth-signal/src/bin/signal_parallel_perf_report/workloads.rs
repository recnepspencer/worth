use super::{leased_signal_support, summarize, version_ab, PerfRecord, ASPECT_A};
use std::time::Instant;
use worth_signal::facade::runtime::{mark_dirty_batch, RuntimePolicy};
use worth_signal::facade::specialist::RunMode;
use worth_signal::facade::{
    AspectVersion, BatchChange, ChangedRegion, DependencyEdge, NodeEvaluationResult, SignalGraph,
    SignalRuntime,
};

pub(super) fn run_deep_chain(
    executor_profile: &'static str,
    runtime_policy_name: &'static str,
    runtime_policy: RuntimePolicy,
    workers: usize,
    host: &worth_execution::ExecutionAuthority,
) -> PerfRecord {
    let mut runtime = SignalRuntime::<(), (), (), (), ()>::builder(SignalGraph::new())
        .with_kernel_defaults()
        .runtime_policy(runtime_policy)
        .build();
    let mut graph = runtime.graph_mut();
    let mut chain = Vec::new();
    for _ in 0..512 {
        let inputs = chain.last().copied().into_iter().collect::<Vec<_>>();
        chain.push(
            graph
                .node()
                .with_contract(leased_signal_support::contract(&inputs, ASPECT_A))
                .build(),
        );
    }
    for index in 1..chain.len() {
        graph
            .set_dependencies(
                chain[index],
                [DependencyEdge::new(chain[index - 1], ASPECT_A)],
            )
            .unwrap();
    }

    let bootstrap = graph
        .build_evaluation_plan(&chain, RunMode::ForceOnDemand)
        .unwrap();
    graph
        .execute_prepared_plan(&bootstrap, &(), &|ctx| {
            let position = chain.iter().position(|&node| node == ctx.node()).unwrap();
            let value = if position == 0 {
                1
            } else {
                ctx.read_aspect_version(chain[position - 1], ASPECT_A)?
                    .get(ASPECT_A)
            };
            Ok(ctx.finish(version_ab(value, 0)))
        })
        .unwrap();

    mark_dirty_batch(
        &mut *graph,
        &BatchChange::from_sources([(chain[0], ASPECT_A)]),
    )
    .unwrap();
    let plan_start = Instant::now();
    let plan = graph
        .build_evaluation_plan(&chain, RunMode::Default)
        .unwrap();
    let planning_nanos = plan_start.elapsed().as_nanos();
    let execute_start = Instant::now();
    let lease = leased_signal_support::lease(host, workers);
    let report = graph
        .execute_prepared_plan_checked(
            &plan,
            &(),
            &|ctx| {
                ctx.work().checkpoint(chain.len() as u64).map_err(|_| {
                    worth_signal::facade::SignalError::invalid_input("chain lookup stopped")
                })?;
                let position = chain.iter().position(|&node| node == ctx.node()).unwrap();
                let value = if position == 0 {
                    2
                } else {
                    ctx.read(chain[position - 1], ASPECT_A)?
                };
                Ok(version_ab(value, 0))
            },
            worth_execution::ExecutionRequest::leased(&lease),
        )
        .unwrap();
    summarize(
        "deep-chain-512",
        executor_profile,
        runtime_policy_name,
        planning_nanos,
        execute_start.elapsed().as_nanos(),
        &report,
    )
}

pub(super) fn run_wide_stage(
    executor_profile: &'static str,
    runtime_policy_name: &'static str,
    runtime_policy: RuntimePolicy,
    workers: usize,
    host: &worth_execution::ExecutionAuthority,
) -> PerfRecord {
    let mut runtime = SignalRuntime::<(), (), (), (), ()>::builder(SignalGraph::new())
        .with_kernel_defaults()
        .runtime_policy(runtime_policy)
        .build();
    let mut graph = runtime.graph_mut();
    let requested: Vec<_> = (0..256)
        .map(|_| {
            graph
                .node()
                .with_contract(leased_signal_support::contract(&[], ASPECT_A))
                .build()
        })
        .collect();
    let bootstrap = graph
        .build_evaluation_plan(&requested, RunMode::ForceOnDemand)
        .unwrap();
    graph
        .execute_prepared_plan(&bootstrap, &(), &|ctx| {
            Ok(ctx.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
        })
        .unwrap();

    mark_dirty_batch(
        &mut *graph,
        &BatchChange::from_sources(requested.iter().copied().map(|node| (node, ASPECT_A))),
    )
    .unwrap();
    let plan_start = Instant::now();
    let plan = graph
        .build_evaluation_plan(&requested, RunMode::Default)
        .unwrap();
    let planning_nanos = plan_start.elapsed().as_nanos();
    let execute_start = Instant::now();
    let lease = leased_signal_support::lease(host, workers);
    let report = graph
        .execute_prepared_plan_checked(
            &plan,
            &(),
            &|ctx| Ok(ctx.finish(NodeEvaluationResult::from_version(version_ab(2, 0)))),
            worth_execution::ExecutionRequest::leased(&lease),
        )
        .unwrap();
    summarize(
        "wide-stage-256",
        executor_profile,
        runtime_policy_name,
        planning_nanos,
        execute_start.elapsed().as_nanos(),
        &report,
    )
}

pub(super) fn run_partition_tolerance(
    executor_profile: &'static str,
    runtime_policy_name: &'static str,
    runtime_policy: RuntimePolicy,
    workers: usize,
    host: &worth_execution::ExecutionAuthority,
) -> PerfRecord {
    let mut runtime = SignalRuntime::<(), (), (), (), ()>::builder(SignalGraph::new())
        .with_kernel_defaults()
        .runtime_policy(runtime_policy)
        .build();
    let mut graph = runtime.graph_mut();
    let source = graph
        .node()
        .with_contract(leased_signal_support::contract(&[], ASPECT_A))
        .build();
    let branches: Vec<_> = (0..96)
        .map(|_| {
            graph
                .node()
                .with_contract(leased_signal_support::contract(&[source], ASPECT_A))
                .tolerance(1)
                .build()
        })
        .collect();
    let target = graph
        .node()
        .with_contract(leased_signal_support::contract(&branches, ASPECT_A))
        .output_identity()
        .build();
    for (index, branch) in branches.iter().copied().enumerate() {
        let partition = if index % 2 == 0 { "shell" } else { "core" };
        graph
            .set_dependencies(
                branch,
                [DependencyEdge::whole_partition(source, ASPECT_A, partition)],
            )
            .unwrap();
        graph
            .set_dependencies(target, [DependencyEdge::new(branch, ASPECT_A)])
            .unwrap();
    }

    let bootstrap_targets: Vec<_> = std::iter::once(source)
        .chain(branches.iter().copied())
        .chain(std::iter::once(target))
        .collect();
    let bootstrap = graph
        .build_evaluation_plan(&bootstrap_targets, RunMode::ForceOnDemand)
        .unwrap();
    let bootstrap_branches = branches.clone();
    graph
        .execute_prepared_plan(&bootstrap, &(), &move |ctx| {
            let node = ctx.node();
            let result = if node == source {
                ctx.finish(
                    NodeEvaluationResult::from_version(version_ab(10, 0))
                        .with_changed_region(ChangedRegion::new("shell"))
                        .with_changed_region(ChangedRegion::new("core")),
                )
            } else if bootstrap_branches.contains(&node) {
                let version = ctx.read_aspect_version(source, ASPECT_A)?;
                ctx.finish(NodeEvaluationResult::from_version(version))
            } else {
                let mut total = 0_u64;
                for branch in &bootstrap_branches {
                    total += ctx.read_aspect_version(*branch, ASPECT_A)?.get(ASPECT_A);
                }
                ctx.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                        ASPECT_A, total,
                    )]))
                    .with_output_identity("partition-aggregate"),
                )
            };
            Ok(result)
        })
        .unwrap();

    mark_dirty_batch(
        &mut *graph,
        &BatchChange::singleton(
            source,
            ASPECT_A,
            vec![ChangedRegion::new("core"), ChangedRegion::new("shell")],
        ),
    )
    .unwrap();
    let plan_start = Instant::now();
    let plan = graph
        .build_evaluation_plan(&[target], RunMode::Default)
        .unwrap();
    let planning_nanos = plan_start.elapsed().as_nanos();
    let execute_start = Instant::now();
    let execute_branches = branches.clone();
    let lease = leased_signal_support::lease(host, workers);
    let report = graph
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
                } else if execute_branches.contains(&node) {
                    let version = version_ab(ctx.read(source, ASPECT_A)?, 0);
                    ctx.finish(NodeEvaluationResult::from_version(version))
                } else {
                    let mut total = 0_u64;
                    for branch in &execute_branches {
                        total += version_ab(ctx.read(*branch, ASPECT_A)?, 0).get(ASPECT_A);
                    }
                    ctx.finish(
                        NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                            ASPECT_A, total,
                        )]))
                        .with_output_identity("partition-aggregate"),
                    )
                };
                Ok(result)
            },
            worth_execution::ExecutionRequest::leased(&lease),
        )
        .unwrap();
    summarize(
        "partition-tolerance-96",
        executor_profile,
        runtime_policy_name,
        planning_nanos,
        execute_start.elapsed().as_nanos(),
        &report,
    )
}
