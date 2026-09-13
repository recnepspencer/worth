use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{self, TryRecvError};
use std::sync::Arc;
use std::thread;

use super::super::lifecycle_state::{
    SignalOwnerCloseDenial, SignalOwnerMetadataHoldDenial,
    MAXIMUM_IN_FLIGHT_SIGNAL_OWNER_OPERATIONS,
};
use super::super::{
    SignalOwnerAdmissionDenial, SignalOwnerLifecycleObservation,
    SignalOwnerLifecyclePoisonRecovery, SignalOwnerLifecycleState, SignalOwnerServiceCounters,
};
use super::progress_bound::{observe_within, PROGRESS_BOUND};

#[test]
fn close_drains_admitted_work_and_monotonically_denies_late_admission() {
    let counters = Arc::new(SignalOwnerServiceCounters::default());
    let lifecycle = SignalOwnerLifecycleState::new(41, Arc::clone(&counters));
    let admission = lifecycle.admit(41).expect("open owner admits work");
    let (closed_sender, closed_receiver) = mpsc::sync_channel(1);
    let closing_lifecycle = Arc::clone(&lifecycle);

    thread::spawn(move || {
        let result = closing_lifecycle.close(41);
        let _ = closed_sender.send(result);
    });

    let observation_lifecycle = Arc::clone(&lifecycle);
    let closing_observation = observe_within(move || {
        let observation = observation_lifecycle.observation();
        (observation != SignalOwnerLifecycleObservation::Open).then_some(observation)
    });
    let close_waited_for_admission = closed_receiver.try_recv() == Err(TryRecvError::Empty);
    let (late_admission_tx, late_admission_rx) = mpsc::sync_channel(1);
    let late_lifecycle = Arc::clone(&lifecycle);
    thread::spawn(move || {
        let result = late_lifecycle.admit(41).map(|_| ());
        let _ = late_admission_tx.send(result);
    });
    let late_admission = late_admission_rx.recv_timeout(PROGRESS_BOUND);
    drop(admission);
    let close_result = closed_receiver.recv_timeout(PROGRESS_BOUND);

    assert_eq!(
        closing_observation,
        Ok(SignalOwnerLifecycleObservation::Closing),
        "close never exposed Closing within the bound"
    );
    assert!(
        close_waited_for_admission,
        "close completed before its admitted operation released"
    );
    assert_eq!(close_result, Ok(Ok(())), "close drain must finish in bound");
    assert_eq!(
        lifecycle.observation(),
        SignalOwnerLifecycleObservation::Closed
    );
    assert!(matches!(
        late_admission,
        Ok(Err(SignalOwnerAdmissionDenial::OwnerUnavailable))
    ));
    lifecycle
        .close(41)
        .expect("closed is terminal and idempotent");
    assert_eq!(counters.snapshot().close_batches(), 0);
}

#[test]
fn lifecycle_poison_policy_preserves_open_status_and_later_close() {
    let counters = Arc::new(SignalOwnerServiceCounters::default());
    let lifecycle = SignalOwnerLifecycleState::new(9, counters);
    let poisoned = catch_unwind(AssertUnwindSafe(|| lifecycle.poison_status_for_test()));
    assert!(poisoned.is_err());

    assert_eq!(
        lifecycle.observation(),
        SignalOwnerLifecycleObservation::Open
    );
    assert_eq!(
        lifecycle.poison_recovery(),
        Some(SignalOwnerLifecyclePoisonRecovery::PreservedLifecycleStatus)
    );
    let admission = lifecycle
        .admit(9)
        .expect("recovered lifecycle still admits work");
    let (closed_tx, closed_rx) = mpsc::sync_channel(1);
    let closing_lifecycle = Arc::clone(&lifecycle);
    thread::spawn(move || {
        let _ = closed_tx.send(closing_lifecycle.close(9));
    });
    let observation_lifecycle = Arc::clone(&lifecycle);
    let closing_observation = observe_within(move || {
        let observation = observation_lifecycle.observation();
        (observation == SignalOwnerLifecycleObservation::Closing).then_some(observation)
    });
    drop(admission);
    let close_result = closed_rx.recv_timeout(PROGRESS_BOUND);

    assert_eq!(
        closing_observation,
        Ok(SignalOwnerLifecycleObservation::Closing),
        "poison-recovered lifecycle did not expose Closing within the bound"
    );
    assert_eq!(
        close_result,
        Ok(Ok(())),
        "poison-recovered close drain must finish in bound"
    );
    assert_eq!(
        lifecycle.observation(),
        SignalOwnerLifecycleObservation::Closed
    );
}

#[test]
fn foreign_owner_cannot_admit_or_close_the_lifecycle() {
    let counters = Arc::new(SignalOwnerServiceCounters::default());
    let lifecycle = SignalOwnerLifecycleState::new(7, counters);

    assert!(matches!(
        lifecycle.admit(8),
        Err(SignalOwnerAdmissionDenial::ForeignOwner)
    ));
    assert!(lifecycle.close(8).is_err());
    assert_eq!(
        lifecycle.observation(),
        SignalOwnerLifecycleObservation::Open
    );
}

#[test]
fn owner_admission_capacity_is_finite_pre_effect_and_returns_exactly() {
    let counters = Arc::new(SignalOwnerServiceCounters::default());
    let lifecycle = SignalOwnerLifecycleState::new(71, counters);
    let admissions = (0..MAXIMUM_IN_FLIGHT_SIGNAL_OWNER_OPERATIONS)
        .map(|_| lifecycle.admit(71).expect("each bounded slot admits"))
        .collect::<Vec<_>>();

    assert_eq!(
        lifecycle.admit(71).unwrap_err(),
        SignalOwnerAdmissionDenial::OperationCapacityExhausted {
            maximum_in_flight_operations: MAXIMUM_IN_FLIGHT_SIGNAL_OWNER_OPERATIONS,
        }
    );
    drop(admissions);
    lifecycle
        .admit(71)
        .expect("releasing every admission restores exact capacity");
}

#[test]
fn preexisting_and_fresh_same_thread_admissions_reject_owner_state_reentry() {
    let lifecycle =
        SignalOwnerLifecycleState::new(72, Arc::new(SignalOwnerServiceCounters::default()));
    let first = lifecycle.admit(72).expect("first admission is idle");
    let second = lifecycle
        .admit(72)
        .expect("a second admission may be created before either holds owner state");
    let first_hold = first
        .hold_owner_metadata()
        .expect("first admission acquires metadata");

    assert!(matches!(
        second.hold_owner_metadata(),
        Err(SignalOwnerMetadataHoldDenial::ExecutingThreadReentry)
    ));
    assert_eq!(
        lifecycle.admit(72).unwrap_err(),
        SignalOwnerAdmissionDenial::OwnerReentry
    );
    drop(first_hold);
    second
        .hold_owner_metadata()
        .expect("the preexisting admission is healthy after the earlier hold releases");
}

#[test]
fn explicit_close_from_own_admission_denies_before_closing() {
    let lifecycle =
        SignalOwnerLifecycleState::new(73, Arc::new(SignalOwnerServiceCounters::default()));
    let admission = lifecycle.admit(73).expect("owner admits local work");
    assert_eq!(
        lifecycle.close(73),
        Err(SignalOwnerCloseDenial::ExecutingThreadHasAdmission)
    );
    assert_eq!(
        lifecycle.observation(),
        SignalOwnerLifecycleObservation::Open
    );
    drop(admission);
    lifecycle
        .close(73)
        .expect("close succeeds after local work exits");
}
