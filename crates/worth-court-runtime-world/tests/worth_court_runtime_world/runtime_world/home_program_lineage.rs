//! Program windows are DATA; these tests grant no live Home authority.
use worth_execution::ExecutionRequest;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::{
    CanonicalDigestId, DurableWorldOwnerIdentity, HomeProgramRevisionBoundary,
};
use worth_foundational::facade::{
    DurableCompositePublicationAttemptIdentity, HomeIdentity, WorldOwnerSlotIdentity,
};
use worth_runtime_world::facade::{WorldHomeProgramLineage, WorldReadmissionDenial as Denial};
fn owner(slot: u64) -> DurableWorldOwnerIdentity {
    DurableWorldOwnerIdentity::from_recorded(
        HomeIdentity::from_observed_bytes([1; 16]).unwrap(),
        WorldOwnerSlotIdentity::from_recorded(slot),
    )
}
fn row(before: u8, after: u8, through: u64) -> HomeProgramRevisionBoundary {
    HomeProgramRevisionBoundary::from_recorded(
        CanonicalDigestId::new([before; 32]),
        CanonicalDigestId::new([after; 32]),
        DurableCompositePublicationAttemptIdentity::from_recorded(owner(1), through),
    )
}
#[test]
fn original_attempt_labels_change_only_after_their_recorded_transition_boundary() {
    let budget = SerialMemoryBudget::new(Some(1 << 20));
    let policy =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&policy);
    let lineage = WorldHomeProgramLineage::from_recorded(
        owner(1),
        CanonicalDigestId::new([1; 32]),
        CanonicalDigestId::new([3; 32]),
        [row(1, 2, 5), row(2, 3, 9)].into_iter(),
        &request,
    )
    .unwrap();
    assert!(budget.charged_memory_bytes() > 0);
    for (attempt, revision) in [(0, 1), (5, 1), (6, 2), (9, 2), (10, 3)] {
        assert!(lineage
            .matches_recorded_attempt([revision; 32], attempt, &request)
            .unwrap());
        assert!(!lineage
            .matches_recorded_attempt([4; 32], attempt, &request)
            .unwrap());
    }
    assert!(!lineage
        .matches_recorded_attempt([2; 32], 5, &request)
        .unwrap());
    assert!(!lineage
        .matches_recorded_attempt([1; 32], 6, &request)
        .unwrap());
    assert!(!lineage
        .matches_recorded_attempt([3; 32], 9, &request)
        .unwrap());
    assert!(!lineage
        .matches_recorded_attempt([2; 32], 10, &request)
        .unwrap());
    drop(lineage);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
#[test]
fn recorded_revision_windows_refuse_foreign_disconnected_or_reordered_boundaries() {
    let policy = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 20)),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&policy);
    for rows in [
        vec![row(1, 2, 5), row(1, 3, 9)],
        vec![row(1, 2, 5), row(2, 3, 5)],
        vec![row(1, 2, 9), row(2, 3, 5)],
    ] {
        assert!(matches!(
            WorldHomeProgramLineage::from_recorded(
                owner(1),
                CanonicalDigestId::new([1; 32]),
                CanonicalDigestId::new([3; 32]),
                rows.into_iter(),
                &request
            ),
            Err(Denial::ProgramRevisionMismatch)
        ));
    }
    assert!(matches!(
        WorldHomeProgramLineage::from_recorded(
            owner(2),
            CanonicalDigestId::new([1; 32]),
            CanonicalDigestId::new([2; 32]),
            [row(1, 2, 5)].into_iter(),
            &request
        ),
        Err(Denial::ProgramRevisionMismatch)
    ));
}
