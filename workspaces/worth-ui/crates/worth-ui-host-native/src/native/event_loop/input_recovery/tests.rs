use super::*;
use crate::native::event_loop::{
    UiNativeEventLoopClientCallback, UiNativeEventLoopClientClose,
    UiNativeEventLoopClientDenial as Denial, UiNativeObservationClock,
    UiNativeObservationReadinessGrant, UiNativeObservationTimeProgress, UiNativeReadinessGrant,
};
use winit::event::WindowEvent;
use worth_ui_host_contract::*;

#[test]
fn loop_recovery_waits_for_drain_and_allows_callback_host_reentry() {
    for refuses in [false, true] {
        let shared = exhausted_host();
        let mut client = RecoveryClient {
            shared: Rc::clone(&shared),
            refuses,
            calls: 0,
        };
        assert!(settle(&shared, &mut client, UiNativeEventLoopDirective::Continue).is_ok());
        assert_eq!(client.calls, 0, "retained input must precede cancellation");
        assert_eq!(
            shared
                .borrow_mut()
                .lifecycle
                .drain_observations(97)
                .into_batches()
                .len(),
            16
        );
        assert!(settle(&shared, &mut client, UiNativeEventLoopDirective::Close).is_ok());
        assert_eq!(client.calls, 0, "closing does not reopen input");
        let result = settle(&shared, &mut client, UiNativeEventLoopDirective::Continue);
        if refuses {
            let Err(UiNativeEventLoopRunDenial::ClientCallback(failure)) = result else {
                panic!("failed cancellation must stop at the named callback")
            };
            assert_eq!(
                failure.callback(),
                UiNativeEventLoopClientCallback::NativeInputRetentionExhausted
            );
            assert_eq!(failure.denial(), Denial::ApplicationProgressDenied);
            assert!(shared
                .borrow()
                .lifecycle
                .input_report()
                .terminal_stop()
                .is_some());
        } else {
            assert!(result.is_ok());
            assert_eq!(
                shared.borrow().lifecycle.input_report().terminal_stop(),
                None
            );
            assert_eq!(
                shared
                    .borrow_mut()
                    .lifecycle
                    .observe_window_event_at(&WindowEvent::Focused(true), 30, None)
                    .effect(),
                crate::native::UiNativeLifecycleEffect::Retained
            );
        }
        assert_eq!(client.calls, 1);
        shared.borrow_mut().lifecycle.release_session(97);
    }
}

fn exhausted_host() -> Rc<RefCell<UiNativeHostState>> {
    let mut state = UiNativeHostState::new();
    let UiHostProtocolNegotiation::Compatible(protocol) =
        UiHostProtocolContract::current().negotiate()
    else {
        panic!("current protocol")
    };
    state.lifecycle.register_session(97).unwrap();
    state.lifecycle.install_initial_profile(1.0, [800, 600]);
    state.lifecycle.record_completed_presentation(
        protocol,
        97,
        UiHostObservationPresentationBasis::new(
            UiHostSurfaceIdentity::mint_unbound().unwrap(),
            UiMountedFrameIdentity::mint_unbound().unwrap(),
            UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            UiHostPresentationEpoch::issued_by_host(1),
        ),
    );
    for tick in 0..17 {
        state
            .lifecycle
            .observe_window_event_at(&WindowEvent::Focused(true), tick, None);
    }
    assert!(state.lifecycle.input_report().terminal_stop().is_some());
    Rc::new(RefCell::new(state))
}

struct RecoveryClient {
    shared: Rc<RefCell<UiNativeHostState>>,
    refuses: bool,
    calls: usize,
}

impl UiNativeEventLoopClient for RecoveryClient {
    fn native_input_retention_exhausted(
        &mut self,
        grant: crate::UiNativeInputRecoveryGrant,
    ) -> Result<
        (
            crate::UiNativeInputRecoveryAcknowledgement,
            UiNativeEventLoopDirective,
        ),
        Denial,
    > {
        self.calls += 1;
        // Production cancellation uses the host adapter; a held host-state
        // borrow here would panic before either success or refusal is observed.
        let state = self.shared.borrow_mut();
        assert!(!state.lifecycle.has_retained_observations());
        assert!(state.lifecycle.input_report().terminal_stop().is_some());
        if self.refuses {
            Err(Denial::ApplicationProgressDenied)
        } else {
            Ok((
                grant.acknowledge_cancellation(),
                UiNativeEventLoopDirective::Continue,
            ))
        }
    }
    fn install_observation_clock(&mut self, _: UiNativeObservationClock) -> Result<(), Denial> {
        unreachable!()
    }
    fn observation_time_ready(&mut self) -> Result<UiNativeObservationTimeProgress, Denial> {
        unreachable!()
    }
    fn native_surface_ready(
        &mut self,
        _: UiNativeReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, Denial> {
        unreachable!()
    }
    fn redraw_ready(
        &mut self,
        _: UiNativeReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, Denial> {
        unreachable!()
    }
    fn native_observations_ready(
        &mut self,
        _: UiNativeObservationReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, Denial> {
        unreachable!()
    }
    fn presentation_attribution(
        &self,
        _: &crate::native::UiNativeRetainedFrameObservation,
    ) -> Option<crate::UiNativeClientPresentationAttribution> {
        None
    }
    fn close(self) -> UiNativeEventLoopClientClose {
        UiNativeEventLoopClientClose::Complete
    }
}
