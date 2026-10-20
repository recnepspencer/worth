//! The planning allocation declaration leaves room for both live extents.
use super::*;
use std::num::{NonZeroU64, NonZeroUsize};
use worth_execution::{CancellationToken, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn the_checked_plan_ceiling_has_an_exact_scratch_and_retention_boundary() {
    const EXTENT: u64 = 20_000;
    let authority = crate::tests::support::test_execution_authority();
    let request = |bytes| LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, Some(bytes), Some(100_000)),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    };
    let quote_lease = authority.request_lease(request(8 * 1024 * 1024)).unwrap();
    let scan = crate::execution::admit_ordered_scan(vec![(PartitionIdentity::new(1), ())]).unwrap();
    let fixed = scan
        .declared_memory_requirement_for_lease::<(), CheckedPlan<'_>, PacketBudgetDenial>(
            &quote_lease,
            0,
            0,
            0,
            0,
        )
        .unwrap();
    drop(quote_lease);
    for under in [false, true] {
        let lease = authority
            .request_lease(request(fixed + 2 * EXTENT - u64::from(under)))
            .unwrap();
        let ceiling = preparation_ceiling(&scan, &lease);
        assert_eq!(ceiling, EXTENT - u64::from(under));
        let meter = CustomPreparationBudget::new(lease.status(), Some(100_000), Some(ceiling));
        let work = crate::validation::custom_rule::CustomInvariantWorkMeter::new(Some(
            NonZeroU64::new(100_000).unwrap(),
        ))
        .with_lease(&meter);
        assert_eq!(work.try_claim_bytes(EXTENT), !under);
        assert_eq!(
            meter.stop(),
            under.then_some(CustomPreparationStop::MemoryExhausted)
        );
    }
}
