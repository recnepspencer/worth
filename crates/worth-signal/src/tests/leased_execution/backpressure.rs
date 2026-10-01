use std::sync::atomic::{AtomicUsize, Ordering};

use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};

use crate::facade::{
    Aspect, AspectVersion, BoundedSignalInputs, DeclaredSignalInput, EvaluationRequestMode,
    NodeContract, SignalError, SignalGraph,
};

use super::support::{authority, request};

const VALUE: Aspect = Aspect::new(0);
const SOURCE_COUNT: usize = 32;
const TARGET_COUNT: usize = 8;

struct Trial {
    stages: usize,
    parallel_stages: usize,
    charged_work: u64,
    values: Vec<u64>,
    performed_stage_bindings: Vec<(usize, u64, u32, u32)>,
}

fn graph_with_settled_sources(
    target_count: usize,
) -> (
    SignalGraph,
    Vec<crate::facade::NodeId>,
    Vec<crate::facade::NodeId>,
) {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(
        crate::facade::SignalRuntimePolicy::forensic().with_parallel_admission(
            crate::facade::ParallelAdmissionPolicy {
                throughput_min_parallel_tasks: 1,
                balanced_min_parallel_tasks: 1,
                latency_bounded_min_parallel_tasks: 1,
                full_parallel_min_tasks: 1,
            },
        ),
    );
    let sources = (0..SOURCE_COUNT)
        .map(|_| {
            graph
                .node()
                .with_contract(
                    NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()),
                )
                .build()
        })
        .collect::<Vec<_>>();
    // Source settlement is fixture setup. Its work ceiling is separate from
    // the deliberately constrained target request exercised below.
    let bootstrap = authority().request_lease(request(4, 10_000_000)).unwrap();
    graph
        .evaluate_checked(
            &sources,
            EvaluationRequestMode::Default,
            &(),
            &|_| Ok(AspectVersion::zero().with(VALUE, 1)),
            &bootstrap,
        )
        .expect("source facts settle before the backpressure request");
    drop(bootstrap);
    let targets = (0..target_count)
        .map(|_| {
            graph
                .node()
                .with_contract(
                    NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::new(
                        sources
                            .iter()
                            .copied()
                            .map(|source| DeclaredSignalInput::new(source, VALUE)),
                    )),
                )
                .build()
        })
        .collect();
    (graph, sources, targets)
}

fn memory_request(workers: usize, memory_bytes: u64) -> worth_execution::LeaseRequest {
    // This fixture varies memory. Its separate work allowance covers the
    // measured multi-task checked read path at every admitted width.
    let mut admission = request(workers, 100_000_000);
    admission.policy = ExecutionRequestPolicy::new(
        admission.policy.posture(),
        admission.policy.determinism(),
        ExecutionBudget::new(
            admission.policy.budget().max_workers(),
            memory_bytes,
            100_000_000,
        ),
    );
    admission
}

