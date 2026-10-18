//! Cold construction is the independent oracle for changed native shapes.
use super::*;
use crate::{CancellationToken, ExecutionResourceLease, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
fn ids(values: &[u64]) -> CanonicalUniqueVec<PartitionIdentity> {
    CanonicalUniqueVec::try_from_sorted_unique(
        values
            .iter()
            .map(|id| PartitionIdentity::new(*id))
            .collect(),
    )
    .unwrap()
}
fn cold(
    values: &[u64],
) -> (
    ReductionTree<u64, fn(&u64, &u64) -> u64>,
    super::super::ReductionMetrics,
) {
    fn sum(a: &u64, b: &u64) -> u64 {
        a + b
    }
    ReductionTree::try_from_declared(
        super::super::super::plan::ReductionPlan::from_canonical(ids(values)),
        values
            .iter()
            .map(|id| (PartitionIdentity::new(*id), *id))
            .collect(),
        0,
        sum as fn(&u64, &u64) -> u64,
    )
    .unwrap()
}
fn tree(values: &[u64]) -> ReductionTree<u64, fn(&u64, &u64) -> u64> {
    cold(values).0
}
fn lease(bytes: u64, cancellation: CancellationToken) -> ExecutionResourceLease<'static> {
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, bytes, 1_000_000),
        ),
        cancellation,
        deadline: None,
    })
    .unwrap()
}
#[test]
fn changed_shape_cost_matches_cold_priority_history_and_invokes_no_reducer() {
    let prior = tree(&[1, 3, 5, 7, 9]);
    let mut history = prior.clone();
    history.insert(PartitionIdentity::new(2), 2).unwrap();
    history.delete(PartitionIdentity::new(2)).unwrap();
    let lease = lease(1_000_000, CancellationToken::new());
    for (removed, inserted, next) in [
        (&[3][..], &[2, 6][..], &[1, 2, 5, 6, 7, 9][..]),
        (&[1, 3, 5, 7, 9][..], &[][..], &[][..]),
        (&[][..], &[2][..], &[1, 2, 3, 5, 7, 9][..]),
    ] {
        let predicted = prior
            .checked_full_build_cost_after_edits(
                &ids(removed),
                &ids(inserted),
                ExecutionAllocationPolicy::Execution(&lease),
            )
            .unwrap();
        assert_eq!(
            predicted,
            history
                .checked_full_build_cost_after_edits(
                    &ids(removed),
                    &ids(inserted),
                    ExecutionAllocationPolicy::Execution(&lease)
                )
                .unwrap()
        );
        let (_, actual) = cold(next);
        assert_eq!(predicted, (actual.charged_work, actual.charged_span));
        assert_eq!(lease.charged_memory_bytes(), 0, "temporary paths released");
        assert_eq!(prior.result(), history.result());
    }
    let panic_combine = ReductionTree {
        root: prior.root.clone(),
        identity: 0,
        combine: |_: &u64, _: &u64| -> u64 { panic!("shape must not combine") },
        len: prior.len,
    };
    assert!(panic_combine
        .checked_full_build_cost_after_edits(
            &ids(&[3]),
            &ids(&[4]),
            ExecutionAllocationPolicy::SystemAllocation
        )
        .is_ok());
}
#[test]
fn shape_refusal_preserves_prior_and_original_interruption_without_fallback() {
    let prior = tree(&[1, 3, 5]);
    let tiny = lease(1, CancellationToken::new());
    assert!(matches!(
        prior.checked_full_build_cost_after_edits(
            &ids(&[]),
            &ids(&[2]),
            ExecutionAllocationPolicy::Execution(&tiny)
        ),
        Err(ReductionShapeCostDenial::Allocation(_))
    ));
    assert_eq!(tiny.charged_memory_bytes(), 0);
    let source = crate::CancellationSource::new();
    let stopped = lease(1_000_000, source.token());
    source.cancel();
    assert!(matches!(
        prior.checked_full_build_cost_after_edits(
            &ids(&[1]),
            &ids(&[]),
            ExecutionAllocationPolicy::Execution(&stopped)
        ),
        Err(ReductionShapeCostDenial::Allocation(cause)) if cause.classify()==crate::ExecutionAllocationDenialClass::Request(crate::WorkCeilingDenial::Stopped(crate::MapKernelStop::Cancelled))
    ));
    assert_eq!(prior.result(), &9);
    assert!(matches!(
        prior.checked_full_build_cost_after_edits(
            &ids(&[8]),
            &ids(&[]),
            ExecutionAllocationPolicy::SystemAllocation
        ),
        Err(ReductionShapeCostDenial::Structural(
            ReductionDenial::UnknownIdentity(_)
        ))
    ));
    assert!(matches!(
        prior.checked_full_build_cost_after_edits(
            &ids(&[]),
            &ids(&[1]),
            ExecutionAllocationPolicy::SystemAllocation
        ),
        Err(ReductionShapeCostDenial::Structural(
            ReductionDenial::IdentityAlreadyPresent(_)
        ))
    ));
    assert!(prior
        .checked_full_build_cost_after_edits(
            &ids(&[1]),
            &ids(&[2]),
            ExecutionAllocationPolicy::SystemAllocation
        )
        .is_ok());
}
