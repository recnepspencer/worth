//! A sibling's process custody must constrain even intrinsically bounded outputs.
use std::sync::atomic::{AtomicUsize, Ordering};

use super::support::private_authority_process::{
    run_with_private_authority, NativeExecutionEvidence,
};
use super::support::{authority, request};
use crate::facade::{
    Aspect, AspectVersion, BoundedSignalInputs, EvaluationRequestMode, NodeContract, SignalError,
    SignalExecutionFailure, SignalExecutionStopReason, SignalGraph, SignalLeaseDenial,
};
use worth_execution::ExecutionRequest;
use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};

#[test]
fn typed_outputs_publish_or_refuse_before_callbacks_across_process_room_boundary() {
    if run_with_private_authority(
        "tests::leased_execution::typed_result_room::typed_outputs_publish_or_refuse_before_callbacks_across_process_room_boundary",
        NativeExecutionEvidence::TypedResultRoom,
    ) { return; }
    const VALUE: Aspect = Aspect::new(0);
    let sibling = authority().request_lease(request(2, u64::MAX)).unwrap();
    let mut evaluation = request(2, u64::MAX);
    evaluation.policy = ExecutionRequestPolicy::new(
        evaluation.policy.posture(),
        evaluation.policy.determinism(),
        ExecutionBudget::new(evaluation.policy.max_workers(), Some(64 << 20), None),
    );
    let lease = authority().request_lease(evaluation).unwrap();
    let process_room = sibling.available_memory_bytes().unwrap();
    for width in [1_usize, 8] {
        let mut refused = 0;
        let mut published = 0;
        for room in (4_096..=1_048_576).step_by(256) {
            let hold = sibling.reserve_memory(process_room - room).unwrap();
            assert_eq!(lease.available_memory_bytes(), Some(room));
            let mut graph = SignalGraph::new();
            let nodes: Vec<_> = (0..width)
                .map(|_| {
                    graph
                        .node()
                        .with_contract(
                            NodeContract::wildcard()
                                .with_produces(VALUE)
                                .with_bounded_inputs(BoundedSignalInputs::default()),
                        )
                        .build()
                })
                .collect();
            let calls = AtomicUsize::new(0);
            let outcome = graph.evaluate_checked(
                &nodes,
                EvaluationRequestMode::Default,
                &(),
                &|_| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Ok(AspectVersion::zero().with(VALUE, 7))
                },
                ExecutionRequest::leased(&lease),
            );
            match outcome {
                Ok(report) => {
                    assert_eq!(report.tasks_executed as usize, width);
                    assert_eq!(calls.load(Ordering::SeqCst), width);
                    for node in nodes {
                        assert_eq!(graph.node_aspect_version(node).unwrap().get(VALUE), 7);
                    }
                    published += 1;
                }
                Err(error) => {
                    assert_eq!(
                        calls.load(Ordering::SeqCst),
                        0,
                        "width={width}, room={room}: {error:?}"
                    );
                    assert!(memory_refusal(&error), "unexpected refusal: {error:?}");
                    for node in nodes {
                        assert_eq!(graph.node_aspect_version(node).unwrap().get(VALUE), 0);
                    }
                    refused += 1;
                }
            }
            assert_eq!(lease.charged_memory_bytes(), 0);
            drop(hold);
        }
        assert!(
            refused > 0 && published > 0,
            "width={width}: boundary not crossed"
        );
    }
    println!("TYPED_RESULT_ROOM both widths crossed admission without late refusals");
}

fn memory_refusal(error: &SignalError) -> bool {
    match error {
        SignalError::PreparationMemoryExhausted { .. }
        | SignalError::ExecutionAdmissionDenied(SignalLeaseDenial::MemoryExhausted(_)) => true,
        SignalError::ExecutionStopped(stop) => match stop.reason() {
            SignalExecutionStopReason::PreparationMemoryExhausted { .. }
            | SignalExecutionStopReason::Admission(SignalLeaseDenial::MemoryExhausted(_)) => true,
            SignalExecutionStopReason::Failure {
                cause: SignalExecutionFailure::Domain(error),
                ..
            } => memory_refusal(error),
            _ => false,
        },
        _ => false,
    }
}
