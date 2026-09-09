use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

use super::callback_thread;
use super::{
    UiNativeEventLoopApplication, UiNativeEventLoopClient, UiNativeEventLoopDirective,
    UiNativeEventLoopRunDenial, UiNativeObservationReadinessGrant,
};
use crate::native::{UiNativeHostState, UiNativeLifecycleEffect, UiNativeLifecycleRequiredAction};

impl<Client: UiNativeEventLoopClient>
    ApplicationHandler<crate::native::readiness::UiNativeApplicationWake>
    for UiNativeEventLoopApplication<Client>
{
    fn new_events(&mut self, event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
        self.restore_wait_before_observation_deadline(event_loop);
        self.advance_physical_signal_clock(event_loop);
        self.progress_due_presentation_retry(event_loop);
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
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
        if !self.owns_window(window_id) {
            return;
        }
        match event {
            WindowEvent::RedrawRequested => self.redraw(event_loop),
            WindowEvent::Resized(size) => self.resize(event_loop, [size.width, size.height]),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.change_scale(event_loop, scale_factor)
            }
            WindowEvent::Occluded(occluded) => self.change_visibility(event_loop, occluded),
            WindowEvent::CloseRequested => self.handle_close_requested(event_loop),
            event => self.observe_native_input(event_loop, &event),
        }
    }

    fn user_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _event: crate::native::readiness::UiNativeApplicationWake,
    ) {
        self.progress_application_readiness(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
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
        if self.first_frame_presented {
            self.signal_native_observation_readiness(event_loop);
            self.idle_wait_turns += 1;
            self.first_frame_presented = false;
        }
        self.close_observation_time_and_schedule(event_loop);
    }
}

impl<Client: UiNativeEventLoopClient> UiNativeEventLoopApplication<Client> {
    fn change_visibility(&mut self, event_loop: &ActiveEventLoop, occluded: bool) {
        let changed = self
            .shared
            .borrow_mut()
            .presentation_surface
            .as_mut()
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

    fn resize(&mut self, event_loop: &ActiveEventLoop, size: [u32; 2]) {
        let replacement = {
            let mut shared = self.shared.borrow_mut();
            let minimized = size.contains(&0);
            let UiNativeHostState {
                device,
                presentation_surface,
                resources,
                ..
            } = &mut *shared;
            let changed = device.as_ref().zip(presentation_surface.as_mut()).map_or(
                Ok(false),
                |(device, surface)| {
                    crate::native::lifecycle::resize_surface(device, surface, size, resources)
                },
            );
            changed.map(|changed| {
                let suspended = presentation_surface
                    .as_ref()
                    .is_some_and(|surface| surface.state().suspended());
                (changed, suspended, minimized)
            })
        };
        match replacement {
            Ok((true, suspended, minimized)) => {
                let mut shared = self.shared.borrow_mut();
                if minimized {
                    let _ = shared
                        .presentation_surface
                        .as_mut()
                        .map(|surface| surface.observe_occlusion(true));
                }
                let transition = if minimized {
                    crate::native::UiNativeSurfaceBasisTransition::Minimized
                } else if suspended {
                    crate::native::UiNativeSurfaceBasisTransition::ZeroSized
                } else {
                    crate::native::UiNativeSurfaceBasisTransition::Resize
                };
                let _directive = shared.observe_surface_basis_transition(transition);
                drop(shared);
                if !suspended && !minimized {
                    self.commit_visible_surface_readiness(event_loop);
                }
            }
            Ok((false, _, minimized)) => self.change_visibility(event_loop, minimized),
            Err(()) => {
                self.fail(event_loop, UiNativeEventLoopRunDenial::GraphicsPreparation);
                return;
            }
        }
        let scale_factor = self
            .shared
            .borrow()
            .presentation_surface
            .as_ref()
            .map(|surface| surface.state().scale_factor());
        if let Some(scale_factor) = scale_factor {
            self.shared
                .borrow_mut()
                .lifecycle
                .observe_profile_transition_at(
                    scale_factor,
                    size,
                    self.physical_clock.current_tick(),
                );
        }
        if let Some(input) = self.pointer_input.as_mut() {
            input.refresh_client_origin();
        }
    }

    fn change_scale(&mut self, event_loop: &ActiveEventLoop, scale_factor: f64) {
        let physical_size = self
            .shared
            .borrow()
            .window
            .as_ref()
            .map(|window| window.client_physical_size());
        let replacement = {
            let mut shared = self.shared.borrow_mut();
            let minimized = physical_size.is_some_and(|size| size.contains(&0));
            let UiNativeHostState {
                device,
                presentation_surface,
                resources,
                ..
            } = &mut *shared;
            let changed = physical_size
                .zip(device.as_ref().zip(presentation_surface.as_mut()))
                .map_or(Ok(false), |(size, (device, surface))| {
                    crate::native::lifecycle::rebind_surface_scale(
                        device,
                        surface,
                        scale_factor,
                        size,
                        resources,
                    )
                });
            changed.map(|changed| {
                let suspended = presentation_surface
                    .as_ref()
                    .is_some_and(|surface| surface.state().suspended());
                (changed, suspended, minimized)
            })
        };
        match replacement {
            Ok((true, suspended, minimized)) => {
                let mut shared = self.shared.borrow_mut();
                if minimized {
                    let _ = shared
                        .presentation_surface
                        .as_mut()
                        .map(|surface| surface.observe_occlusion(true));
                }
                let transition = if minimized {
                    crate::native::UiNativeSurfaceBasisTransition::Minimized
                } else if suspended {
                    crate::native::UiNativeSurfaceBasisTransition::ZeroSized
                } else {
                    crate::native::UiNativeSurfaceBasisTransition::Dpi
                };
                let _directive = shared.observe_surface_basis_transition(transition);
                drop(shared);
                if !suspended && !minimized {
                    self.commit_visible_surface_readiness(event_loop);
                }
            }
            Ok((false, _, minimized)) => self.change_visibility(event_loop, minimized),
            Err(()) => {
                self.fail(event_loop, UiNativeEventLoopRunDenial::GraphicsPreparation);
                return;
            }
        }
        if let Some(size) = physical_size {
            self.shared
                .borrow_mut()
                .lifecycle
                .observe_profile_transition_at(
                    scale_factor,
                    size,
                    self.physical_clock.current_tick(),
                );
        }
        if let Some(input) = self.pointer_input.as_mut() {
            input.refresh_client_origin();
        }
    }

    fn observe_native_input(&mut self, event_loop: &ActiveEventLoop, event: &WindowEvent) {
        let reachability =
            crate::native::event_loop::contract::UiNativeInputReachability::observe_window_event(
                event,
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
        self.notify_native_observations_ready(event_loop);
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
