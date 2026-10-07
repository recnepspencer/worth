//! Consumer-owned scopes must fit even when the producer declares no inputs.
use std::sync::atomic::{AtomicUsize, Ordering};

use worth_execution::LeaseRequest;
use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};

use crate::facade::{
    mark_dirty, Aspect, AspectVersion, BoundedSignalInputs, DeclaredSignalInput,
    EvaluationRequestMode, NodeContract, NodeId, PartitionSubscription, ScopePath, SignalError,
    SignalGraph,
};
use crate::logic::planner::EvaluationPlan;

use super::support::{authority, request};

const VALUE: Aspect = Aspect::new(0);
const CONSUMERS: usize = 8;
const FUNDED: u64 = 128 * 1024 * 1024;
// Scope setup and the probe both include real retained diagnostic copies.
// Keep work independently funded so this scenario isolates memory admission.
const FUNDED_WORK: u64 = 1_000_000_000;

struct Fixture {
    graph: SignalGraph,
    producer: NodeId,
    consumers: Vec<NodeId>,
    scopes: Vec<PartitionSubscription>,
    plan: EvaluationPlan,
}

fn admission(memory: u64) -> LeaseRequest {
    let mut request = request(4, FUNDED_WORK);
    request.policy = ExecutionRequestPolicy::new(
        request.policy.posture(),
        request.policy.determinism(),
        ExecutionBudget::new(request.policy.budget().max_workers(), memory, FUNDED_WORK),
    );
    request
}

fn fixture(segment_bytes: usize) -> Fixture {
    let mut graph = SignalGraph::new();
    let producer = graph
        .node()
        .with_contract(NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()))
        .partitioned_output()
        .build();
    let scopes = (0..CONSUMERS)
        .map(|consumer| {
            PartitionSubscription::exact(
                ScopePath::new(
                    (0..ScopePath::MAX_DEPTH)
                        .map(|level| format!("{consumer}-{level}-{}", "s".repeat(segment_bytes))),
                )
                .unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let lease = authority().request_lease(admission(FUNDED)).unwrap();
    graph
        .evaluate_checked(
            &[producer],
            EvaluationRequestMode::Default,
            &(),
            &|_| Ok(AspectVersion::zero().with(VALUE, 1)),
            &lease,
        )
        .expect("input-free producer settles before scoped consumer setup");
    drop(lease);
    let consumers = scopes
        .iter()
        .map(|scope| {
            graph
                .node()
                .with_contract(NodeContract::wildcard().with_bounded_inputs(
                    BoundedSignalInputs::new([DeclaredSignalInput::scoped(
                        producer,
                        VALUE,
                        scope.clone(),
                    )]),
                ))
                .build()
        })
        .collect::<Vec<_>>();
    for (&consumer, scope) in consumers.iter().zip(&scopes) {
        let lease = authority().request_lease(admission(FUNDED)).unwrap();
        graph
            .evaluate_checked(
                &[consumer],
                EvaluationRequestMode::Default,
                &(),
                &|context| {
                    Ok(AspectVersion::zero()
                        .with(VALUE, context.read_scoped(producer, VALUE, scope)? + 1))
                },
                &lease,
            )
            .expect("scoped reads install real reverse dependencies");
    }
    assert_eq!(graph.subscribers_of(producer).unwrap().len(), CONSUMERS);
    mark_dirty(&mut graph, producer, VALUE).unwrap();
    let plan = graph
        .build_evaluation_plan(&[producer], EvaluationRequestMode::Default)
        .unwrap();
    assert_eq!(plan.summary.task_count, 1);
    Fixture {
        graph,
        producer,
        consumers,
        scopes,
        plan,
    }
}

fn completes(fixture: &Fixture, memory: u64) -> bool {
    let mut graph = fixture.graph.clone();
    let calls = AtomicUsize::new(0);
    let lease = authority().request_lease(admission(memory)).unwrap();
    match graph.execute_prepared_plan_checked(
        &fixture.plan,
        &(),
        &|_| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(AspectVersion::zero().with(VALUE, 2))
        },
        &lease,
    ) {
        Ok(report) => {
            assert_eq!(report.tasks_executed, 1);
            assert_eq!(calls.load(Ordering::SeqCst), 1);
            assert_eq!(
                graph
                    .node_aspect_version(fixture.producer)
                    .unwrap()
                    .get(VALUE),
                2
            );
            assert!(
                report
                    .execution
                    .last()
                    .unwrap()
                    .physical()
                    .peak_charged_memory_bytes()
                    <= memory
            );
            assert_eq!(
                graph.subscribers_of(fixture.producer).unwrap().len(),
                CONSUMERS
            );
            for (&consumer, scope) in fixture.consumers.iter().zip(&fixture.scopes) {
                assert_eq!(
                    graph
                        .node_version_for_scope(fixture.producer, VALUE, Some(scope))
                        .unwrap(),
                    2
                );
                assert_eq!(graph.node_aspect_version(consumer).unwrap().get(VALUE), 2);
            }
            true
        }
        Err(SignalError::ExecutionStopped(stop)) => {
            assert!(
                matches!(
                    stop.reason(),
                    crate::facade::SignalExecutionStopReason::PreparationMemoryExhausted { .. }
                        | crate::facade::SignalExecutionStopReason::Admission(
                            crate::data::error::SignalLeaseDenial::MemoryExhausted(_)
                        )
                ),
                "unexpected resource stop: {stop:?}"
            );
            assert_eq!(
                calls.load(Ordering::SeqCst),
                0,
                "resource admission precedes the callback: {stop:?}"
            );
            assert_eq!(stop.publication_progress().completed_tasks(), 0);
            assert_eq!(
                graph
                    .node_aspect_version(fixture.producer)
                    .unwrap()
                    .get(VALUE),
                1
            );
            false
        }
        Err(error) => panic!("scoped producer request lost its resource outcome: {error}"),
    }
}

fn minimum_memory(fixture: &Fixture) -> u64 {
    let mut denied = 64 * 1024;
    let mut completed = FUNDED;
    assert!(!completes(fixture, denied));
    assert!(completes(fixture, completed));
    while denied + 1 < completed {
        let probe = denied + (completed - denied) / 2;
        if completes(fixture, probe) {
            completed = probe;
        } else {
            denied = probe;
        }
    }
    assert!(!completes(fixture, completed - 1));
    assert!(completes(fixture, completed));
    completed
}

#[test]
fn consumer_scope_heap_is_admitted_before_an_input_free_producer_callback() {
    let short = fixture(16);
    let long = fixture(1024);
    let short_minimum = minimum_memory(&short);
    let long_minimum = minimum_memory(&long);
    assert!(
        long_minimum > short_minimum,
        "consumer-owned scope heap must affect admission"
    );
    assert!(!completes(&long, short_minimum));
    assert!(completes(&short, short_minimum));
}
