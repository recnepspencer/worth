use std::cell::RefCell;
use std::rc::Rc;

use super::super::{stop_before_callbacks, UiNativeEventLoopRunDenial};
use super::{CleanupClient, PendingProbe};
use crate::native::presentation::{
    reserve_presentation_owners, settle_port_result, UiNativePresentationFailure,
    UiNativePresentationPortFailure,
};
use crate::native::{UiNativeEffectPosture, UiNativeHostState, UiNativeResourceClass};

#[test]
fn stop_report_retains_effect_posture_and_exact_cleanup_census() {
    let state = Rc::new(RefCell::new(UiNativeHostState::new()));
    state
        .borrow_mut()
        .lifecycle
        .record_presentation_indeterminate();
    let report = stop_before_callbacks(
        state,
        CleanupClient {
            completes: true,
            terminal_resources_complete: true,
        },
        UiNativeEventLoopRunDenial::EventLoopRun,
    );
    assert_eq!(report.cause(), UiNativeEventLoopRunDenial::EventLoopRun);
    assert_eq!(
        report.effect_posture(),
        UiNativeEffectPosture::PresentationIndeterminate
    );
    let peak = report.peak_census();
    assert_eq!(peak.physical_signal_runtimes, 1);
    assert_eq!(peak.physical_signal_workers, 1);
    assert!(peak.entries().all(|(name, count)| {
        matches!(name, "physical_signal_runtimes" | "physical_signal_workers") || count == 0
    }));
    assert!(report.terminal_census().is_zero());
    assert!(report.client_cleanup_complete());
}

#[test]
fn four_timeout_rounds_stop_with_typed_deadline_and_terminal_zero() {
    let state = Rc::new(RefCell::new(UiNativeHostState::new()));
    for round in 0..4 {
        let attempt = worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound()
            .expect("timeout attempt identity");
        state
            .borrow_mut()
            .lifecycle
            .observe_presentation_retry_outcome(
                attempt,
                &worth_ui_host_contract::UiHostSurfacePresentationOutcome::RejectedBeforeEffects(
                    worth_ui_host_contract::UiHostSurfacePresentationDenial::ExternalTimeout,
                ),
            );
        let finalization = state
            .borrow_mut()
            .lifecycle
            .finalize_presentation_retry_round(std::time::Instant::now());
        if round < 3 {
            let crate::native::UiNativePresentationRetryFinalization::Wake(
                crate::native::UiNativePresentationRetryWake::Timeout(deadline),
            ) = finalization
            else {
                panic!("bounded timeout round must schedule exactly one wake")
            };
            assert!(state
                .borrow_mut()
                .lifecycle
                .consume_due_presentation_timeout(deadline));
        } else {
            let denial = super::super::presentation_retry::terminal_denial(finalization)
                .expect("the fourth timeout round is terminal");
            let report = stop_before_callbacks(
                Rc::clone(&state),
                CleanupClient {
                    completes: true,
                    terminal_resources_complete: true,
                },
                denial,
            );
            assert_eq!(
                report.cause(),
                UiNativeEventLoopRunDenial::PresentationDeadlineExpired
            );
            assert!(report.terminal_census().is_zero());
            assert!(report.client_cleanup_complete());
        }
    }
}

#[test]
fn held_resource_with_clean_client_cannot_report_a_clean_stop() {
    for class in UiNativeResourceClass::all() {
        let state = Rc::new(RefCell::new(UiNativeHostState::new()));
        let _held = state.borrow_mut().resources.register(*class).unwrap();
        let report = stop_before_callbacks(
            state,
            CleanupClient {
                completes: true,
                terminal_resources_complete: true,
            },
            UiNativeEventLoopRunDenial::EventLoopCreation,
        );
        assert_eq!(
            report.cause(),
            UiNativeEventLoopRunDenial::EventLoopCreation
        );
        assert!(!report.terminal_census().is_zero(), "omitted {class:?}");
        assert!(report.client_cleanup_complete());
    }
}

