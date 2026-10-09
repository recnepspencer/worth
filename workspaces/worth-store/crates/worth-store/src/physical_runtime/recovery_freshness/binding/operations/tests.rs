use std::num::NonZeroU64;

use super::super::{
    failure::sample_failure_from_evidence, StoreRecoveryBindingFreshness,
    StoreRecoveryOperationFate as Fate,
};
use super::*;
use crate::physical_runtime::instance::PhysicalResidencyOwner;

mod fixture;
use fixture::{evidence, fingerprint, world};

#[test]
fn fixed_index_resolves_full_collisions_and_sorts_without_growing_evidence() {
    let (fixture, owner) = world();
    let capacity = RecoveryBindingOperationsCapacity::for_records(3).unwrap();
    let expected = 3 * std::mem::size_of::<StoreRecoveryOperationEvidence>() as u64
        + 8 * std::mem::size_of::<Option<usize>>() as u64;
    assert_eq!(capacity.requested_bytes(), expected);
    let grant = grant(&owner, expected);
    let mut operations = RecoveryBindingOperations::prepare(capacity, 3, &grant).unwrap();
    operations.forced_start = Some(7);
    let initial = (operations.evidence.capacity(), operations.slots.capacity());
    let pointer = operations.evidence.as_ptr();
    let first = evidence(&fixture, 1);
    let second = evidence(&fixture, 2);
    let third = evidence(&fixture, 3);
    let mut expected_keys = [
        first.idempotency_identity,
        second.idempotency_identity,
        third.idempotency_identity,
    ];
    operations.merge(third.clone()).unwrap();
    assert_eq!(operations.last_probe_count, 1);
    operations.merge(first.clone()).unwrap();
    assert_eq!(operations.last_probe_count, 2);
    operations.merge(second.clone()).unwrap();
    assert_eq!(operations.last_probe_count, 3);
    operations.merge(second).unwrap();
    assert_eq!(operations.last_probe_count, 3);
    assert!(operations.last_probe_count <= operations.slots.len());
    assert_eq!(
        (operations.evidence.capacity(), operations.slots.capacity()),
        initial
    );
    assert_eq!(operations.evidence.as_ptr(), pointer);
    let retained = operations.into_evidence();
    expected_keys.sort_unstable();
    assert_eq!(
        retained
            .iter()
            .map(|item| item.idempotency_identity)
            .collect::<Vec<_>>(),
        expected_keys
    );
    assert_eq!(retained.capacity(), initial.0);
    assert_eq!(retained.as_ptr(), pointer);
    drop(retained);
    drop(grant);
    assert_eq!(owner.ports().counters().active_operation_bytes(), 0);
    drop(owner);
    fixture.media.close();
}

#[test]
fn arrival_order_retains_first_equal_fate_and_promotes_only_indeterminate() {
    let (fixture, owner) = world();
    let source = evidence(&fixture, 4);
    for terminal_first in [false, true] {
        let capacity = RecoveryBindingOperationsCapacity::for_records(4).unwrap();
        let grant = grant(&owner, capacity.requested_bytes());
        let mut operations = RecoveryBindingOperations::prepare(capacity, 1, &grant).unwrap();
        let mut terminal = source.clone();
        terminal.fate = Fate::AcknowledgedDurable;
        terminal.attempt_binding_identity = None;
        let mut wal_x = source.clone();
        wal_x.attempt_binding_identity = Some([7; 32]);
        let mut wal_y = source.clone();
        wal_y.attempt_binding_identity = Some([11; 32]);
        if terminal_first {
            operations.merge(terminal.clone()).unwrap();
            operations.merge(wal_x).unwrap();
            operations.merge(wal_y).unwrap();
        } else {
            operations.merge(wal_x).unwrap();
            assert_eq!(
                operations.merge(wal_y),
                Err(MergeDenial::ConflictingOperationEvidence)
            );
            assert_eq!(
                operations.evidence()[0].attempt_binding_identity,
                Some([7; 32])
            );
            operations.merge(terminal.clone()).unwrap();
        }
        assert_eq!(operations.evidence(), &[terminal.clone()]);
        let mut conflict = terminal.clone();
        conflict.fate = Fate::ProvenNoEffect;
        assert_eq!(
            operations.merge(conflict),
            Err(MergeDenial::ConflictingOperationEvidence)
        );
        assert_eq!(operations.evidence(), &[terminal]);
        drop(operations);
        drop(grant);
    }
    drop(owner);
    fixture.media.close();
}

