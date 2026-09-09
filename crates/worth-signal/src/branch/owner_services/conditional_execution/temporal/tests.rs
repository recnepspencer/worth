use std::num::NonZeroUsize;
use std::sync::mpsc;
use std::time::Duration;

use crate::branch::owner_services::SignalOwnerCancellationSource;
use crate::data::aspect::SignalAspectLoweringOwner;
use crate::data::graph::SignalGraph;
use crate::data::temporal::{
    ClockAdvanceRequest, ClockDomain, ClockTick, TemporalCondition, TemporalWakeRetirementReason,
};
use crate::logic::transaction::SignalRuntime;

use super::super::SignalConditionalExecutionPort;
use super::SignalConditionalTemporalPartitionDenial as Denial;

fn claimed_service() -> (
    SignalRuntime<(), (), (), ()>,
    SignalConditionalExecutionPort<(), (), ()>,
) {
    claimed_service_with_policy(crate::runtime_policy::SignalRuntimePolicy::development())
}

pub(super) fn claimed_service_with_policy(
    policy: crate::runtime_policy::SignalRuntimePolicy,
) -> (
    SignalRuntime<(), (), (), ()>,
    SignalConditionalExecutionPort<(), (), ()>,
) {
    let claimant = SignalAspectLoweringOwner::fresh();
    let mut graph = SignalGraph::new();
    graph.claim_aspect_lowering_owner(&claimant).unwrap();
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    runtime.set_runtime_policy(policy);
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    runtime.owner_port_slots().unwrap();
    let source = worth_proof::ConditionalSourceObservationOwner::fresh();
    let service = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source.authority())
        .unwrap();
    (runtime, service)
}

#[test]
fn partition_issuance_requires_current_service_basis_but_admitted_partition_stays_pinned() {
    let (mut runtime, service) = claimed_service();
    let partition = service
        .admit_temporal_partition(NonZeroUsize::new(2).unwrap())
        .unwrap();
    let owner = service.owner.upgrade().unwrap();
    assert_eq!(
        partition.owner_runtime_instance_id(),
        owner.runtime_instance_id()
    );
    assert_eq!(
        partition.issuance_basis().admission_identity(),
        service.issuance_basis().admission_identity()
    );
    let (_, mutation, _) = runtime.owner_port_slots().unwrap();
    let cancellation = SignalOwnerCancellationSource::new();
    let (_, successor) = mutation
        .capture_exact(service.issuance_basis(), &cancellation.token())
        .unwrap()
        .into_parts();
    assert!(matches!(
        service.admit_temporal_partition(NonZeroUsize::new(2).unwrap()),
        Err(Denial::ServiceAdmission(_))
    ));
    partition
        .schedule_temporal_wake(
            TemporalCondition::at_or_after(ClockTick::new(5)),
            ClockTick::new(5),
        )
        .unwrap();
    assert_eq!(
        partition.temporal_wake_summary().unwrap().scheduled_count(),
        1
    );
    let successor_service = service.reissue_for_successor_basis(&successor).unwrap();
    let successor_partition = successor_service
        .admit_temporal_partition(NonZeroUsize::new(2).unwrap())
        .unwrap();
    assert_eq!(
        successor_partition
            .temporal_wake_summary()
            .unwrap()
            .scheduled_count(),
        0
    );
    assert_eq!(owner.conditional_temporal.live_count(), 2);
}

#[test]
fn partition_retirement_drop_and_owner_close_release_registered_resources() {
    let (runtime, service) = claimed_service();
    let owner = service.owner.upgrade().unwrap();
    let retired = service
        .admit_temporal_partition(NonZeroUsize::new(1).unwrap())
        .unwrap();
    let dropped = service
        .admit_temporal_partition(NonZeroUsize::new(1).unwrap())
        .unwrap();
    let closed = service
        .admit_temporal_partition(NonZeroUsize::new(1).unwrap())
        .unwrap();
    assert_eq!(owner.conditional_temporal.live_count(), 3);
    retired.retire().unwrap();
    drop(dropped);
    assert_eq!(owner.conditional_temporal.live_count(), 1);
    closed
        .schedule_temporal_wake(
            TemporalCondition::at_or_after(ClockTick::new(5)),
            ClockTick::new(5),
        )
        .unwrap();
    owner.close().unwrap();
    assert_eq!(owner.conditional_temporal.live_count(), 0);
    assert!(closed.cell.upgrade().is_none());
    assert!(matches!(
        closed.temporal_wake_summary(),
        Err(Denial::OwnerAdmission(_))
    ));
    drop(owner);
    drop(runtime);
    assert!(matches!(
        closed.temporal_wake_summary(),
        Err(Denial::OwnerUnavailable(_))
    ));
}

