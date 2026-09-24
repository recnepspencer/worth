use std::cell::Cell;
use std::rc::Rc;

use crate::native::physical_work_signal::UiNativePhysicalPresentationBasis;
use crate::native::presentation::{
    reserve_presentation_owners, settle_port_result, UiNativePendingExternalObligation,
    UiNativePresentationFailure, UiNativePresentationPortFailure,
};

use super::UiNativeHostState;

struct RetryablePresentationProbe {
    settled: Rc<Cell<bool>>,
    polls: Rc<Cell<u32>>,
}

impl UiNativePendingExternalObligation for RetryablePresentationProbe {
    fn poll_observation(
        &mut self,
        basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
        _device: Option<&wgpu::Device>,
    ) -> crate::native::physical_work_signal::UiNativePhysicalSignalExternalObservation {
        self.polls.set(self.polls.get().saturating_add(1));
        basis.observe(if self.settled.get() {
            crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Completed
        } else {
            crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Pending
        })
    }
}

#[test]
fn temporal_retry_refreshes_the_exact_retained_presentation_attempt() {
    let settled = Rc::new(Cell::new(false));
    let polls = Rc::new(Cell::new(0));
    let mut state = UiNativeHostState::new();
    let owners = reserve_presentation_owners(
        &mut state.resources,
        &mut state.physical_signal,
        UiNativePhysicalPresentationBasis::test(),
    )
    .unwrap_or_else(|_| panic!("empty host admits one presentation obligation"));
    let Err(UiNativePresentationFailure::Pending(pending)) = settle_port_result(
        &mut state.resources,
        &mut state.physical_signal,
        owners,
        Err(UiNativePresentationPortFailure::ReadbackUnsettled(
            Box::new(RetryablePresentationProbe {
                settled: Rc::clone(&settled),
                polls: Rc::clone(&polls),
            }),
        )),
    ) else {
        panic!("unsettled presentation must retain its physical obligation");
    };
    state.pending_presentations.push(pending);

    for tick in 1..=7 {
        assert_eq!(state.physical_signal.next_due_tick(), Some(tick));
        state.physical_signal.advance_clock_to(tick).unwrap();
        assert!(state.progress_one_physical_signal_ready());
        assert_eq!(state.pending_presentations.len(), 1);
    }
    assert_eq!(polls.get(), 7);

    state.physical_signal.advance_clock_to(8).unwrap();
    assert_eq!(state.physical_signal.next_due_tick(), Some(9));
    settled.set(true);
    state.physical_signal.advance_clock_to(9).unwrap();
    assert!(state.progress_one_physical_signal_ready());
    assert_eq!(polls.get(), 8);
    assert!(state.pending_presentations.is_empty());
    assert_eq!(state.physical_signal.observation().active_requests, 0);
    assert!(state.resources.current().is_zero());
}

#[test]
fn timed_out_recovery_completes_the_retained_presentation_as_indeterminate() {
    let settled = Rc::new(Cell::new(false));
    let polls = Rc::new(Cell::new(0));
    let mut state = UiNativeHostState::new();
    let owners = reserve_presentation_owners(
        &mut state.resources,
        &mut state.physical_signal,
        UiNativePhysicalPresentationBasis::test(),
    )
    .unwrap_or_else(|_| panic!("presentation owners"));
    let Err(UiNativePresentationFailure::Pending(mut pending)) = settle_port_result(
        &mut state.resources,
        &mut state.physical_signal,
        owners,
        Err(UiNativePresentationPortFailure::ReadbackUnsettled(
            Box::new(RetryablePresentationProbe {
                settled: Rc::clone(&settled),
                polls: Rc::clone(&polls),
            }),
        )),
    ) else {
        panic!("unsettled presentation must remain pending");
    };
    assert!(pending.bind_completion_identity(1, None));
    state.pending_presentations.push(pending);

    for _ in 0..80 {
        if state
            .physical_signal
            .observation()
            .counters
            .recovery_schedules
            != 0
        {
            break;
        }
        let due = state
            .physical_signal
            .next_due_tick()
            .expect("pending request has a due transition");
        state
            .physical_signal
            .advance_clock_to(due)
            .expect("clock progression");
        if state
            .physical_signal
            .observation()
            .counters
            .recovery_schedules
            == 0
            && state.physical_signal.observation().pending_wakes != 0
        {
            state.progress_one_physical_signal_ready();
        }
    }
    assert_eq!(
        state
            .physical_signal
            .observation()
            .counters
            .recovery_schedules,
        1
    );
    settled.set(true);
    assert!(state.progress_one_physical_signal_ready());
    assert_eq!(state.physical_signal.observation().active_requests, 0);
    assert_eq!(state.pending_presentations.len(), 1);
    assert!(matches!(
        state.pending_presentations[0].take_completion(),
        crate::native::presentation::UiNativePendingPresentationCompletion::Indeterminate
    ));
    assert!(state.resources.current().is_zero());
}

struct RejectedPresentationProbe;

impl UiNativePendingExternalObligation for RejectedPresentationProbe {
    fn poll_observation(
        &mut self,
        basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
        _device: Option<&wgpu::Device>,
    ) -> crate::native::physical_work_signal::UiNativePhysicalSignalExternalObservation {
        basis.observe(
            crate::native::physical_work_signal::UiNativePhysicalSignalStatus::RejectedBeforeEffects,
        )
    }
}

#[test]
fn terminal_rejection_releases_the_exact_presentation_owners() {
    let mut state = UiNativeHostState::new();
    let owners = reserve_presentation_owners(
        &mut state.resources,
        &mut state.physical_signal,
        UiNativePhysicalPresentationBasis::test(),
    )
    .unwrap_or_else(|_| panic!("empty host admits one presentation obligation"));
    let Err(UiNativePresentationFailure::Pending(pending)) = settle_port_result(
        &mut state.resources,
        &mut state.physical_signal,
        owners,
        Err(UiNativePresentationPortFailure::ReadbackUnsettled(
            Box::new(RejectedPresentationProbe),
        )),
    ) else {
        panic!("unsettled presentation must retain its physical obligation");
    };
    state.pending_presentations.push(pending);

    state.physical_signal.advance_clock_to(1).unwrap();
    assert!(state.progress_one_physical_signal_ready());
    assert!(state.pending_presentations.is_empty());
    assert_eq!(state.physical_signal.observation().active_requests, 0);
    assert!(state.resources.current().is_zero());
}
