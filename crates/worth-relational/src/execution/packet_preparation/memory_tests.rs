//! The artifact is built directly in its declared retained allocation.
use super::*;
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn an_artifact_reserves_its_declared_allocation_once() {
    const ALLOCATION: u64 = 20_000;
    let authority = crate::tests::support::test_execution_authority();
    let policy = |bytes| {
        ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, bytes, Some(100_000)),
        )
    };
    let request = |bytes| LeaseRequest {
        policy: policy(bytes),
        deadline: None,
        cancellation: CancellationToken::new(),
    };
    let quote_lease = authority.request_lease(request(8 * 1024 * 1024)).unwrap();
    let scan = crate::execution::admit_ordered_scan(vec![(PartitionIdentity::new(1), ())]).unwrap();
    let required = scan.declared_memory_requirement_for_lease::<(), PreparedArtifact<Vec<u8>>, super::super::PacketBudgetDenial>(
        &quote_lease, 0, ALLOCATION, 0, 0).unwrap();
    drop(quote_lease);
    for under in [false, true] {
        let lease = authority
            .request_lease(request(required - u64::from(under)))
            .unwrap();
        let mut began = false;
        let result = prepare_borrowed_artifact(&lease, None, ALLOCATION, |budget| {
            began = true;
            budget.claim(ALLOCATION)?;
            let mut buffer: Vec<u8> = Vec::new();
            buffer.try_reserve_exact(ALLOCATION as usize).unwrap();
            Ok((buffer, ALLOCATION))
        });
        if under {
            assert!(!began, "one byte under refuses before allocating");
            assert!(matches!(
                result,
                Err(PacketExecutionStop::Execution {
                    reason: worth_execution::MapStop::Admission(
                        worth_execution::LeaseDenial::MemoryExhausted(_)
                    ),
                    ..
                })
            ));
        } else {
            assert_eq!(result.unwrap().capacity(), ALLOCATION as usize);
            assert!(began);
        }
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}
