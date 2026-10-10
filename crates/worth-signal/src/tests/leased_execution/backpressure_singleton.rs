use std::sync::atomic::{AtomicUsize, Ordering};

use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};

use crate::facade::{
    Aspect, AspectVersion, BoundedSignalInputs, DeclaredSignalInput, EvaluationRequestMode,
    NodeContract, NodeId, SignalError, SignalGraph,
};
use crate::logic::planner::EvaluationPlan;

use super::support::{authority, request};

const VALUE: Aspect = Aspect::new(0);
const EXISTING_CONSUMERS: usize = 128;

struct Fixture {
    graph: SignalGraph,
    source: NodeId,
    target: NodeId,
    plan: EvaluationPlan,
}

fn request_with_memory(memory_bytes: u64) -> worth_execution::LeaseRequest {
    let mut admission = request(4, 10_000_000);
    admission.policy = ExecutionRequestPolicy::new(
        admission.policy.posture(),
        admission.policy.determinism(),
        ExecutionBudget::new(
            admission.policy.budget().max_workers(),
            memory_bytes,
            10_000_000,
        ),
    );
    admission
}

fn fixture(existing_reads_source: bool) -> Fixture {
    let mut graph = SignalGraph::new();
    let source = graph
        .node()
        .with_contract(NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()))
        .build();
    // Setup admits committed source and subscriber facts under a separate
    // measured ceiling. The target request below probes much smaller leases.
    let source_lease = authority()
        .request_lease(request_with_memory(64 * 1024 * 1024))
        .unwrap();
    graph
        .evaluate_checked(
            &[source],
            EvaluationRequestMode::Default,
            &(),
            &|_| Ok(AspectVersion::zero().with(VALUE, 1)),
            worth_execution::ExecutionRequest::leased(&source_lease),
        )
        .expect("source settles before existing consumers");
    drop(source_lease);

    let consumers = (0..EXISTING_CONSUMERS)
        .map(|_| {
            graph
                .node()
                .with_contract(NodeContract::wildcard().with_bounded_inputs(
                    BoundedSignalInputs::new([DeclaredSignalInput::new(source, VALUE)]),
                ))
                .build()
        })
        .collect::<Vec<_>>();
    // Each settled consumer is fixture setup. A singleton request keeps the
    // setup below its own preparation ceiling as subscriber membership grows.
    for batch in consumers.chunks(1) {
        let setup_lease = authority()
            .request_lease(request_with_memory(64 * 1024 * 1024))
            .unwrap();
        graph
            .evaluate_checked(
                batch,
                EvaluationRequestMode::Default,
                &(),
                &|ctx| {
                    let value = if existing_reads_source {
                        ctx.read(source, VALUE)? + 1
                    } else {
                        2
                    };
                    Ok(AspectVersion::zero().with(VALUE, value))
                },
                worth_execution::ExecutionRequest::leased(&setup_lease),
            )
            .expect("existing consumer facts settle before the target request");
    }
    assert_eq!(
        graph.subscribers_of(source).unwrap().len(),
        if existing_reads_source {
            EXISTING_CONSUMERS
        } else {
            0
        }
    );
    let target = graph
        .node()
        .with_contract(
            NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::new([
                DeclaredSignalInput::new(source, VALUE),
            ])),
        )
        .build();
    let plan = graph
        .build_evaluation_plan(&[target], EvaluationRequestMode::Default)
        .expect("prepared target plan uses a settled producer");
    assert_eq!(plan.summary.task_count, 1);
    Fixture {
        graph,
        source,
        target,
        plan,
    }
}

fn execute(
    fixture: &mut Fixture,
    memory_bytes: u64,
    calls: &AtomicUsize,
) -> Result<crate::logic::planner::ExecutionReport, SignalError> {
    let lease = authority()
        .request_lease(request_with_memory(memory_bytes))
        .unwrap();
    let source = fixture.source;
    fixture.graph.execute_prepared_plan_checked(
        &fixture.plan,
        &(),
        &|ctx| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(AspectVersion::zero().with(VALUE, ctx.read(source, VALUE)? + 2))
        },
        worth_execution::ExecutionRequest::leased(&lease),
    )
}

fn probe_from(base: &Fixture) -> Fixture {
    let mut graph = base.graph.clone();
    let plan = graph
        .build_evaluation_plan(&[base.target], EvaluationRequestMode::Default)
        .expect("cloned committed fixture rebuilds its target plan");
    assert_eq!(plan.summary.task_count, 1);
    Fixture {
        graph,
        source: base.source,
        target: base.target,
        plan,
    }
}

fn resource_stop(error: SignalError, calls: &AtomicUsize, fixture: &Fixture) -> bool {
    let SignalError::ExecutionStopped(stop) = error else {
        panic!("singleton resource stop must retain the execution report");
    };
    let prepared_denial = match stop.reason() {
        crate::data::error::SignalExecutionStopReason::PreparationMemoryExhausted { .. } => true,
        crate::data::error::SignalExecutionStopReason::Admission(_) => false,
        other => panic!("unexpected singleton stop: {other:?}"),
    };
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(stop.publication_progress().completed_tasks(), 0);
    assert_eq!(
        fixture
            .graph
            .node_aspect_version(fixture.target)
            .unwrap()
            .get(VALUE),
        0
    );
    prepared_denial
}

#[test]
fn source_subscriber_copy_is_admitted_before_a_singleton_callback() {
    let mut loaded = fixture(true);
    let control = fixture(false);
    let loaded_calls = AtomicUsize::new(0);
    // Locate the control threshold on clones of committed fixture state.
    // This budget comes from observed behavior, not the owner's capacity formula.
    let mut denied = 0_u64;
    let mut minimum_funded = 64 * 1024 * 1024_u64;
    while minimum_funded - denied > 1 {
        let middle = denied + (minimum_funded - denied) / 2;
        let mut candidate = probe_from(&control);
        let calls = AtomicUsize::new(0);
        match execute(&mut candidate, middle, &calls) {
            Ok(report) => {
                assert_eq!(report.tasks_executed, 1);
                assert_eq!(calls.load(Ordering::SeqCst), 1);
                minimum_funded = middle;
            }
            Err(error) => {
                resource_stop(error, &calls, &candidate);
                denied = middle;
            }
        }
    }
    let mut control_at_threshold = probe_from(&control);
    let control_calls = AtomicUsize::new(0);
    let report = execute(&mut control_at_threshold, minimum_funded, &control_calls)
        .expect("control completes at its measured minimum");
    assert_eq!(report.tasks_executed, 1);
    assert_eq!(control_calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        control_at_threshold
            .graph
            .node_aspect_version(control_at_threshold.target)
            .unwrap()
            .get(VALUE),
        3
    );
    let loaded_preparation = match execute(&mut loaded, minimum_funded, &loaded_calls) {
        Err(error) => resource_stop(error, &loaded_calls, &loaded),
        Ok(_) => panic!("subscriber-heavy target completed at the control minimum"),
    };
    assert!(
        loaded_preparation,
        "subscriber copies must deny before dispatch"
    );

    let funded = execute(&mut loaded, 64 * 1024 * 1024, &loaded_calls)
        .expect("a funded target request completes after the denied attempt");
    assert_eq!(funded.tasks_executed, 1);
    assert_eq!(loaded_calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        loaded
            .graph
            .node_aspect_version(loaded.target)
            .unwrap()
            .get(VALUE),
        3
    );
    assert_eq!(
        loaded.graph.subscribers_of(loaded.source).unwrap().len(),
        EXISTING_CONSUMERS + 1
    );
}
