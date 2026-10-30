// Separate process because the physical authority is constructed once per process.
#[allow(unused_imports)]
use crate::process_zero_memory::isolated_context::*;
use std::num::NonZeroUsize;
use {
    worth_execution::CancellationToken, worth_execution::ExecutionAllocationDenialKind,
    worth_execution::ExecutionAllocationPolicy, worth_execution::ExecutionAuthority,
    worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionByteBuffer,
    worth_execution::LeaseDenial, worth_execution::LeaseRequest,
    worth_execution::MemoryLimitDenial, worth_execution::MemoryLimitLevel,
};
use {
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
};
#[test]
fn explicit_zero_process_memory_preserves_zero_and_refuses_positive_payloads() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let authority = worth_execution::ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: std::num::NonZeroUsize::new(1).unwrap(),
        charged_memory_bytes: Some(0),
    })
    .unwrap();
    assert!(matches!(
        authority.request_lease(request(1)),
        Err(worth_execution::LeaseDenial::MemoryLimitExceedsParent)
    ));
    let lease = authority.request_lease(request(0)).unwrap();
    let bytes = worth_execution::ExecutionByteBuffer::allocate(
        0,
        worth_execution::ExecutionAllocationPolicy::Execution(&lease),
    )
    .unwrap()
    .seal()
    .unwrap();
    assert!(bytes.is_empty());
    assert_eq!(bytes.charged_payload_bytes(), Some(0));
    let denied = worth_execution::ExecutionByteBuffer::allocate(
        1,
        worth_execution::ExecutionAllocationPolicy::Execution(&lease),
    )
    .unwrap_err();
    assert_eq!(
        denied.kind(),
        worth_execution::ExecutionAllocationDenialKind::Lease(
            worth_execution::LeaseDenial::MemoryExhausted(
                worth_execution::MemoryLimitDenial::Capacity {
                    requested: 1,
                    admitted: 0,
                    level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
                }
            )
        )
    );
    assert_eq!(denied.requested_payload_bytes(), Some(1));
}