#[test]
fn incomplete_client_without_held_resources_cannot_report_a_clean_stop() {
    let state = Rc::new(RefCell::new(UiNativeHostState::new()));
    let report = stop_before_callbacks(
        state,
        CleanupClient {
            completes: false,
            terminal_resources_complete: true,
        },
        UiNativeEventLoopRunDenial::EventLoopCreation,
    );
    assert_eq!(
        report.cause(),
        UiNativeEventLoopRunDenial::EventLoopCreation
    );
    assert!(report.terminal_census().is_zero());
    assert!(!report.client_cleanup_complete());
}

#[test]
fn client_owned_terminal_resources_are_part_of_cleanup_completion() {
    let complete = crate::native::UiNativeClientShutdownObservation::from_client(1, true);
    assert!(complete.terminal_resources_complete());

    let query_open = crate::native::UiNativeClientShutdownObservation::from_client(1, false);
    assert!(!query_open.terminal_resources_complete());

    let intent_open = crate::native::UiNativeClientShutdownObservation::from_client(1, true)
        .with_intent_resources_empty(false);
    assert!(!intent_open.terminal_resources_complete());

    let derived_open = crate::native::UiNativeClientShutdownObservation::from_client(1, true)
        .with_resources(crate::native::UiNativeClientResourceObservation::reported(
            1, 1, 1, 0,
        ));
    assert!(!derived_open.terminal_resources_complete());
}

#[test]
fn client_resource_debt_cannot_report_complete_cleanup() {
    let state = Rc::new(RefCell::new(UiNativeHostState::new()));
    let report = stop_before_callbacks(
        state,
        CleanupClient {
            completes: true,
            terminal_resources_complete: false,
        },
        UiNativeEventLoopRunDenial::EventLoopCreation,
    );
    assert_eq!(
        report.cause(),
        UiNativeEventLoopRunDenial::EventLoopCreation
    );
    assert!(!report.client_cleanup_complete());
}

#[test]
fn indeterminate_external_work_moves_into_retryable_cleanup_authority() {
    let state = Rc::new(RefCell::new(UiNativeHostState::new()));
    let dropped = Rc::new(std::cell::Cell::new(false));
    let settles = Rc::new(std::cell::Cell::new(false));
    {
        let mut state = state.borrow_mut();
        let UiNativeHostState {
            resources,
            physical_signal,
            pending_presentations,
            ..
        } = &mut *state;
        let owners = reserve_presentation_owners(
            resources,
            physical_signal,
            crate::native::physical_work_signal::UiNativePhysicalPresentationBasis::test(),
        )
        .unwrap_or_else(|_| panic!("empty registry must reserve presentation owners"));
        let pending = settle_port_result(
            resources,
            physical_signal,
            owners,
            Err(UiNativePresentationPortFailure::ReadbackUnsettled(
                Box::new(PendingProbe {
                    dropped: Rc::clone(&dropped),
                    settles: Rc::clone(&settles),
                }),
            )),
        );
        let Err(UiNativePresentationFailure::Pending(pending)) = pending else {
            panic!("unsettled external work must enter retryable cleanup");
        };
        pending_presentations.push(pending);
    }
    let report = stop_before_callbacks(
        state,
        CleanupClient {
            completes: true,
            terminal_resources_complete: true,
        },
        UiNativeEventLoopRunDenial::EventLoopRun,
    );
    assert_eq!(report.cause(), UiNativeEventLoopRunDenial::EventLoopRun);
    assert!(!dropped.get());
    let cleanup = report.into_cleanup().expect("pending cleanup authority");
    let cleanup = match cleanup.retry() {
        Err(cleanup) => cleanup,
        Ok(_) => panic!("unsettled external work must retain cleanup authority"),
    };
    settles.set(true);
    std::thread::sleep(std::time::Duration::from_millis(2));
    assert!(cleanup.retry().expect("settled cleanup").is_zero());
    assert!(dropped.get());
}