#[test]
fn duplicate_limit_and_conflict_keep_the_exact_prefix_and_failure_census() {
    let (fixture, owner) = world();
    let capacity = RecoveryBindingOperationsCapacity::for_records(4).unwrap();
    let grant = grant(&owner, capacity.requested_bytes());
    let mut operations = RecoveryBindingOperations::prepare(capacity, 2, &grant).unwrap();
    operations.forced_start = Some(0);
    let retained = evidence(&fixture, 5);
    let mut expired = evidence(&fixture, 6);
    expired.freshness = StoreRecoveryBindingFreshness::ExpiredAtSelectedCheckpoint;
    operations.merge(retained.clone()).unwrap();
    operations.merge(expired.clone()).unwrap();
    let before = operations.evidence().to_vec();
    operations.merge(retained.clone()).unwrap();
    assert_eq!(
        operations.merge(evidence(&fixture, 7)),
        Err(MergeDenial::OperationBindingLimit)
    );
    assert_eq!(operations.evidence(), before);
    let failure = sample_failure_from_evidence(
        MergeDenial::OperationBindingLimit,
        operations.evidence().iter(),
        3,
        17,
    );
    assert_eq!(failure.operation_bindings_observed(), 3);
    assert_eq!(
        (failure.freshness_retained(), failure.freshness_expired()),
        (1, 1)
    );
    assert_eq!(
        (
            failure.wal_members_observed(),
            failure.redo_bytes_observed()
        ),
        (3, 17)
    );
    let mut conflict = expired;
    conflict.request_fingerprint = fingerprint(&fixture, 23);
    assert_eq!(
        operations.merge(conflict),
        Err(MergeDenial::ConflictingOperationEvidence)
    );
    assert_eq!(operations.evidence(), before);
    let failure = sample_failure_from_evidence(
        MergeDenial::ConflictingOperationEvidence,
        operations.evidence().iter(),
        0,
        0,
    );
    assert_eq!(failure.operation_bindings_observed(), 2);
    assert_eq!(
        (failure.freshness_retained(), failure.freshness_expired()),
        (1, 1)
    );
    drop(operations);
    drop(grant);
    drop(owner);
    fixture.media.close();
}

#[test]
fn zero_overflow_and_full_record_capacity_do_not_trigger_growth() {
    let zero = RecoveryBindingOperationsCapacity::for_records(0).unwrap();
    assert_eq!(zero.requested_bytes(), 0);
    assert!(matches!(
        RecoveryBindingOperationsCapacity::for_records(u64::MAX),
        Err(AllocationDenial::SizeOverflow)
    ));
    assert!(matches!(
        RecoveryBindingOperationsCapacity::for_records(isize::MAX as u64),
        Err(AllocationDenial::SizeOverflow)
    ));
    let (fixture, owner) = world();
    let mut empty = RecoveryBindingOperations::empty(0);
    assert_eq!(
        empty.merge(evidence(&fixture, 8)),
        Err(MergeDenial::OperationBindingLimit)
    );
    assert_eq!((empty.evidence.capacity(), empty.slots.capacity()), (0, 0));
    let capacity = RecoveryBindingOperationsCapacity::for_records(1).unwrap();
    let grant = grant(&owner, capacity.requested_bytes());
    let mut operations = RecoveryBindingOperations::prepare(capacity, 2, &grant).unwrap();
    operations.merge(evidence(&fixture, 9)).unwrap();
    let pointer = operations.evidence.as_ptr();
    assert_eq!(
        operations.merge(evidence(&fixture, 10)),
        Err(MergeDenial::InvalidCheckpointBinding)
    );
    assert_eq!(operations.evidence.len(), 1);
    assert_eq!(operations.evidence.capacity(), 1);
    assert_eq!(operations.evidence.as_ptr(), pointer);
    drop(operations);
    drop(grant);
    drop(owner);
    fixture.media.close();
}

#[test]
fn prepare_requires_real_sufficient_recovery_backing_before_vector_reservation() {
    let (fixture, owner) = world();
    let ports = owner.ports();
    let capacity = RecoveryBindingOperationsCapacity::for_records(2).unwrap();
    let requested = capacity.requested_bytes();
    let short = grant(&owner, requested - 1);
    assert!(matches!(
        RecoveryBindingOperations::prepare(capacity, 2, &short),
        Err(AllocationDenial::BackingMismatch)
    ));
    let wrong = ports
        .begin_operation(
            PhysicalOperationAllocationScope::Maintenance,
            NonZeroU64::new(requested).unwrap(),
        )
        .unwrap();
    let capacity = RecoveryBindingOperationsCapacity::for_records(2).unwrap();
    assert!(matches!(
        RecoveryBindingOperations::prepare(capacity, 2, &wrong),
        Err(AllocationDenial::BackingMismatch)
    ));
    drop(short);
    drop(wrong);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
    drop(owner);
    fixture.media.close();
}

fn grant(owner: &PhysicalResidencyOwner, bytes: u64) -> OperationAllocationGrant {
    owner
        .ports()
        .begin_operation(
            PhysicalOperationAllocationScope::Recovery,
            NonZeroU64::new(bytes).unwrap(),
        )
        .unwrap()
}
