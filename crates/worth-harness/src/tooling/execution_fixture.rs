//! Accounted execution fixtures for unit-test binaries.

use std::{num::NonZeroUsize, sync::OnceLock};
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig,
    ExecutionResourceLease, LeaseRequest, SerialMemoryBudget, SerialRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

static TEST_EXECUTION_AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();

pub fn test_execution_authority() -> &'static ExecutionAuthority {
    TEST_EXECUTION_AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(8).expect("positive test worker count"),
            charged_memory_bytes: Some(512 * 1024 * 1024),
        })
        .expect("one execution fixture authority per test binary")
    })
}

pub fn test_execution_lease() -> ExecutionResourceLease<'static> {
    test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(
                    NonZeroUsize::new(4).expect("positive test worker count"),
                    None,
                    Some(1_000_000),
                ),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .expect("test execution lease admitted")
}

pub fn serial_request() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    )
}

pub fn serial_request_with_memory(memory: SerialMemoryBudget) -> SerialRequest {
    SerialRequest::from_memory(memory, CancellationSource::new().token(), None)
}

pub fn retained_byte_owner_bill(bytes: &[u8]) -> u64 {
    let budget = worth_execution::SerialMemoryBudget::new(Some(
        (bytes.len() as u64).checked_add(64 * 1024).unwrap(),
    ));
    let host = serial_request_with_memory(budget.clone());
    let request = worth_execution::ExecutionRequest::serial(&host);
    let mut building =
        worth_execution::ExecutionRequestArrayBuilder::<u8>::allocate(request, bytes.len())
            .unwrap();
    building.extend_from_slice(bytes).unwrap();
    let owned = building.seal_immutable_bytes().unwrap();
    assert_eq!(owned.charged_payload_bytes(), Some(bytes.len() as u64));
    let total = budget.charged_memory_bytes();
    assert!(total > bytes.len() as u64);
    drop(owned);
    assert_eq!(budget.charged_memory_bytes(), 0);
    total
}
