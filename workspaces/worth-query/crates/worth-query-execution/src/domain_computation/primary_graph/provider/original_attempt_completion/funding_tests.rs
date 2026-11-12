//! Pre-effect storage borrows its production owner's actual bootstrap funding.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::with_bootstrap_advancement;
use std::num::NonZeroUsize;
use worth_execution::ExecutionAllocationPolicy;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
fn policy() -> ExecutionRequestPolicy {
    ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, Some(8 * 1024 * 1024), Some(100)),
    )
}
#[test]
fn original_delivery_storage_keeps_funding_until_each_owner_drops() {
    with_bootstrap_advancement(policy(), |phase| {
        let policy = phase.execution_request().allocation_policy();
        let ExecutionAllocationPolicy::Execution(lease) = policy else {
            panic!("the owner phase lends its actual allocation lease")
        };
        let baseline = lease.charged_memory_bytes();
        let (publisher, receiver) = OriginalCompletionPublisher::pending_slot(policy)
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
    })
    .unwrap();
}
#[test]
fn original_delivery_storage_refuses_without_room_and_returns_its_reservations() {
    with_bootstrap_advancement(policy(), |phase| {
        let policy = phase.execution_request().allocation_policy();
        let ExecutionAllocationPolicy::Execution(lease) = policy else {
            panic!("the owner phase lends its actual allocation lease")
        };
        let original = lease.charged_memory_bytes();
        let slot_bytes =
            (std::mem::size_of::<PendingOriginalSlot>() + 2 * std::mem::size_of::<usize>()) as u64;
        let available = lease
            .available_memory_bytes()
            .expect("the phase declares a finite memory ceiling");
        let held = lease
            .reserve_memory(
                available
                    .checked_sub(slot_bytes)
                    .expect("the healthy phase admits slot backing"),
            )
            .unwrap();
        let baseline = lease.charged_memory_bytes();
        let cause = match OriginalCompletionPublisher::pending_slot(policy) {
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
        drop(held);
        assert_eq!(lease.charged_memory_bytes(), original);
    })
    .unwrap();
}
