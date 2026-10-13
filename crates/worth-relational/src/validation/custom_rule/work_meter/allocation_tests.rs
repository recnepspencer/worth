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
        let meter = CustomPreparationBudget::new(lease.status(), Some(100), Some(12));
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
    let budget = CustomPreparationBudget::new(lease.status(), Some(100), Some(0));
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

#[test]
fn dynamic_structural_claims_and_peaks_use_the_actual_request_lease() {
    use crate::execution::RelationalExecutionDenialCause;
    use crate::execution::{InvariantPacketAllocations, PacketBudgetDenial};
    use crate::validation::data::CustomInvariantLeaseBudget;

    for refuse_peak in [false, true] {
        let lease = crate::tests::support::test_execution_lease();
        let allocations = InvariantPacketAllocations::new(&lease).unwrap();
        let admission = allocations.scratch();
        let budget = CustomPreparationBudget::with_scratch_admission(
            lease.status(),
            Some(100),
            admission.clone(),
        );
        let work = CustomInvariantWorkMeter::new(NonZeroU64::new(100).unwrap()).with_lease(&budget);
        let geometry_bytes = 12 * 1024 * 1024;
        assert!(work.claim_scratch(geometry_bytes));
        assert!(work.check_scratch_peak(1024 * 1024));
        assert_eq!(budget.claimed_memory(), geometry_bytes);
        assert!(
            work.try_charge(1),
            "structural reads use the same admission"
        );
        let admitted_memory = budget.claimed_memory();
        let refused = if refuse_peak {
            work.check_scratch_peak(64 * 1024 * 1024)
        } else {
            work.claim_scratch(64 * 1024 * 1024)
        };
        assert!(!refused, "growth beyond the real 32 MiB lease is refused");
        assert_eq!(budget.claimed_memory(), admitted_memory);
        assert_eq!(budget.stop(), Some(CustomPreparationStop::MemoryExhausted));
        let failure = admission
            .failure()
            .expect("owner retains the typed refusal");
        assert!(
            matches!(
                failure,
                PacketBudgetDenial::ReadRequest(
                    RelationalExecutionDenialCause::PolicyMemoryExhausted {
                        requested, admitted, ..
                    }
                ) if requested > admitted
            ),
            "unexpected allocation refusal: {failure:?}"
        );
        assert!(!work.claim_scratch(1));
        let PacketBudgetDenial::ReadRequest(original_cause) = failure else {
            unreachable!("the allocation refusal was checked above")
        };
        assert!(
            matches!(admission.failure(), Some(PacketBudgetDenial::ReadRequest(latched)) if latched == original_cause)
        );
        assert_eq!(budget.claimed_memory(), admitted_memory);
    }
}
