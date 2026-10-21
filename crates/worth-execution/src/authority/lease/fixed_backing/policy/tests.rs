//! Genuine policy reservations, independent of any physical backing author.
use super::*;
use crate::{CancellationSource, CancellationToken, ExecutionResourceLease, LeaseRequest};
use std::{
    num::NonZeroUsize,
    time::{Duration, Instant},
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn owned_payload_reservation_keeps_real_funding_and_typed_refusal_until_drop() {
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, Some(128), Some(100)),
    );
    let parent = ExecutionResourceLease::serial(LeaseRequest {
        policy,
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .unwrap();
    let allocation = ExecutionAllocationPolicy::Execution(&parent);
    let before = parent.charged_memory_bytes();
    let held = allocation.reserve_memory(128).unwrap().unwrap();
    assert_eq!(held.bytes(), 128);
    assert_eq!(parent.charged_memory_bytes(), before + 128);
    let denied = allocation.reserve_memory(1).unwrap_err();
    assert!(matches!(
        denied.kind(),
        ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(_))
    ));
    assert_eq!(denied.requested_payload_bytes(), Some(1));
    let stopped = CancellationSource::new();
    let child = parent.controlled_child(
        stopped.token(),
        Some(Instant::now() - Duration::from_secs(1)),
    );
    let elapsed = ExecutionAllocationPolicy::Execution(&child)
        .reserve_memory(1)
        .unwrap_err();
    assert_eq!(
        elapsed.kind(),
        ExecutionAllocationDenialKind::DeadlineElapsed
    );
    stopped.cancel();
    let cancelled = ExecutionAllocationPolicy::Execution(&child)
        .reserve_memory(1)
        .unwrap_err();
    assert_eq!(
        cancelled.kind(),
        ExecutionAllocationDenialKind::Cancelled,
        "real interruption precedes exhausted memory/deadline"
    );
    assert_eq!(cancelled.requested_payload_bytes(), Some(1));
    drop(held);
    assert_eq!(parent.charged_memory_bytes(), before);
    assert_eq!(
        allocation.reserve_memory(128).unwrap().unwrap().bytes(),
        128,
        "uncancelled lawful parent retry"
    );
    assert!(!parent
        .controlled_child(CancellationToken::new(), None)
        .is_cancelled());
    assert!(ExecutionAllocationPolicy::SystemAllocation
        .reserve_memory(u64::MAX)
        .unwrap()
        .is_none());
}
