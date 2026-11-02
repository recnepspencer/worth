//! Pre-effect original delivery storage retains and returns real SDK funding.
use super::*;
use std::num::NonZeroUsize;
use worth_execution::{
    CancellationToken, ExecutionAllocationPolicy, ExecutionResourceLease, LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
fn lease(bytes: u64) -> ExecutionResourceLease<'static> {
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, Some(bytes), Some(100)),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .unwrap()
}
#[test]
fn original_delivery_storage_keeps_funding_until_each_owner_drops() {
    let lease = lease(8 * 1024 * 1024);
    let baseline = lease.charged_memory_bytes();
    let (publisher, receiver) =
        OriginalCompletionPublisher::pending_slot(ExecutionAllocationPolicy::Execution(&lease))
            .unwrap_or_else(|cause| panic!("funded original storage: {cause:?}"));
    let total = lease.charged_memory_bytes();
    assert!(total > baseline);
    drop(publisher);
    assert_eq!(
        lease.charged_memory_bytes(),
        total,
        "the original receiver still owns both allocations"
    );
    let envelope = receiver.cleanup_envelope();
    assert!(
        lease.charged_memory_bytes() > baseline,
        "cleanup envelope retains its separate funding"
    );
    assert!(
        lease.charged_memory_bytes() < total,
        "the last slot owner returns its funding"
    );
    drop(envelope);
    assert_eq!(lease.charged_memory_bytes(), baseline);
}
#[test]
fn original_delivery_storage_refuses_without_room_and_returns_its_reservations() {
    let slot_bytes =
        (std::mem::size_of::<PendingOriginalSlot>() + 2 * std::mem::size_of::<usize>()) as u64;
    let lease = lease(slot_bytes);
    let baseline = lease.charged_memory_bytes();
    let result =
        OriginalCompletionPublisher::pending_slot(ExecutionAllocationPolicy::Execution(&lease));
    let cause = match result {
        Ok(_) => panic!("the cleanup envelope requires its own admitted backing"),
        Err(cause) => cause,
    };
    assert!(matches!(
        cause.classify(),
        worth_execution::ExecutionAllocationDenialClass::Request(
            worth_execution::WorkCeilingDenial::Admission(
                worth_execution::LeaseDenial::MemoryExhausted(_)
            )
        )
    ));
    assert_eq!(
        lease.charged_memory_bytes(),
        baseline,
        "failed envelope admission returns the earlier slot reservation"
    );
}
