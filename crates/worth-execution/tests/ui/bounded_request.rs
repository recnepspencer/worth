use worth_execution::{
    CancellationToken, ExecutionMemoryReservation, ExecutionRequest, SerialRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
fn main() {
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, 4096, 100),
    );
    let serial = SerialRequest::from_policy(&policy, CancellationToken::new(), None);
    ExecutionRequest::serial(&serial)
        .in_scope(|lease| {
            assert!(lease.is_none());
            let _memory = ExecutionMemoryReservation::reserve_in_scope(
                lease,
                std::mem::size_of::<u64>() as u64,
            )
            .unwrap();
        })
        .unwrap();
}
