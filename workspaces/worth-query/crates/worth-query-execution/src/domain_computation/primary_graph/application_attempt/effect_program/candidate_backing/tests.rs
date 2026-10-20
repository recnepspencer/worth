use super::*;
use worth_execution::{CancellationToken, ExecutionResourceLease, LeaseRequest, MemoryLimitDenial};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

fn lease(memory: Option<u64>) -> ExecutionResourceLease<'static> {
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, memory, None),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .unwrap()
}

#[test]
fn actual_backing_reserves_original_parent_beside_siblings_and_releases_after_payload() {
    let parent = lease(Some(1024));
    let sibling = parent.reserve_memory(13).unwrap();
    struct Payload<'a>(&'a ExecutionResourceLease<'static>);
    impl Drop for Payload<'_> {
        fn drop(&mut self) {
            assert!(self.0.charged_memory_bytes() >= 13 + size_of::<Payload<'_>>() as u64);
        }
    }
    let (mut values, ticket) = allocate::<Payload<'_>>(
        1,
        ExecutionAllocationPolicy::Execution(&parent),
        &crate::domain_computation::primary_graph::tests::fixture::live_scope(),
    )
    .unwrap();
    values.push(Payload(&parent));
    assert_eq!(
        ticket.as_ref().unwrap().memory.bytes(),
        size_of::<Payload<'_>>() as u64
    );
    drop(values);
    drop(ticket);
    assert_eq!(parent.charged_memory_bytes(), 13);
    drop(sibling);
    assert_eq!(parent.charged_memory_bytes(), 0);
}

#[test]
fn finite_request_refuses_actual_backing_without_consuming_sibling_credit() {
    let parent = lease(Some(13));
    let sibling = parent.reserve_memory(5).unwrap();
    let denied = allocate::<u64>(
        2,
        ExecutionAllocationPolicy::Execution(&parent),
        &crate::domain_computation::primary_graph::tests::fixture::live_scope(),
    )
    .unwrap_err();
    assert!(matches!(
        denied.candidate_memory_denial(),
        Some(MemoryLimitDenial::Capacity {
            requested: 16,
            admitted: 8,
            ..
        })
    ));
    assert_eq!(parent.charged_memory_bytes(), 5);
    drop(sibling);
    assert_eq!(parent.charged_memory_bytes(), 0);
}

#[test]
fn checked_layout_overflow_and_cancellation_precede_allocation() {
    let parent = lease(None);
    let denied = allocate::<u64>(
        usize::MAX,
        ExecutionAllocationPolicy::Execution(&parent),
        &crate::domain_computation::primary_graph::tests::fixture::live_scope(),
    )
    .unwrap_err();
    assert_eq!(denied.kind(), Kind::CandidateCapacityExceeded);
    assert_eq!(parent.charged_memory_bytes(), 0);
    let cancellation = worth_execution::CancellationSource::new();
    let child = parent.controlled_child(cancellation.token(), None);
    cancellation.cancel();
    let denied = allocate::<u64>(
        2,
        ExecutionAllocationPolicy::Execution(&child),
        &crate::domain_computation::primary_graph::tests::fixture::live_scope(),
    )
    .unwrap_err();
    assert_eq!(
        denied.allocation_denial().unwrap().kind(),
        worth_execution::ExecutionAllocationDenialKind::Cancelled
    );
    assert_eq!(parent.charged_memory_bytes(), 0);
}

#[test]
fn growth_funds_real_coexistence_on_the_same_original_ticket() {
    let parent = lease(Some(32));
    let (mut values, mut ticket) = allocate::<u64>(
        1,
        ExecutionAllocationPolicy::Execution(&parent),
        &crate::domain_computation::primary_graph::tests::fixture::live_scope(),
    )
    .unwrap();
    values.push(7);
    extend(
        &mut values,
        &mut ticket,
        vec![11, 13],
        &crate::domain_computation::primary_graph::tests::fixture::live_scope(),
    )
    .unwrap();
    assert_eq!(values, [7, 11, 13]);
    assert_eq!(ticket.as_ref().unwrap().memory.bytes(), 24);
    let denied = extend(
        &mut values,
        &mut ticket,
        vec![17],
        &crate::domain_computation::primary_graph::tests::fixture::live_scope(),
    )
    .unwrap_err();
    assert!(matches!(
        denied.candidate_memory_denial(),
        Some(MemoryLimitDenial::Capacity { .. })
    ));
    assert_eq!(values, [7, 11, 13]);
    assert_eq!(ticket.as_ref().unwrap().memory.bytes(), 24);
    drop(values);
    drop(ticket);
    assert_eq!(parent.charged_memory_bytes(), 0);
}

#[test]
fn cancelled_original_lineage_refuses_growth_with_payload_and_credit_intact() {
    let parent = lease(None);
    let cancellation = worth_execution::CancellationSource::new();
    let original = parent.controlled_child(cancellation.token(), None);
    let (mut values, mut ticket) = allocate::<u64>(
        1,
        ExecutionAllocationPolicy::Execution(&original),
        &crate::domain_computation::primary_graph::tests::fixture::live_scope(),
    )
    .unwrap();
    values.push(7);
    cancellation.cancel();
    let denied = extend(
        &mut values,
        &mut ticket,
        vec![11],
        &crate::domain_computation::primary_graph::tests::fixture::live_scope(),
    )
    .unwrap_err();
    assert_eq!(denied.kind(), Kind::Cancelled);
    assert_eq!(values, [7]);
    assert_eq!(ticket.as_ref().unwrap().memory.bytes(), 8);
    assert_eq!(parent.charged_memory_bytes(), 8);
    drop(values);
    drop(ticket);
    assert_eq!(parent.charged_memory_bytes(), 0);
}

#[test]
fn original_system_request_cancellation_and_deadline_precede_backing_layout() {
    use worth_query_admission::facade::authenticated_principal::{
        WorthQueryCancellationSource, WorthQueryRequestScope,
    };
    let cancellation = WorthQueryCancellationSource::new();
    let cancelled = WorthQueryRequestScope::new(None, cancellation.token());
    cancellation.cancel();
    let deadline = WorthQueryRequestScope::new(
        Some(std::time::Instant::now()),
        WorthQueryCancellationSource::new().token(),
    );
    for (request, expected) in [
        (cancelled, Kind::Cancelled),
        (deadline, Kind::DeadlineExceeded),
    ] {
        let denied = allocate::<u64>(
            usize::MAX,
            ExecutionAllocationPolicy::SystemAllocation,
            &request,
        )
        .unwrap_err();
        assert_eq!(
            denied.kind(),
            expected,
            "the original stop must precede even an overflowing backing layout"
        );
    }
}