fn run_trial(workers: usize, memory_bytes: u64) -> Result<Trial, SignalError> {
    let (mut graph, sources, targets) = graph_with_settled_sources(TARGET_COUNT);
    let calls = (0..TARGET_COUNT)
        .map(|_| AtomicUsize::new(0))
        .collect::<Vec<_>>();
    let lease = authority()
        .request_lease(memory_request(workers, memory_bytes))
        .unwrap();
    let outcome = graph.evaluate_checked(
        &targets,
        EvaluationRequestMode::Default,
        &(),
        &|ctx| {
            let index = targets.iter().position(|node| *node == ctx.node()).unwrap();
            calls[index].fetch_add(1, Ordering::SeqCst);
            let value = sources.iter().try_fold(index as u64, |value, source| {
                Ok::<u64, SignalError>(value + ctx.read(*source, VALUE)?)
            })?;
            Ok(AspectVersion::zero().with(VALUE, value))
        },
        &lease,
    );
    let report = match outcome {
        Ok(report) => report,
        Err(error) => {
            if let SignalError::ExecutionStopped(stop) = &error {
                let callbacks = calls
                    .iter()
                    .map(|counter| counter.load(Ordering::SeqCst))
                    .sum::<usize>();
                assert_eq!(
                    callbacks,
                    stop.publication_progress().completed_tasks(),
                    "resource denial may only follow a fully published callback prefix: {stop:?}"
                );
            }
            return Err(error);
        }
    };
    assert_eq!(report.tasks_executed, TARGET_COUNT as u32);
    assert!(calls.iter().all(|calls| calls.load(Ordering::SeqCst) == 1));
    let values = targets
        .iter()
        .map(|node| graph.node_aspect_version(*node).unwrap().get(VALUE))
        .collect::<Vec<_>>();
    let mut performed_stage_bindings = graph
        .invalidation_performed_work()
        .into_iter()
        .filter_map(|binding| {
            targets
                .iter()
                .position(|target| *target == binding.target)
                .map(|index| {
                    (
                        index,
                        binding.readiness_epoch.0,
                        binding.stage_order.stage,
                        binding.stage_order.order,
                    )
                })
        })
        .collect::<Vec<_>>();
    performed_stage_bindings.sort_unstable_by_key(|binding| binding.0);
    assert_eq!(
        values,
        (0..TARGET_COUNT)
            .map(|index| SOURCE_COUNT as u64 + index as u64)
            .collect::<Vec<_>>()
    );
    assert!(!report.execution.is_empty());
    assert!(
        report
            .execution
            .last()
            .unwrap()
            .physical()
            .active_workers_high_watermark()
            <= workers
    );
    Ok(Trial {
        stages: report.stages.len(),
        parallel_stages: report
            .stages
            .iter()
            .filter(|stage| {
                stage.outcome == crate::logic::planner::StageExecutionOutcome::CompletedParallel
            })
            .count(),
        charged_work: report.execution.last().unwrap().charged_work(),
        values,
        performed_stage_bindings,
    })
}

#[test]
fn bounded_epoch_backpressure_uses_multiple_waves_without_duplicate_evaluation() {
    // Explicit finite budgets probe the public resource boundary. The test
    // requires one that admits a strict multi-task prefix; it does not assume
    // an internal byte formula for an epoch grant.
    let budgets_kib = [
        64, 96, 128, 192, 256, 384, 512, 768, 1024, 1536, 2048, 2560, 3072, 3584, 4096, 4608, 5120,
        6144, 7168, 8192, 9216, 10240, 12288, 16384, 24576, 32768, 36864, 40960, 45056, 49152,
        53248, 57344, 65536,
    ];
    let mut selected = None;
    for kib in budgets_kib {
        let mut trials = Vec::new();
        for workers in [1, 2, 4] {
            match run_trial(workers, kib * 1024) {
                Ok(trial) => trials.push(trial),
                Err(SignalError::ExecutionStopped(stop))
                    if matches!(
                        stop.reason(),
                        crate::data::error::SignalExecutionStopReason::PreparationMemoryExhausted { .. }
                            | crate::data::error::SignalExecutionStopReason::Admission(_)
                    ) =>
                {
                    break;
                }
                Err(error) => panic!("unexpected backpressure outcome at {kib} KiB: {error}"),
            }
        }
        if trials.len() == 3
            && trials[2].stages > 1
            && trials[2].stages < TARGET_COUNT
            && trials[2].parallel_stages > 0
        {
            selected = Some((kib, trials));
            break;
        }
    }
    let (kib, trials) =
        selected.expect("some explicit memory budget must admit a strict bounded prefix");
    assert_eq!(
        trials[0].parallel_stages, 0,
        "one worker stays serial at {kib} KiB"
    );
    for trial in &trials[1..] {
        assert_eq!(trial.values, trials[0].values);
        assert_eq!(trial.charged_work, trials[0].charged_work);
    }
    let bindings = &trials[2].performed_stage_bindings;
    assert_eq!(bindings.len(), TARGET_COUNT);
    let readiness_epoch = bindings[0].1;
    let stage = bindings[0].2;
    for (index, binding) in bindings.iter().enumerate() {
        assert_eq!(binding.0, index);
        assert_eq!(
            binding.1, readiness_epoch,
            "resource slices changed stage readiness"
        );
        assert_eq!(binding.2, stage);
        assert_eq!(
            binding.3, index as u32,
            "resource slices changed task order"
        );
    }
}