#[test]
fn bounded_wake_lifecycle_reclaims_capacity_and_keeps_no_retired_history() {
    let (_runtime, service) = claimed_service();
    let partition = service
        .admit_temporal_partition(NonZeroUsize::new(2).unwrap())
        .unwrap();
    let condition = TemporalCondition::at_or_after(ClockTick::new(5));
    let first = partition
        .schedule_temporal_wake(condition.clone(), ClockTick::new(5))
        .unwrap();
    let mut second = partition
        .schedule_temporal_wake(condition.clone(), ClockTick::new(5))
        .unwrap();
    assert!(matches!(
        partition.schedule_temporal_wake(condition.clone(), ClockTick::new(5)),
        Err(Denial::ActiveWakeCapacityExhausted {
            maximum_active_wakes: 2
        })
    ));
    for _ in 0..128 {
        second = partition
            .supersede_temporal_wake(second.id(), condition.clone(), ClockTick::new(5))
            .unwrap()
            .scheduled()
            .clone();
        let summary = partition.temporal_wake_summary().unwrap();
        assert_eq!((summary.scheduled_count(), summary.retired_count()), (2, 0));
    }
    partition
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(5),
        ))
        .unwrap();
    let first_batch = partition
        .promote_due_temporal_wakes_ready_bounded(NonZeroUsize::new(1).unwrap())
        .unwrap();
    assert_eq!(first_batch.promotion().ready_wakes()[0].id(), first.id());
    assert_eq!(first_batch.promotion().promoted_wake_count(), 1);
    assert!(first_batch.due_work_remaining());
    assert!(matches!(
        partition.schedule_temporal_wake(condition.clone(), ClockTick::new(5)),
        Err(Denial::ActiveWakeCapacityExhausted { .. })
    ));
    partition
        .retire_temporal_wake(first.id(), TemporalWakeRetirementReason::Consumed)
        .unwrap();
    let second_batch = partition
        .promote_due_temporal_wakes_ready_bounded(NonZeroUsize::new(1).unwrap())
        .unwrap();
    assert_eq!(second_batch.promotion().ready_wakes()[0].id(), second.id());
    assert!(!second_batch.due_work_remaining());
    partition
        .schedule_temporal_wake(condition, ClockTick::new(5))
        .unwrap();
    let summary = partition.temporal_wake_summary().unwrap();
    assert_eq!(
        (
            summary.scheduled_count(),
            summary.ready_count(),
            summary.retired_count()
        ),
        (1, 1, 0)
    );
}

#[test]
fn parked_partition_does_not_hold_another_partition_or_registry() {
    let (_runtime, service) = claimed_service();
    let parked = service
        .admit_temporal_partition(NonZeroUsize::new(2).unwrap())
        .unwrap();
    let independent = service
        .admit_temporal_partition(NonZeroUsize::new(2).unwrap())
        .unwrap();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (completed_tx, completed_rx) = mpsc::channel();
    std::thread::scope(|scope| {
        let parked_ref = &parked;
        let parked_task = scope.spawn(move || {
            parked_ref.with_admitted_partition(|_| {
                entered_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                Ok(())
            })
        });
        entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let independent_task = scope.spawn(|| {
            independent
                .advance_clock(ClockAdvanceRequest::new(
                    ClockDomain::MonotonicExecution,
                    ClockTick::new(8),
                ))
                .unwrap();
            let third = service
                .admit_temporal_partition(NonZeroUsize::new(1).unwrap())
                .unwrap();
            third.retire().unwrap();
            completed_tx.send(()).unwrap();
        });
        let completed = completed_rx.recv_timeout(Duration::from_secs(5));
        release_tx.send(()).unwrap();
        parked_task.join().unwrap().unwrap();
        independent_task.join().unwrap();
        completed.expect(
            "independent partition and registration complete while first partition is parked",
        );
    });
    // Advancing the independent clock must not move this partition's basis.
    parked
        .schedule_temporal_wake(
            TemporalCondition::at_or_after(ClockTick::new(1)),
            ClockTick::new(1),
        )
        .unwrap();
    assert_eq!(
        independent
            .temporal_wake_summary()
            .unwrap()
            .scheduled_count(),
        0
    );
}
