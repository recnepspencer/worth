//! Several producer publications accumulate causes for one unscoped consumer.
use std::sync::atomic::{AtomicUsize, Ordering};

use worth_execution::LeaseRequest;
use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};

use crate::facade::{
    mark_dirty, Aspect, AspectVersion, BoundedSignalInputs, DeclaredSignalInput,
    EvaluationRequestMode, NodeContract, NodeId, SignalError, SignalExecutionStopReason,
    SignalGraph,
};
use crate::logic::planner::EvaluationPlan;

use super::support::{authority, request};

const VALUE: Aspect = Aspect::new(0);
const PRODUCERS: usize = 16;
const FUNDED: u64 = 128 * 1024 * 1024;

fn admission(memory: u64) -> LeaseRequest {
    let mut admission = request(4, 1_000_000_000);
    admission.policy = ExecutionRequestPolicy::new(
        admission.policy.posture(),
        admission.policy.determinism(),
        ExecutionBudget::new(
            admission.policy.budget().max_workers(),
            memory,
            1_000_000_000,
        ),
    );
    admission
}

struct Fixture {
    graph: SignalGraph,
    producers: Vec<NodeId>,
    consumer: NodeId,
    plan: EvaluationPlan,
}

fn fixture() -> Fixture {
    let mut graph = SignalGraph::new();
    let producers = (0..PRODUCERS)
        .map(|_| {
            graph
                .node()
                .with_contract(
                    NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()),
                )
                .build()
        })
        .collect::<Vec<_>>();
    let lease = authority().request_lease(admission(FUNDED)).unwrap();
    graph
        .evaluate_checked(
            &producers,
            EvaluationRequestMode::Default,
            &(),
            &|_| Ok(AspectVersion::zero().with(VALUE, 1)),
            worth_execution::ExecutionRequest::leased(&lease),
        )
        .expect("producer facts settle before the shared consumer reads them");
    drop(lease);
    let consumer = graph
        .node()
        .with_contract(
            NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::new(
                producers
                    .iter()
                    .copied()
                    .map(|source| DeclaredSignalInput::new(source, VALUE)),
            )),
        )
        .build();
    let lease = authority().request_lease(admission(FUNDED)).unwrap();
    graph
        .evaluate_checked(
            &[consumer],
            EvaluationRequestMode::Default,
            &(),
            &|context| {
                let sum = producers.iter().try_fold(0, |sum, source| {
                    Ok::<u64, SignalError>(sum + context.read(*source, VALUE)?)
                })?;
                Ok(AspectVersion::zero().with(VALUE, sum))
            },
            worth_execution::ExecutionRequest::leased(&lease),
        )
        .expect("the shared consumer installs real unscoped dependencies");
    drop(lease);
    for &producer in &producers {
        assert_eq!(graph.subscribers_of(producer).unwrap().len(), 1);
        mark_dirty(&mut graph, producer, VALUE).unwrap();
    }
    let plan = graph
        .build_evaluation_plan(&producers, EvaluationRequestMode::Default)
        .unwrap();
    assert_eq!(plan.summary.task_count, PRODUCERS as u32);
    Fixture {
        graph,
        producers,
        consumer,
        plan,
    }
}

fn completes(fixture: &Fixture, memory: u64) -> bool {
    let mut graph = fixture.graph.clone();
    let calls = AtomicUsize::new(0);
    let lease = authority().request_lease(admission(memory)).unwrap();
    let outcome = graph.execute_prepared_plan_checked(
        &fixture.plan,
        &(),
        &|_| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(AspectVersion::zero().with(VALUE, 2))
        },
        worth_execution::ExecutionRequest::leased(&lease),
    );
    let (completed, success) = match outcome {
        Ok(report) => {
            assert_eq!(report.tasks_executed, PRODUCERS as u32);
            assert!(
                report
                    .execution
                    .last()
                    .unwrap()
                    .physical()
                    .peak_charged_memory_bytes()
                    <= memory
            );
            (PRODUCERS, true)
        }
        Err(SignalError::ExecutionStopped(stop)) => {
            assert!(
                matches!(
                    stop.reason(),
                    SignalExecutionStopReason::PreparationMemoryExhausted { .. }
                        | SignalExecutionStopReason::Admission(
                            crate::data::error::SignalLeaseDenial::MemoryExhausted(_)
                        )
                ),
                "unexpected stop: {stop:?}"
            );
            let completed = stop.publication_progress().completed_tasks();
            assert_eq!(
                calls.load(Ordering::SeqCst),
                completed,
                "every evaluated producer belongs to a committed epoch: {stop:?}"
            );
            (completed, false)
        }
        Err(error) => panic!("shared-consumer request lost its resource outcome: {error}"),
    };
    assert_eq!(calls.load(Ordering::SeqCst), completed);
    assert_eq!(
        graph
            .node_aspect_version(fixture.consumer)
            .unwrap()
            .get(VALUE),
        PRODUCERS as u64
    );
    assert_eq!(
        fixture
            .producers
            .iter()
            .filter(|producer| { graph.node_aspect_version(**producer).unwrap().get(VALUE) == 2 })
            .count(),
        completed
    );
    success
}

#[test]
fn accumulated_unscoped_causes_are_admitted_before_producer_callbacks() {
    let fixture = fixture();
    let mut denied = 64 * 1024;
    let mut completed = FUNDED;
    assert!(!completes(&fixture, denied));
    assert!(completes(&fixture, completed));
    while denied + 1 < completed {
        let probe = denied + (completed - denied) / 2;
        if completes(&fixture, probe) {
            completed = probe;
        } else {
            denied = probe;
        }
    }
    assert!(!completes(&fixture, completed - 1));
    assert!(completes(&fixture, completed));
}
