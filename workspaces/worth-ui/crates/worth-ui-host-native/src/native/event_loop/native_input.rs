use super::{
    UiNativeEventLoopApplication, UiNativeEventLoopClient, UiNativeEventLoopDirective,
    UiNativeEventLoopRunDenial, UiNativeObservationReadinessGrant,
};
use crate::native::UiNativeLifecycleEffect;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

impl<Client: UiNativeEventLoopClient> UiNativeEventLoopApplication<Client> {
    pub(super) fn observe_native_input(
        &mut self,
        event_loop: &ActiveEventLoop,
        event: &WindowEvent,
    ) {
        let composition = self.shared.borrow().lifecycle.ime_composition_posture();
        let reachability =
            crate::native::event_loop::contract::UiNativeInputReachability::observe_window_event(
                event,
                composition,
            );
        let event_tick = self.physical_clock.current_tick();
        let pointer_witness =
            super::pointer_position::event_pointer_witness(&mut self.pointer_input, event);
        let disposition = self
            .shared
            .borrow_mut()
            .lifecycle
            .observe_window_event_at_with_pointer_witness(event, event_tick, pointer_witness);
        self.pending_input_reachability.merge(reachability);
        if disposition.effect() != UiNativeLifecycleEffect::Retained && reachability.is_empty() {
            return;
        }
        if matches!(event, WindowEvent::CursorMoved { .. })
            && !self
                .shared
                .borrow()
                .lifecycle
                .observation_drain_capacity_reached()
        {
            // Retain ordered observations now, then drain them through the
            // existing redraw grant instead of presenting inside every move.
            self.signal_native_observation_readiness(event_loop);
        } else {
            self.notify_native_observations_ready(event_loop);
        }
    }

    pub(super) fn signal_native_observation_readiness(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> bool {
        let has_ready_work = self.shared.borrow().lifecycle.has_retained_observations()
            || !self.pending_input_reachability.is_empty();
        let window = self
            .shared
            .borrow()
            .window
            .as_ref()
            .map(|window| std::sync::Arc::clone(window));
        match crate::native::readiness::signal_level_ready(
            &self.readiness,
            self.input_readiness_owner,
            has_ready_work,
            || {
                if let Some(window) = &window {
                    window.request_redraw();
                }
            },
        ) {
            Ok(crate::native::readiness::UiNativeReadinessSignalDisposition::RedrawRequested) => {
                self.readiness_signals += 1;
            }
            Ok(crate::native::readiness::UiNativeReadinessSignalDisposition::Coalesced) => {
                self.coalesced_wakes += 1;
            }
            Ok(crate::native::readiness::UiNativeReadinessSignalDisposition::NoWork) => {}
            Err(()) => {
                self.fail(event_loop, UiNativeEventLoopRunDenial::ApplicationDriver);
                return true;
            }
        }
        false
    }

    pub(super) fn notify_native_observations_ready(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> bool {
        if self.signal_native_observation_readiness(event_loop) {
            return true;
        }
        let Ok(grant) = self.readiness.take_level(self.input_readiness_owner) else {
            return false;
        };
        let reachability = std::mem::take(&mut self.pending_input_reachability);
        let directive = self.client.as_mut().and_then(|client| {
            client
                .native_observations_ready(UiNativeObservationReadinessGrant::issued(
                    grant.generation(),
                    reachability,
                ))
                .ok()
        });
        if directive.is_none() {
            self.fail(event_loop, UiNativeEventLoopRunDenial::ApplicationDriver);
            return true;
        }
        let directive = directive.expect("checked observation directive");
        let work_remains = self.shared.borrow().lifecycle.has_retained_observations()
            || !self.pending_input_reachability.is_empty();
        if work_remains {
            self.signal_native_observation_readiness(event_loop);
            if matches!(directive, UiNativeEventLoopDirective::Close) {
                return false;
            }
        }
        if self.apply_client_directive(event_loop, directive) {
            return true;
        }
        false
    }
}
