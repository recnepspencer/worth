use winit::event_loop::ActiveEventLoop;

use super::{UiNativeEventLoopApplication, UiNativeEventLoopClient, UiNativeEventLoopRunDenial};
use crate::native::{UiNativeHostState, UiNativeSurfaceBasisTransition};

mod pending;

pub(super) use pending::UiNativePendingResize;
use pending::UiNativeResizeAdmission;

impl<Client: UiNativeEventLoopClient> UiNativeEventLoopApplication<Client> {
    /// Observes a client extent the window reported.
    ///
    /// A positive extent waits for the next dispatch turn, which prepares one
    /// target for the newest extent observed by then. A zero extent suspends
    /// presentation now.
    pub(super) fn observe_resize(&mut self, event_loop: &ActiveEventLoop, size: [u32; 2]) {
        match self.pending_resize.observe(size) {
            UiNativeResizeAdmission::Pending => {
                if let Some(window) = self.shared.borrow().window.as_ref() {
                    window.request_redraw();
                }
                self.refresh_client_origin();
            }
            UiNativeResizeAdmission::Suspend(size) => {
                if self.replace_surface_basis(event_loop, size, None) {
                    self.observe_native_profile(self.surface_scale_factor(), size);
                }
            }
        }
    }

    /// Prepares the target for the newest pending extent, if any.
    ///
    /// Every dispatch turn that can hand the client work calls this first:
    /// redraw, a posted wake, input observation, and the idle turn. The client
    /// may present from any of them, so none may present onto the target of an
    /// extent the window has already left. The viewport is published only for
    /// the extent actually prepared, so no presentation reports an extent its
    /// pixels do not have.
    pub(super) fn prepare_pending_resize(&mut self, event_loop: &ActiveEventLoop) {
        let Some(size) = self.pending_resize.take() else {
            return;
        };
        if self.replace_surface_basis(event_loop, size, None) {
            self.observe_native_profile(self.surface_scale_factor(), size);
        }
    }

    pub(super) fn change_scale(&mut self, event_loop: &ActiveEventLoop, scale_factor: f64) {
        // The scale transition reads the window's current extent, so it
        // prepares the target any pending extent was waiting for.
        self.pending_resize.supersede();
        let physical_size = self
            .shared
            .borrow()
            .window
            .as_ref()
            .map(|window| window.client_physical_size());
        let Some(size) = physical_size else {
            return;
        };
        if self.replace_surface_basis(event_loop, size, Some(scale_factor)) {
            self.observe_native_profile(Some(scale_factor), size);
        }
    }

    /// Replaces the surface basis with `size`, and with `scale_factor` when
    /// the scale changes. Reports whether the event loop continues.
    fn replace_surface_basis(
        &mut self,
        event_loop: &ActiveEventLoop,
        size: [u32; 2],
        scale_factor: Option<f64>,
    ) -> bool {
        let minimized = size.contains(&0);
        let replacement = {
            let mut shared = self.shared.borrow_mut();
            let UiNativeHostState {
                presentation_owners,
                resources,
                ..
            } = &mut *shared;
            let changed = presentation_owners.as_mut().map_or(
                Ok(false),
                |crate::native::UiNativePresentationOwners { device, surface }| match scale_factor {
                    Some(scale_factor) => crate::native::lifecycle::rebind_surface_scale(
                        device,
                        surface,
                        scale_factor,
                        size,
                        resources,
                    ),
                    None => {
                        crate::native::lifecycle::resize_surface(device, surface, size, resources)
                    }
                },
            );
            changed.map(|changed| {
                let suspended = presentation_owners
                    .as_ref()
                    .is_some_and(|owners| owners.surface.state().suspended());
                (changed, suspended)
            })
        };
        match replacement {
            Ok((true, suspended)) => {
                let mut shared = self.shared.borrow_mut();
                if minimized {
                    let _ = shared
                        .presentation_surface_mut()
                        .map(|surface| surface.observe_occlusion(true));
                }
                let transition = if minimized {
                    UiNativeSurfaceBasisTransition::Minimized
                } else if suspended {
                    UiNativeSurfaceBasisTransition::ZeroSized
                } else if scale_factor.is_some() {
                    UiNativeSurfaceBasisTransition::Dpi
                } else {
                    UiNativeSurfaceBasisTransition::Resize
                };
                let _directive = shared.observe_surface_basis_transition(transition);
                drop(shared);
                if !suspended && !minimized {
                    self.commit_visible_surface_readiness(event_loop);
                }
                true
            }
            Ok((false, _)) => {
                self.change_visibility(event_loop, minimized);
                true
            }
            Err(()) => {
                self.fail(event_loop, UiNativeEventLoopRunDenial::GraphicsPreparation);
                false
            }
        }
    }

    fn surface_scale_factor(&self) -> Option<f64> {
        self.shared
            .borrow()
            .presentation_surface()
            .map(|surface| surface.state().scale_factor())
    }

    fn observe_native_profile(&mut self, scale_factor: Option<f64>, size: [u32; 2]) {
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
        self.refresh_client_origin();
    }

    fn refresh_client_origin(&mut self) {
        if let Some(input) = self.pointer_input.as_mut() {
            input.refresh_client_origin();
        }
    }
}
