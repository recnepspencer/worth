use super::*;
use std::{cell::Cell, num::NonZeroUsize};
use worth_execution::{
    CancellationSource, CancellationToken, MapKernelStop, SerialRequest, WorkCeilingDenial,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

fn native_request(cancellation: CancellationToken) -> SerialRequest {
    SerialRequest::from_policy(
        &ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            NonZeroUsize::MIN,
        ),
        cancellation,
        None,
    )
}

#[test]
fn decode_scope_keeps_worker_only_native_and_preserves_an_accounted_ancestor() {
    let backing = native_request(CancellationToken::new());
    let request = ExecutionRequest::serial(&backing);
    let native = ValueDecodeState::new(Default::default());
    native
        .run(&request, |session| {
            assert!(session.checkpoint.is_none());
            assert!(native.memory.borrow().is_none());
        })
        .unwrap();

    let accounted = ValueDecodeState::new(Default::default());
    let parent = SerialRequest::from_policy(
        &ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, None, None),
        ),
        CancellationToken::new(),
        None,
    );
    ExecutionRequest::serial(&parent)
        .run(ExecutionWorkCeiling::new(None), |_| {
            accounted
                .run(&request, |session| {
                    assert!(session.checkpoint.is_some());
                    assert!(accounted.memory.borrow().is_some());
                })
                .unwrap();
        })
        .unwrap();
}

#[test]
fn cancelled_native_decode_stops_before_body_and_keeps_its_original_refusal() {
    let cancellation = CancellationSource::new();
    let backing = native_request(cancellation.token());
    let request = ExecutionRequest::serial(&backing);
    let state = ValueDecodeState::new(Default::default());
    let called = Cell::new(false);
    cancellation.cancel();
    let expected = Stop::Scope(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled));
    assert_eq!(state.run(&request, |_| called.set(true)), Err(expected));
    assert!(!called.get());
    assert_eq!(state.denial(), Some(expected));
    assert_eq!(state.run(&request, |_| called.set(true)), Err(expected));
    assert!(!called.get());
    assert!(state.memory.borrow().is_none());
}
