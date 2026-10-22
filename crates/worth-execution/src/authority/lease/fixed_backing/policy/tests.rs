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

#[test]
fn allocation_policy_binds_authentic_descendant_funding_and_builder_controls() {
    let request = || LeaseRequest {
        policy: ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            NonZeroUsize::MIN,
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    };
    let parent = ExecutionResourceLease::serial(request()).unwrap();
    let foreign = ExecutionResourceLease::serial(request()).unwrap();
    let stopped = CancellationSource::new();
    let child = parent
        .child(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, Some(100_000), None),
            ),
            cancellation: stopped.token(),
            deadline: None,
        })
        .unwrap();
    let allocation = ExecutionAllocationPolicy::Execution(&parent);
    let outcome = crate::ExecutionRequest::leased(&child).run_native(|_| {
        let before = child.charged_memory_bytes();
        let explicit_parent = parent.reserve_memory(200_000).unwrap();
        assert_eq!(
            child.charged_memory_bytes(),
            before,
            "raw parent allocation retains its explicit owner"
        );
        drop(explicit_parent);
        let denial = allocation.reserve_memory(200_000).unwrap_err();
        assert!(matches!(
            denial.kind(),
            ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(_))
        ));
        assert_eq!(child.charged_memory_bytes(), before);

        let held = allocation.reserve_memory(8).unwrap().unwrap();
        assert_eq!(child.charged_memory_bytes(), before + 8);
        drop(held);
        assert_eq!(child.charged_memory_bytes(), before);
        let unrelated = ExecutionAllocationPolicy::Execution(&foreign);
        assert_eq!(
            unrelated.reserve_memory(8).unwrap_err().kind(),
            ExecutionAllocationDenialKind::Lease(LeaseDenial::UnrelatedNestedLease)
        );
        assert_eq!(
            unrelated.check_live().unwrap_err().kind(),
            ExecutionAllocationDenialKind::Lease(LeaseDenial::UnrelatedNestedLease)
        );

        let mut builder = crate::ExecutionArrayBuilder::<u64>::allocate(1, allocation).unwrap();
        assert_eq!(child.charged_memory_bytes(), before + 8);
        stopped.cancel();
        assert_eq!(
            allocation.check_live().unwrap_err().kind(),
            ExecutionAllocationDenialKind::Cancelled
        );
        assert_eq!(
            builder.push(10).unwrap_err().kind(),
            ExecutionAllocationDenialKind::Cancelled,
            "the builder retains its actual child's controls"
        );
        drop(builder);
        assert_eq!(child.charged_memory_bytes(), before);
    });
    assert_eq!(
        outcome.unwrap_err(),
        crate::WorkCeilingDenial::Stopped(crate::MapKernelStop::Cancelled)
    );
    assert_eq!(child.charged_memory_bytes(), 0);
    assert_eq!(parent.charged_memory_bytes(), 0);
    allocation.check_live().unwrap();
    assert!(
        allocation.reserve_memory(200_000).unwrap().is_none(),
        "outside the active child, the original worker-only policy remains native"
    );
}
