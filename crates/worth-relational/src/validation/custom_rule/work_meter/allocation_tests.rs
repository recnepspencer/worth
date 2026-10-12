//! Read-only planning work and retained allocations are separate declarations.
use super::*;
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn planning_visits_claim_only_the_allocation_they_declare() {
    for posture in [ExecutionPosture::Serial, ExecutionPosture::Automatic] {
        let lease = crate::tests::support::test_execution_authority()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    posture,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(NonZeroUsize::MIN, 1024, 100),
                ),
                cancellation: CancellationToken::new(),
                deadline: None,
            })
            .unwrap();
        let meter = CustomPreparationBudget::new(lease.status(), 100, 12);
        assert!(
            meter.try_visit(20),
            "a borrowed visit retains no allocation"
        );
        assert_eq!(meter.claimed_memory(), 0);
        assert!(meter.try_plan_item(12));
        assert_eq!(meter.claimed_memory(), 12);
        assert_eq!(meter.charged_work(), 20 + 1);
        assert!(
            !meter.try_plan_item(1),
            "the next actual allocation is unfunded"
        );
        assert_eq!(meter.stop(), Some(CustomPreparationStop::MemoryExhausted));
        assert_eq!(meter.claimed_memory(), 12);
    }
}

#[test]
fn authored_checkpoints_do_not_allocate_structural_nodes() {
    use crate::validation::data::CustomInvariantLeaseBudget;
    let lease = crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, 1024, 100),
            ),
            cancellation: CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    let budget = CustomPreparationBudget::new(lease.status(), 100, 0);
    let work = CustomInvariantWorkMeter::new(NonZeroU64::new(100).unwrap()).with_lease(&budget);
    assert!(
        work.checkpoint(20),
        "twenty authored arithmetic units have no heap allocation"
    );
    assert_eq!(budget.claimed_memory(), 0);
    assert_eq!(budget.charged_work(), 20);
    assert!(!work.claim_scratch(1));
    assert_eq!(budget.stop(), Some(CustomPreparationStop::MemoryExhausted));
}
