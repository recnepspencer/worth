use std::num::NonZeroUsize;

use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, SignalConditionalRetentionReservation,
};
use crate::data::temporal::{
    BoundedTemporalReadyPromotionSummary, ClockAdvanceRequest, ClockDomain, ClockTick,
    ReadyTemporalWake, TemporalCondition,
};

use super::retention_tests::policy;
use super::tests::claimed_service_with_policy;
use super::SignalConditionalTemporalPartitionDenial as Denial;

#[test]
fn held_promotion_custody_denies_before_clock_or_frontier_effects_and_drop_restores_capacity() {
    let requested = policy(1, 2);
    let (_runtime, service) = claimed_service_with_policy(requested);
    let owner = service.owner.upgrade().unwrap();
    let ledger = &owner.conditional_retention;
    let maximum = NonZeroUsize::new(1).unwrap();
    let partition = service
        .admit_temporal_partition(NonZeroUsize::new(2).unwrap())
        .unwrap();
    for tick in [3, 8] {
        partition
            .schedule_temporal_wake(
                TemporalCondition::at_or_after(ClockTick::new(tick)),
                ClockTick::new(tick),
            )
            .unwrap();
    }
    let handle = std::mem::size_of::<SignalConditionalRetentionReservation>() as u64;
    let output_bytes = (std::mem::size_of::<BoundedTemporalReadyPromotionSummary>()
        + std::mem::size_of::<ReadyTemporalWake>()) as u64
        + handle;
    let ballast = ledger
        .reserve(
            0,
            Charge::from_bytes(
                requested
                    .conditional_evaluation_budget
                    .maximum_retained_bytes
                    - ledger.usage().1
                    - output_bytes
                    - handle,
            ),
        )
        .unwrap();
    let (_, first) = partition
        .advance_clock_and_promote_due_temporal_wakes_ready_bounded(
            ClockAdvanceRequest::new(ClockDomain::MonotonicExecution, ClockTick::new(3)),
            maximum,
        )
        .unwrap();
    assert_eq!(first.promotion().promoted_wake_count(), 1);
    assert_eq!(
        ledger.usage().1,
        requested
            .conditional_evaluation_budget
            .maximum_retained_bytes
    );
    let before = partition
        .with_admitted_partition(|state| {
            Ok((
                state.temporal.clock_basis(),
                state.temporal.frontier_snapshot(),
                state.temporal.wake_summary(),
            ))
        })
        .unwrap();
    assert!(matches!(
        partition.advance_clock_and_promote_due_temporal_wakes_ready_bounded(
            ClockAdvanceRequest::new(ClockDomain::MonotonicExecution, ClockTick::new(8)),
            maximum,
        ),
        Err(Denial::RetentionCapacityExhausted)
    ));
    assert!(matches!(
        partition.promote_due_temporal_wakes_ready_bounded(maximum),
        Err(Denial::RetentionCapacityExhausted)
    ));
    let after = partition
        .with_admitted_partition(|state| {
            Ok((
                state.temporal.clock_basis(),
                state.temporal.frontier_snapshot(),
                state.temporal.wake_summary(),
            ))
        })
        .unwrap();
    assert_eq!(after, before);
    drop(first);
    let (_, second) = partition
        .advance_clock_and_promote_due_temporal_wakes_ready_bounded(
            ClockAdvanceRequest::new(ClockDomain::MonotonicExecution, ClockTick::new(8)),
            maximum,
        )
        .unwrap();
    assert_eq!(second.promotion().promoted_wake_count(), 1);
    assert!(!second.due_work_remaining());
    drop(second);
    drop(ballast);
}

#[test]
fn promotion_result_keeps_exact_bytes_after_partition_and_owner_are_gone() {
    let (runtime, service) = claimed_service_with_policy(policy(1, 2));
    let owner = service.owner.upgrade().unwrap();
    let ledger = owner.conditional_retention.clone();
    let partition = service
        .admit_temporal_partition(NonZeroUsize::new(2).unwrap())
        .unwrap();
    partition
        .schedule_temporal_wake(
            TemporalCondition::at_or_after(ClockTick::new(3)),
            ClockTick::new(3),
        )
        .unwrap();
    let (_, result) = partition
        .advance_clock_and_promote_due_temporal_wakes_ready_bounded(
            ClockAdvanceRequest::new(ClockDomain::MonotonicExecution, ClockTick::new(3)),
            NonZeroUsize::new(2).unwrap(),
        )
        .unwrap();
    partition.retire().unwrap();
    assert_eq!(ledger.temporal_usage(), (0, 0));
    owner.close().unwrap();
    drop(service);
    drop(owner);
    drop(runtime);
    assert_eq!(
        ledger.usage(),
        (
            0,
            (std::mem::size_of::<BoundedTemporalReadyPromotionSummary>()
                + std::mem::size_of::<ReadyTemporalWake>()
                + std::mem::size_of::<SignalConditionalRetentionReservation>()) as u64
        )
    );
    assert_eq!(result.promotion().ready_wakes().len(), 1);
    drop(result);
    assert_eq!(ledger.usage(), (0, 0));
}
