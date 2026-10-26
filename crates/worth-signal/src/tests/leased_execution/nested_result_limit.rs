//! A finite ancestor funds result custody before nested callback effects.
use super::support::{authority, request};
use crate::facade::{
    Aspect, AspectVersion, BoundedSignalInputs, EvaluationRequestMode, NodeContract,
    NodeEvaluationResult, SignalError, SignalExecutionFailure, SignalExecutionStopReason,
    SignalGraph,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};
use worth_execution::{CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest};

const VALUE: Aspect = Aspect::new(0);
const LIMIT: u64 = 32 << 20;

fn serial(limit: Option<u64>) -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(limit),
        CancellationToken::new(),
        None,
    )
}

fn contract_violation(error: &SignalError) -> bool {
    match error {
        SignalError::CheckedResultCapacityExceeded { required, declared } => required > declared,
        SignalError::ExecutionStopped(stop) => match stop.reason() {
            SignalExecutionStopReason::Failure {
                cause: SignalExecutionFailure::Domain(error),
                ..
            } => contract_violation(error),
            SignalExecutionStopReason::Failure {
                cause: SignalExecutionFailure::ResultCapacityExceeded,
                ..
            } => true,
            _ => false,
        },
        _ => false,
    }
}

fn check_nested(outer: ExecutionRequest<'_, '_>, inner: ExecutionRequest<'_, '_>) {
    check_case(outer, inner, None);
    // Include the label Vec backing as well as the label's own allocation.
    check_case(outer, inner, Some((48 << 20) + 1_024));
}

fn check_case(
    outer: ExecutionRequest<'_, '_>,
    inner: ExecutionRequest<'_, '_>,
    declared: Option<u64>,
) {
    let effects = AtomicUsize::new(0);
    let observed = Mutex::new(None);
    let mut graph = SignalGraph::new();
    let node = graph
        .node()
        .with_contract(NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()))
        .build();
    // The outer result is inline, so its result grant does not consume the parent's room.
    let _outer = graph.evaluate_checked(
        &[node],
        EvaluationRequestMode::Default,
        &(),
        &|_| {
            let mut nested = SignalGraph::new();
            let mut contract = NodeContract::wildcard()
                .with_produces(VALUE)
                .with_bounded_inputs(BoundedSignalInputs::default());
            if let Some(bytes) = declared {
                contract = contract.with_max_checked_result_heap_bytes(bytes);
            }
            let target = nested.node().with_contract(contract).build();
            let outcome = nested.evaluate_checked(
                &[target],
                EvaluationRequestMode::Default,
                &(),
                &|_| {
                    effects.fetch_add(1, Ordering::SeqCst);
                    Ok(
                        NodeEvaluationResult::from_version(AspectVersion::zero().with(VALUE, 7))
                            .with_label("x".repeat(48 << 20)),
                    )
                },
                inner,
            );
            let error = outcome.unwrap_err();
            assert_eq!(nested.node_aspect_version(target).unwrap().get(VALUE), 0);
            *observed.lock().unwrap() = Some(error);
            Ok(AspectVersion::zero())
        },
        outer,
    );
    let error = observed
        .into_inner()
        .unwrap()
        .unwrap_or_else(|| panic!("outer callback must run: {_outer:?}"));
    // A callback that has run may only breach its pre-effect contract. An admission
    // refusal (including an unsupported mixed scope) must precede every effect.
    if declared.is_some() {
        assert_eq!(
            effects.load(Ordering::SeqCst),
            0,
            "declared result must be refused before effects: {error:?}"
        );
    }
    assert!(
        effects.load(Ordering::SeqCst) == 0 || contract_violation(&error),
        "{error:?}"
    );
}

#[test]
fn nested_serial_result_custody_observes_finite_ancestor() {
    check_nested(
        ExecutionRequest::serial(&serial(Some(LIMIT))),
        ExecutionRequest::serial(&serial(None)),
    );
}

#[test]
fn nested_lease_result_custody_observes_finite_ancestor() {
    let mut admission = request(1, 20_000_000);
    admission.policy = worth_foundational::ExecutionRequestPolicy::new(
        admission.policy.posture(),
        admission.policy.determinism(),
        worth_foundational::ExecutionBudget::new(
            admission.policy.max_workers(),
            Some(LIMIT),
            Some(20_000_000),
        ),
    );
    let parent = authority().request_lease(admission).unwrap();
    let child = parent.child(request(1, 20_000_000)).unwrap();
    check_nested(
        ExecutionRequest::leased(&parent),
        ExecutionRequest::leased(&child),
    );
}

#[test]
fn mixed_nested_result_requests_keep_pre_effect_custody() {
    let lease = authority().request_lease(request(1, 20_000_000)).unwrap();
    check_nested(
        ExecutionRequest::serial(&serial(Some(LIMIT))),
        ExecutionRequest::leased(&lease),
    );
    check_nested(
        ExecutionRequest::leased(&lease),
        ExecutionRequest::serial(&serial(None)),
    );
}

#[test]
fn serial_ancestor_bounds_a_lease_without_its_own_or_process_ceiling() {
    let lease = worth_execution::ExecutionResourceLease::serial(request(1, 20_000_000)).unwrap();
    assert_eq!(lease.available_memory_bytes(), None);
    check_nested(
        ExecutionRequest::serial(&serial(Some(LIMIT))),
        ExecutionRequest::leased(&lease),
    );
}
