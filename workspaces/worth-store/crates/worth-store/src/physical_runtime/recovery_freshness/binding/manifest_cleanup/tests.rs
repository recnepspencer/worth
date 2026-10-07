use super::*;
use worth_store_physical_format::PersistedRecordIdentity;
use worth_store_wal::{LogSequenceNumber, WalLsnRange};

// Native funding is proved by the sampler owner, not these local vectors.
fn prepared(count: usize) -> ManifestCleanupObservations {
    ManifestCleanupObservations::prepare(Vec::with_capacity(count), 16_384).unwrap()
}

fn interval(start: u64) -> WalLsnRange {
    WalLsnRange::new(
        LogSequenceNumber::new(start),
        LogSequenceNumber::new(start + 1),
    )
    .unwrap()
}

fn intent(sha: u8) -> BlobManifestResidueCleanup {
    worth_store_physical_format::BlobManifestResidueCleanupV1::intent(
        [1; 16],
        [2; 16],
        PersistedRecordIdentity::new([3; 16], 4).unwrap(),
        [sha; 32],
        [5; 32],
        [6; 32],
        [7; 32],
        8,
        9,
        [10; 32],
        11,
        12,
    )
    .unwrap()
    .into()
}

#[test]
fn observed_intent_and_completion_are_ordered_and_exact() {
    let mut sample = prepared(2);
    let pointer = sample.frames.as_ptr();
    let first_intent = intent(4);
    sample
        .observe(interval(20), first_intent, [1; 16], 2)
        .unwrap();
    sample
        .observe(interval(22), first_intent.completed(), [1; 16], 2)
        .unwrap();
    let (frames, peak) = sample.finish().unwrap();
    assert_eq!(frames.len(), 2);
    assert!(frames[0].2 && frames[1].2);
    assert_eq!(frames.as_ptr(), pointer);
    assert_eq!(
        peak,
        (frames.capacity() * std::mem::size_of::<Observation>()) as u64
    );
}

#[test]
fn covered_intent_may_leave_only_a_completion_in_the_tail() {
    let mut sample = prepared(1);
    sample
        .observe(interval(22), intent(4).completed(), [1; 16], 1)
        .unwrap();
    let (frames, _) = sample.finish().unwrap();
    assert_eq!(frames.len(), 1);
    assert!(!frames[0].2);
}

#[test]
fn duplicates_conflicts_foreign_store_and_reversed_order_are_denied() {
    let mut sample = prepared(2);
    let first_intent = intent(4);
    assert_eq!(
        sample.observe(interval(20), first_intent, [9; 16], 2),
        Err(StoreRecoveryBindingSampleDenial::InvalidWalMember)
    );
    sample
        .observe(interval(20), first_intent, [1; 16], 2)
        .unwrap();
    sample
        .observe(interval(21), intent(5).completed(), [1; 16], 2)
        .unwrap();
    assert_eq!(
        sample.finish().err(),
        Some(StoreRecoveryBindingSampleDenial::InvalidWalMember)
    );
    let mut reversed = prepared(2);
    reversed
        .observe(interval(20), first_intent.completed(), [1; 16], 2)
        .unwrap();
    reversed
        .observe(interval(21), first_intent, [1; 16], 2)
        .unwrap();
    assert_eq!(
        reversed.finish().err(),
        Some(StoreRecoveryBindingSampleDenial::InvalidWalMember)
    );
}

#[test]
fn prepared_capacity_is_bounded_and_observation_does_not_grow_it() {
    assert!(matches!(
        ManifestCleanupObservations::prepare(
            Vec::with_capacity(1),
            std::mem::size_of::<Observation>() as u64 - 1,
        ),
        Err(StoreRecoveryBindingSampleDenial::RecoveryMemoryLimit)
    ));
    let mut sample = prepared(0);
    assert_eq!(
        sample.observe(interval(20), intent(4), [1; 16], 1),
        Err(StoreRecoveryBindingSampleDenial::RecoveryMemoryLimit)
    );
    assert_eq!(sample.frames.capacity(), 0);
}

#[test]
fn duplicate_pairs_and_three_record_attempts_are_rejected() {
    let first = intent(4);
    let mut duplicate = prepared(2);
    duplicate.observe(interval(20), first, [1; 16], 2).unwrap();
    duplicate.observe(interval(22), first, [1; 16], 2).unwrap();
    assert_eq!(
        duplicate.finish().err(),
        Some(StoreRecoveryBindingSampleDenial::InvalidWalMember)
    );
    let mut triple = prepared(3);
    triple.observe(interval(20), first, [1; 16], 3).unwrap();
    triple
        .observe(interval(22), first.completed(), [1; 16], 3)
        .unwrap();
    triple
        .observe(interval(24), first.completed(), [1; 16], 3)
        .unwrap();
    assert_eq!(
        triple.finish().err(),
        Some(StoreRecoveryBindingSampleDenial::InvalidWalMember)
    );
}

#[test]
fn singleton_intent_remains_unpaired_and_limits_precede_capacity_failure() {
    let first = intent(4);
    let mut sample = prepared(1);
    let pointer = sample.frames.as_ptr();
    sample.observe(interval(20), first, [1; 16], 1).unwrap();
    assert_eq!(
        sample.observe(interval(22), first.completed(), [1; 16], 1),
        Err(StoreRecoveryBindingSampleDenial::OperationBindingLimit)
    );
    assert_eq!(
        sample.observe(interval(22), first.completed(), [9; 16], 2),
        Err(StoreRecoveryBindingSampleDenial::InvalidWalMember)
    );
    assert_eq!(
        sample.observe(interval(19), first.completed(), [1; 16], 2),
        Err(StoreRecoveryBindingSampleDenial::InvalidWalMember)
    );
    assert_eq!(
        sample.observe(interval(22), first.completed(), [1; 16], 2),
        Err(StoreRecoveryBindingSampleDenial::RecoveryMemoryLimit)
    );
    let (frames, _) = sample.finish().unwrap();
    assert_eq!(frames.len(), 1);
    assert!(!frames[0].2);
    assert_eq!(frames.as_ptr(), pointer);
    assert_eq!(frames.capacity(), 1);
}
