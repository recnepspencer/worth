//! Scratch and retained plan extents share the actual request-memory boundary.
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
    let quote = CustomPreparationBudget::tracked(&quote_lease).unwrap();
    let fixed = quote_lease.charged_memory_bytes();
    drop(quote);
    drop(quote_lease);
    for under in [false, true] {
        let lease = authority
            .request_lease(request(fixed + 2 * EXTENT - u64::from(under)))
            .unwrap();
        let meter = CustomPreparationBudget::tracked(&lease).unwrap();
        let work = crate::validation::custom_rule::CustomInvariantWorkMeter::new(Some(
            NonZeroU64::new(100_000).unwrap(),
        ))
        .with_lease(&meter);
        assert!(work.try_claim_bytes(EXTENT), "the scratch extent fits");
        let ceiling = lease.available_memory_bytes().unwrap();
        assert_eq!(ceiling, EXTENT - u64::from(under));
        assert_eq!(work.try_claim_bytes(EXTENT), !under);
        if under {
            assert!(
                matches!(meter.stop(), Some(CustomPreparationStop::MemoryAdmission(
                worth_execution::MemoryLimitDenial::Capacity { requested, admitted, .. }
            )) if requested == fixed + 2 * EXTENT && admitted == fixed + 2 * EXTENT - 1)
            );
        } else {
            assert_eq!(meter.stop(), None);
        }
    }
}
