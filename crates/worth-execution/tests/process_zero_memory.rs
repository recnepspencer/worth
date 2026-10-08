//! Separate process because the physical authority is constructed once per process.
use std::num::NonZeroUsize;
use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig,
    ExecutionByteAllocationDenialKind, ExecutionByteAllocationPolicy, ExecutionByteBuffer,
    LeaseDenial, LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
fn request(bytes: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), bytes, 1),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}
#[test]
fn explicit_zero_process_memory_preserves_zero_and_refuses_positive_payloads() {
    let authority = ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(1).unwrap(),
        charged_memory_bytes: Some(0),
    })
    .unwrap();
    assert!(matches!(
        authority.request_lease(request(1)),
        Err(LeaseDenial::MemoryLimitExceedsParent)
    ));
    let lease = authority.request_lease(request(0)).unwrap();
    let bytes = ExecutionByteBuffer::allocate(0, ExecutionByteAllocationPolicy::Execution(&lease))
        .unwrap()
        .seal()
        .unwrap();
    assert!(bytes.is_empty());
    assert_eq!(bytes.charged_payload_bytes(), Some(0));
    let denied = ExecutionByteBuffer::allocate(1, ExecutionByteAllocationPolicy::Execution(&lease))
        .unwrap_err();
    assert_eq!(
        denied.kind(),
        ExecutionByteAllocationDenialKind::Lease(LeaseDenial::ResourceExhausted)
    );
    assert_eq!(denied.requested_payload_bytes(), Some(1));
}
