use std::num::NonZeroUsize;

use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, SignalConditionalRetentionDenial,
    SignalConditionalRetentionReservation,
};
use crate::runtime_policy::{SignalConditionalTemporalBudget, SignalRuntimePolicy};

use super::retention::partition_charge;
use super::tests::claimed_service_with_policy;
use super::SignalConditionalTemporalPartitionDenial as Denial;

pub(super) fn policy(partitions: usize, wakes: usize) -> SignalRuntimePolicy {
    SignalRuntimePolicy::development().with_conditional_temporal_budget(
        SignalConditionalTemporalBudget {
            maximum_live_partitions: partitions,
            maximum_reserved_active_wakes: wakes,
        },
    )
}

#[test]
fn temporal_quotas_are_aggregate_and_distinct_from_evaluation_slots() {
    for requested in [policy(1, 8), policy(8, 3)] {
        let (_runtime, service) = claimed_service_with_policy(requested);
        let owner = service.owner.upgrade().unwrap();
        let ledger = &owner.conditional_retention;
        let baseline = ledger.usage();
        let evaluation = ledger
            .reserve(
                requested
                    .conditional_evaluation_budget
                    .maximum_retained_slots,
                Charge::ZERO,
            )
            .unwrap();
        let first = service
            .admit_temporal_partition(NonZeroUsize::new(2).unwrap())
            .unwrap();
        assert_eq!(ledger.temporal_usage(), (1, 2));
        let before = ledger.usage();
        assert!(matches!(
            service.admit_temporal_partition(NonZeroUsize::new(2).unwrap()),
            Err(Denial::RetentionCapacityExhausted)
        ));
        assert_eq!(ledger.usage(), before);
        assert_eq!(owner.conditional_temporal.live_count(), 1);
        first.retire().unwrap();
        assert_eq!(ledger.temporal_usage(), (0, 0));
        drop(evaluation);
        assert_eq!(ledger.usage(), baseline);
        service
            .admit_temporal_partition(NonZeroUsize::new(2).unwrap())
            .unwrap()
            .retire()
            .unwrap();
        assert_eq!(ledger.usage(), baseline);
    }
}

#[test]
fn temporal_byte_denial_preserves_registry_and_identity() {
    let requested = policy(2, 8);
    let (_runtime, service) = claimed_service_with_policy(requested);
    let owner = service.owner.upgrade().unwrap();
    let ledger = &owner.conditional_retention;
    let baseline = ledger.usage();
    let handle = std::mem::size_of::<SignalConditionalRetentionReservation>() as u64;
    let partition_bytes = partition_charge(2).unwrap().bytes() + handle;
    let ballast = ledger
        .reserve(
            0,
            Charge::from_bytes(
                requested
                    .conditional_evaluation_budget
                    .maximum_retained_bytes
                    - baseline.1
                    - (partition_bytes - 1)
                    - handle,
            ),
        )
        .unwrap();
    let before = ledger.usage();
    assert!(matches!(
        service.admit_temporal_partition(NonZeroUsize::new(2).unwrap()),
        Err(Denial::RetentionCapacityExhausted)
    ));
    assert_eq!(ledger.usage(), before);
    assert_eq!(ledger.temporal_usage(), (0, 0));
    assert_eq!(owner.conditional_temporal.live_count(), 0);
    drop(ballast);
    let partition = service
        .admit_temporal_partition(NonZeroUsize::new(2).unwrap())
        .unwrap();
    assert_eq!(partition.id.ordinal(), 0);
    assert_eq!(ledger.usage(), (baseline.0, baseline.1 + partition_bytes));
    partition.retire().unwrap();
    assert_eq!(ledger.usage(), baseline);
}

#[test]
fn temporal_storage_overflow_is_non_mutating() {
    let (_runtime, service) = claimed_service_with_policy(policy(usize::MAX, usize::MAX));
    let owner = service.owner.upgrade().unwrap();
    let before = owner.conditional_retention.usage();
    assert!(matches!(
        service.admit_temporal_partition(NonZeroUsize::new(usize::MAX).unwrap()),
        Err(Denial::StorageChargeOverflow)
    ));
    assert_eq!(owner.conditional_retention.usage(), before);
    assert_eq!(owner.conditional_retention.temporal_usage(), (0, 0));
    assert_eq!(owner.conditional_temporal.live_count(), 0);
}

#[test]
fn temporal_close_fences_new_custody_but_keeps_weak_backing_and_held_cell_charged() {
    let (runtime, service) = claimed_service_with_policy(policy(1, 2));
    let owner = service.owner.upgrade().unwrap();
    let ledger = owner.conditional_retention.clone();
    let partition = service
        .admit_temporal_partition(NonZeroUsize::new(2).unwrap())
        .unwrap();
    let held_cell = partition.cell.upgrade().unwrap();
    owner.close().unwrap();
    assert_eq!(owner.conditional_temporal.live_count(), 0);
    let after_close = ledger.usage();
    assert_eq!(ledger.temporal_usage(), (1, 2));
    assert!(matches!(
        ledger.reserve_temporal(0, 0, Charge::ZERO),
        Err(SignalConditionalRetentionDenial::Closed)
    ));
    assert!(matches!(
        service.admit_temporal_partition(NonZeroUsize::new(1).unwrap()),
        Err(Denial::OwnerAdmission(_))
    ));
    drop(partition);
    assert_eq!(ledger.temporal_usage(), (1, 2));
    assert_eq!(ledger.usage(), after_close);
    drop(held_cell);
    assert_eq!(ledger.temporal_usage(), (0, 0));
    assert_eq!(
        ledger.usage().1,
        after_close.1
            - partition_charge(2).unwrap().bytes()
            - std::mem::size_of::<SignalConditionalRetentionReservation>() as u64
    );
    drop(service);
    drop(owner);
    drop(runtime);
    assert_eq!(ledger.usage(), (0, 0));
}
