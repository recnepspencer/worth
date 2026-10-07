use super::*;
use worth_execution::{
    CancellationToken, ExecutionMemoryReservation, ExecutionRequest, LeaseDenial, SerialRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn legacy_precompute_refuses_memory_after_request_admission() {
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, 8192, 1000),
    );
    let serial = SerialRequest::from_policy(&policy, CancellationToken::new(), None);
    let request = ExecutionRequest::serial(&serial);
    let mut entered = false;
    let result = crate::logic::planner::execution::run_signal_preparation_request(
        request,
        |_, _, preparation| {
            entered = true;
            request
                .in_scope(|lease| {
                    let LeaseDenial::MemoryExhausted(refusal) =
                        ExecutionMemoryReservation::reserve_in_scope(
                            lease,
                            policy.budget().charged_memory_bytes(),
                        )
                        .unwrap_err()
                    else {
                        panic!("admitted framework holds memory");
                    };
                    let _held =
                        ExecutionMemoryReservation::reserve_in_scope(lease, refusal.admitted)
                            .unwrap();
                    allocate_legacy_values(1, preparation)
                })
                .map_err(SignalError::execution_scope_denied)?
        },
    );
    assert!(
        entered,
        "the request admitted before its precompute allocation"
    );
    let SignalError::ExecutionStopped(stop) = result.unwrap_err() else {
        panic!("typed stop required");
    };
    let crate::facade::SignalExecutionStopReason::Admission(
        crate::facade::SignalLeaseDenial::MemoryExhausted(memory),
    ) = stop.reason()
    else {
        panic!("precompute memory cause required: {stop:?}");
    };
    assert_eq!(
        memory.requested - memory.admitted,
        std::mem::size_of::<PreparedEvaluation>() as u64
    );
}
