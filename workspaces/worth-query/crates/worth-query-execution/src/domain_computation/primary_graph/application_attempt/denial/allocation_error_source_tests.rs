//! Native allocation evidence remains inspectable through the standard error chain.
use super::*;
use std::error::Error;

#[test]
fn allocation_cause_keeps_its_error_source_type() {
    let owner =
        crate::domain_computation::primary_graph::application_contribution::test_authority();
    let lease = owner
        .request_lease(worth_execution::LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, 0, 1),
            ),
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap();
    let cause = worth_execution::ExecutionByteBuffer::allocate(
        1,
        worth_execution::ExecutionAllocationPolicy::Execution(&lease),
    )
    .unwrap_err();
    let denial = WorthQueryApplicationAttemptDenial::allocation_denied("attempt", cause);
    let source = denial
        .source()
        .unwrap()
        .downcast_ref::<worth_execution::ExecutionAllocationDenial>()
        .unwrap();
    assert!(std::ptr::eq(source, denial.allocation_denial().unwrap()));
    // One u8 payload has Layout size1; the unused zero-byte policy admits0.
    assert_eq!(
        source.kind(),
        worth_execution::ExecutionAllocationDenialKind::Lease(
            worth_execution::LeaseDenial::MemoryExhausted(worth_execution::MemoryLimitDenial {
                requested: std::mem::size_of::<u8>() as u64,
                admitted: 0,
                level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 },
            }),
        )
    );
}
