//! A public plan can reorder one stage; physical slicing must preserve its semantic order.
use super::support::{authority, request};
use crate::facade::{
    Aspect, AspectVersion, BoundedSignalInputs, DeclaredSignalInput, EvaluationRequestMode,
    NodeContract, NodeEvaluationResult, NodeId, SignalError, SignalGraph, SignalRuntimePolicy,
};
use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};

const VALUE: Aspect = Aspect::new(0);
const SOURCES: usize = 16;
const TARGETS: usize = 8;

struct Trial {
    stages: usize,
    publication: Vec<(u64, NodeId)>,
    task_order: Vec<NodeId>,
    artifacts: Vec<Option<crate::data::trace::RuntimeArtifactState>>,
    explanation: Vec<Option<crate::diagnostics::facts::ExplanationFact>>,
    provenance: Vec<Option<crate::diagnostics::facts::ProvenanceFact>>,
    lineage: Vec<crate::diagnostics::lineage::LineageRecord>,
    lineage_allocators: (u64, u64),
}

fn fixture() -> (SignalGraph, Vec<NodeId>, Vec<NodeId>) {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::forensic().with_parallel_admission(
        crate::facade::ParallelAdmissionPolicy {
            throughput_min_parallel_tasks: 1,
            balanced_min_parallel_tasks: 1,
            latency_bounded_min_parallel_tasks: 1,
            full_parallel_min_tasks: 1,
        },
    ));
    let sources = (0..SOURCES)
        .map(|_| {
            graph
                .node()
                .with_contract(
                    NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()),
                )
                .build()
        })
        .collect::<Vec<_>>();
    let bootstrap = authority().request_lease(request(4, 10_000_000)).unwrap();
    graph
        .evaluate_checked(
            &sources,
            EvaluationRequestMode::Default,
            &(),
            &|_| Ok(AspectVersion::zero().with(VALUE, 1)),
            &bootstrap,
        )
        .unwrap();
    drop(bootstrap);
    let targets = (0..TARGETS)
        .map(|_| {
            graph
                .node()
                .with_contract(
                    NodeContract::wildcard()
                        .with_bounded_inputs(BoundedSignalInputs::new(
                            sources
                                .iter()
                                .copied()
                                .map(|source| DeclaredSignalInput::new(source, VALUE)),
                        ))
                        .with_max_checked_result_heap_bytes(4 * 1024),
                )
                .build()
        })
        .collect();
    (graph, sources, targets)
}

fn trial(base: &SignalGraph, sources: &[NodeId], targets: &[NodeId], memory: u64) -> Option<Trial> {
    let mut graph = base.clone();
    let before = graph.published_output_commit_order_for_test().len();
    let mut plan = graph
        .build_evaluation_plan(targets, EvaluationRequestMode::Default)
        .unwrap();
    assert_eq!(plan.stages.len(), 1);
    assert_eq!(plan.stages[0].tasks.len(), TARGETS);
    plan.stages[0].tasks.reverse();
    let expected = targets.iter().rev().copied().collect::<Vec<_>>();
    let mut admission = request(4, 100_000_000);
    admission.policy = ExecutionRequestPolicy::new(
        admission.policy.posture(),
        admission.policy.determinism(),
        ExecutionBudget::new(admission.policy.budget().max_workers(), memory, 100_000_000),
    );
    let lease = authority().request_lease(admission).ok()?;
    let report = match graph.execute_prepared_plan_checked(
        &plan,
        &(),
        &|ctx| {
            let mut value = ctx.node().index() as u64;
            for &source in sources {
                value += ctx.read(source, VALUE)?;
            }
            Ok::<_, SignalError>(
                NodeEvaluationResult::from_version(AspectVersion::zero().with(VALUE, value))
                    .with_output_identity(format!("{value}:{}", "x".repeat(1024))),
            )
        },
        &lease,
    ) {
        Ok(report) => report,
        Err(SignalError::ExecutionStopped(stop))
            if matches!(
                stop.reason(),
                crate::data::error::SignalExecutionStopReason::PreparationMemoryExhausted { .. }
                    | crate::data::error::SignalExecutionStopReason::Admission(
                        worth_execution::LeaseDenial::ResourceExhausted
                    )
            ) =>
        {
            return None;
        }
        Err(error) => panic!("unexpected reversed-plan request stop: {error:?}"),
    };
    assert_eq!(report.tasks_executed, TARGETS as u32);
    let task_order = report
        .stages
        .iter()
        .flat_map(|stage| stage.task_records.iter().map(|record| record.node))
        .collect::<Vec<_>>();
    assert_eq!(task_order, expected);
    let publication = graph.published_output_commit_order_for_test();
    let publication = publication[before..].to_vec();
    assert_eq!(
        publication
            .iter()
            .map(|(_, node)| *node)
            .collect::<Vec<_>>(),
        expected,
    );
    Some(Trial {
        stages: report.stages.len(),
        publication,
        task_order,
        artifacts: targets
            .iter()
            .map(|node| graph.node_runtime_artifact_state(*node).unwrap().cloned())
            .collect(),
        explanation: targets
            .iter()
            .map(|node| graph.explanation_fact(*node).cloned())
            .collect(),
        provenance: targets
            .iter()
            .map(|node| graph.provenance_fact(*node).cloned())
            .collect(),
        lineage: graph
            .diagnostics_state()
            .lineage_records()
            .iter()
            .cloned()
            .collect(),
        lineage_allocators: graph.diagnostics_state().lineage_allocator_state(),
    })
}

#[test]
fn reversed_public_stage_mints_the_same_artifacts_in_one_epoch_and_singletons() {
    let (base, sources, targets) = fixture();
    let funded = trial(&base, &sources, &targets, 128 * 1024 * 1024).unwrap();
    assert_eq!(funded.stages, 1, "funded request must publish one epoch");
    let mut low = 1;
    let mut high = 128 * 1024 * 1024;
    while low < high {
        let middle = low + (high - low) / 2;
        if trial(&base, &sources, &targets, middle).is_some() {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    let sliced = trial(&base, &sources, &targets, low).expect("minimum funded request completes");
    assert_eq!(
        sliced.stages, TARGETS,
        "minimum funded request must slice into singletons"
    );
    assert_eq!(sliced.publication, funded.publication);
    assert_eq!(sliced.task_order, funded.task_order);
    assert_eq!(sliced.artifacts, funded.artifacts);
    assert_eq!(sliced.explanation, funded.explanation);
    assert_eq!(sliced.provenance, funded.provenance);
    assert_eq!(sliced.lineage, funded.lineage);
    assert_eq!(sliced.lineage_allocators, funded.lineage_allocators);
}
