use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

use super::callback_thread;
use super::{
    UiNativeEventLoopApplication, UiNativeEventLoopClient, UiNativeEventLoopDirective,
    UiNativeEventLoopRunDenial,
};
use crate::native::UiNativeLifecycleRequiredAction;

impl<Client: UiNativeEventLoopClient>
    ApplicationHandler<crate::native::readiness::UiNativeApplicationWake>
    for UiNativeEventLoopApplication<Client>
{
    fn new_events(&mut self, event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
        if event_loop.exiting() {
            return;
        }
        self.restore_wait_before_observation_deadline(event_loop);
        self.advance_physical_signal_clock(event_loop);
        self.progress_due_presentation_retry(event_loop);
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if event_loop.exiting() {
            return;
        }
        let callback_thread = std::thread::current().id();
        let admission = callback_thread::transition(
            &mut self.thread_observation,
            self.run_thread,
            callback_thread,
        );
        let Ok(admission) = admission else {
            return self.fail(event_loop, UiNativeEventLoopRunDenial::ApplicationDriver);
        };
        self.resume_admitted(event_loop, admission);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if event_loop.exiting() || !self.owns_window(window_id) {
            return;
        }
        match event {
            WindowEvent::RedrawRequested => self.redraw(event_loop),
            WindowEvent::Resized(size) => {
                self.observe_resize(event_loop, [size.width, size.height])
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.change_scale(event_loop, scale_factor)
            }
            WindowEvent::Occluded(occluded) => self.change_visibility(event_loop, occluded),
            WindowEvent::CloseRequested => self.handle_close_requested(event_loop),
            event => {
                self.observe_native_input(event_loop, &event);
            }
        }
        self.watch_deadlines(false);
    }

    fn user_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _event: crate::native::readiness::UiNativeApplicationWake,
    ) {
        if event_loop.exiting() {
            return;
        }
        self.prepare_pending_resize(event_loop);
        if event_loop.exiting() {
            return;
        }
        // A modal platform loop dispatches this wake without `NewEvents`, so
        // the wake progresses timed work itself.
        self.advance_physical_signal_clock(event_loop);
        self.progress_due_presentation_retry(event_loop);
        if event_loop.exiting() {
            return;
        }
        self.progress_application_readiness(event_loop);
        // Ready physical work is a completion, not a paint, so it is not left
        // to redraw alone. Windows synthesizes a paint only once no posted
        // message is queued, and readiness wakes are posted messages: a
        // Motion lane waking faster than a turn completes would otherwise
        // hold a presented frame, and everything waiting on it, off the
        // client for as long as it keeps waking.
        if !event_loop.exiting() {
            self.request_physical_signal_redraw();
            self.progress_ready_physical_client(event_loop);
        }
        self.watch_deadlines(true);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if event_loop.exiting() {
            return;
        }
        self.prepare_pending_resize(event_loop);
        if event_loop.exiting() {
            return;
        }
        let close_effects_settled =
            self.pending_close && self.shared.borrow().external_effects_settled_for_close();
        if close_effects_settled
            && self.apply_client_directive(event_loop, UiNativeEventLoopDirective::Close)
        {
            return;
        }
        self.request_physical_signal_redraw();
        if self
            .shared
            .borrow()
            .physical_signal
            .observation()
            .pending_wakes
            != 0
        {
            event_loop.set_control_flow(ControlFlow::Poll);
        }
        self.schedule_physical_signal_deadline(event_loop);
        self.watch_deadlines(true);
        if self.first_frame_presented {
            if self.signal_native_observation_readiness(event_loop) {
                return;
            }
            self.idle_wait_turns += 1;
            self.first_frame_presented = false;
        }
        self.close_observation_time_and_schedule(event_loop);
    }
}

impl<Client: UiNativeEventLoopClient> UiNativeEventLoopApplication<Client> {
    pub(super) fn change_visibility(&mut self, event_loop: &ActiveEventLoop, occluded: bool) {
        let changed = self
            .shared
            .borrow_mut()
            .presentation_surface_mut()
            .map_or(Ok(false), |surface| surface.observe_occlusion(occluded));
        match changed {
            Ok(true) if occluded => {
                let _ = self.shared.borrow_mut().observe_surface_basis_transition(
                    crate::native::UiNativeSurfaceBasisTransition::Minimized,
                );
            }
            Ok(true) => self.commit_visible_surface_readiness(event_loop),
            Ok(false) if !occluded && self.awaits_presentation_visibility() => {
                self.commit_visible_surface_readiness(event_loop);
            }
            Ok(false) => {}
            Err(()) => self.fail(event_loop, UiNativeEventLoopRunDenial::GraphicsPreparation),
        }
    }

    pub(super) fn apply_client_directive(
        &mut self,
        event_loop: &ActiveEventLoop,
        directive: UiNativeEventLoopDirective,
    ) -> bool {
        if matches!(directive, UiNativeEventLoopDirective::Close) {
            self.pending_close = true;
            if !self.shared.borrow().external_effects_settled_for_close() {
                self.request_physical_signal_redraw();
                return false;
            }
            let transition = self.shared.borrow_mut().lifecycle.request_close();
            if transition.required_action() == Some(UiNativeLifecycleRequiredAction::DrainRetained)
            {
                if self.signal_native_observation_readiness(event_loop) {
                    return true;
                }
                self.request_physical_signal_redraw();
                return false;
            }
        }
        if super::directive::apply(event_loop, directive) {
            event_loop.exit();
            true
        } else {
            self.finalize_presentation_retry_round(event_loop)
        }
    }

    fn owns_window(&self, window_id: WindowId) -> bool {
        self.shared
            .borrow()
            .window
            .as_ref()
            .is_some_and(|window| window.id() == window_id)
    }
}
