use std::num::NonZeroUsize;
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::data::temporal::{
    ClockAdvanceRequest, ClockDomain, ClockTick, TemporalCondition, TemporalWakeId,
    TemporalWakeRetirementReason,
};

use super::promotion_fault::{
    SignalTemporalPromotionBoundary as Boundary, SignalTemporalPromotionFault,
    SignalTemporalPromotionFaultAction as Action,
};
use super::retention_tests::policy;
use super::tests::claimed_service_with_policy;
use super::{
    SignalConditionalTemporalPartition as Partition,
    SignalConditionalTemporalPartitionDenial as Denial,
};

#[test]
fn temporal_promotion_divergence_quarantines_only_the_affected_partition() {
    for boundary in [Boundary::AfterClockAdvance, Boundary::AfterFirstPromotion] {
        quarantine_court(boundary, Action::Diverge);
    }
}

#[test]
fn temporal_promotion_unwind_quarantines_only_the_affected_partition() {
    for boundary in [Boundary::AfterClockAdvance, Boundary::AfterFirstPromotion] {
        quarantine_court(boundary, Action::Unwind);
    }
}

fn quarantine_court(boundary: Boundary, action: Action) {
    let (runtime, service) = claimed_service_with_policy(policy(2, 3));
    let owner = service.owner.upgrade().unwrap();
    let ledger = owner.conditional_retention.clone();
    let baseline = ledger.usage();
    let failed = service
        .admit_temporal_partition(NonZeroUsize::new(2).unwrap())
        .unwrap();
    let failed_bytes = ledger.usage().1 - baseline.1;
    let independent = service
        .admit_temporal_partition(NonZeroUsize::new(1).unwrap())
        .unwrap();
    let condition = TemporalCondition::at_or_after(ClockTick::new(5));
    let first = failed
        .schedule_temporal_wake(condition.clone(), ClockTick::new(5))
        .unwrap();
    failed
        .schedule_temporal_wake(condition, ClockTick::new(5))
        .unwrap();
    failed
        .with_admitted_partition(|state| {
            state.promotion_fault = Some(SignalTemporalPromotionFault { boundary, action });
            Ok(())
        })
        .unwrap();
    let before_fault = ledger.usage();
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        failed.advance_clock_and_promote_due_temporal_wakes_ready_bounded(
            ClockAdvanceRequest::new(ClockDomain::MonotonicExecution, ClockTick::new(5)),
            NonZeroUsize::new(2).unwrap(),
        )
    }));
    match action {
        Action::Diverge => assert!(matches!(outcome, Ok(Err(Denial::PartitionQuarantined)))),
        Action::Unwind => assert!(outcome.is_err()),
    }
    assert_eq!(
        ledger.usage(),
        before_fault,
        "failed output staging custody is released"
    );
    assert_eq!(ledger.temporal_usage(), (2, 3));

    // Inspect actual state even after mutex poisoning; production never recovers
    // that lock. The clock/ready counts prove the fault reached its named effect.
    let held_cell = failed.cell.upgrade().unwrap();
    {
        let state = held_cell
            .state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        assert!(state.quarantined);
        assert!(state.promotion_fault.is_none());
        assert_eq!(
            state.temporal.clock_basis().current_tick(),
            ClockTick::new(5)
        );
        let expected_ready = usize::from(boundary == Boundary::AfterFirstPromotion) as u64;
        let summary = state.temporal.wake_summary();
        assert_eq!(summary.ready_count(), expected_ready);
        assert_eq!(summary.scheduled_count(), 2 - expected_ready);
    }
    assert_partition_fenced(&failed, first.id());
    let independent_advance = independent
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(7),
        ))
        .unwrap();
    assert_eq!(independent_advance.next_tick(), ClockTick::new(7));
    assert_eq!(
        independent
            .temporal_wake_summary()
            .unwrap()
            .scheduled_count(),
        0
    );
    assert_eq!(ledger.usage(), before_fault);

    failed.retire().unwrap();
    assert_eq!(owner.conditional_temporal.live_count(), 1);
    assert_eq!(
        ledger.usage(),
        before_fault,
        "held failed cell still owns its storage"
    );
    drop(held_cell);
    assert_eq!(ledger.temporal_usage(), (1, 1));
    assert_eq!(
        ledger.usage(),
        (before_fault.0, before_fault.1 - failed_bytes)
    );
    independent.retire().unwrap();
    assert_eq!(ledger.usage(), baseline);
    drop(service);
    drop(owner);
    drop(runtime);
    assert_eq!(ledger.usage(), (0, 0));
}

fn assert_partition_fenced(partition: &Partition<(), (), ()>, wake: TemporalWakeId) {
    let condition = TemporalCondition::at_or_after(ClockTick::new(9));
    assert!(matches!(
        partition.advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(9),
        )),
        Err(Denial::PartitionQuarantined)
    ));
    assert!(matches!(
        partition.schedule_temporal_wake(condition.clone(), ClockTick::new(9)),
        Err(Denial::PartitionQuarantined)
    ));
    assert!(matches!(
        partition.supersede_temporal_wake(wake, condition, ClockTick::new(9)),
        Err(Denial::PartitionQuarantined)
    ));
    assert!(matches!(
        partition.retire_temporal_wake(wake, TemporalWakeRetirementReason::Consumed),
        Err(Denial::PartitionQuarantined)
    ));
    assert!(matches!(
        partition.promote_due_temporal_wakes_ready_bounded(NonZeroUsize::new(1).unwrap()),
        Err(Denial::PartitionQuarantined)
    ));
    let retained = partition.temporal_wake_summary().unwrap();
    assert_eq!(retained.scheduled_count() + retained.ready_count(), 2);
    // Inspection did not clear the execution fence.
    assert!(matches!(
        partition.promote_due_temporal_wakes_ready_bounded(NonZeroUsize::new(1).unwrap()),
        Err(Denial::PartitionQuarantined)
    ));
}
