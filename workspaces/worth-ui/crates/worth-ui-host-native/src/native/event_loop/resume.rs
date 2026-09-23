use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;

use super::client_invocation::UiNativeEventLoopClientInvocation;
use super::{
    callback_thread, directive, pointer_position, window_port, UiNativeEventLoopApplication,
    UiNativeEventLoopClient, UiNativeEventLoopRunDenial, UiNativeOwnedWindow,
    UiNativeReadinessGrant,
};
use crate::native::{UiNativeOwnedDevice, UiNativeOwnedPresentationSurface};
use window_port::UiNativeWindowPort;

impl<Client: UiNativeEventLoopClient> UiNativeEventLoopApplication<Client> {
    pub(super) fn resume_admitted(
        &mut self,
        event_loop: &ActiveEventLoop,
        _admission: callback_thread::UiNativeEventLoopThreadObservation,
    ) {
        if self.shared.borrow().window.is_some() {
            return;
        }
        let (window, pointer_input) = match self.open_registered_window(event_loop) {
            Ok(opened) => opened,
            Err(denial) => return self.fail(event_loop, denial),
        };
        let (device, surface) = match self.prepare_registered_graphics(&window) {
            Ok(owners) => owners,
            Err(denial) => {
                window.close(&mut self.shared.borrow_mut().resources);
                return self.fail(event_loop, denial);
            }
        };
        self.install_surface(window, device, surface, pointer_input);
        let directive = match self.surface_ready_directive() {
            Ok(directive) => directive,
            Err(denial) => return self.fail(event_loop, denial),
        };
        if directive::apply(event_loop, directive) {
            return event_loop.exit();
        }
        self.commit_readiness(event_loop);
    }

    fn open_registered_window(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<
        (
            UiNativeOwnedWindow,
            Option<Box<pointer_position::UiNativePointerInputPort>>,
        ),
        UiNativeEventLoopRunDenial,
    > {
        if !self.shared.borrow().resources.admits(6) {
            return Err(UiNativeEventLoopRunDenial::IncompleteCleanup);
        }
        let opened = window_port::UiWinitNativeWindowPort::open(event_loop, &self.configuration)
            .map_err(|_| UiNativeEventLoopRunDenial::WindowCreation)?;
        let (window, crossings) = opened
            .register(&mut self.shared.borrow_mut().resources)
            .map_err(|()| UiNativeEventLoopRunDenial::IncompleteCleanup)?;
        self.port_crossings = self.port_crossings.saturating_add(crossings);
        let pointer = install_pointer(&window).ok_or(UiNativeEventLoopRunDenial::WindowCreation)?;
        Ok((window, pointer))
    }

    fn prepare_registered_graphics(
        &mut self,
        window: &UiNativeOwnedWindow,
    ) -> Result<(UiNativeOwnedDevice, UiNativeOwnedPresentationSurface), UiNativeEventLoopRunDenial>
    {
        let prepared = crate::native::graphics::prepare_platform_graphics(Arc::clone(window))
            .map_err(|_| UiNativeEventLoopRunDenial::GraphicsPreparation)?;
        let (device, surface, crossings) = prepared.into_parts();
        self.port_crossings = self.port_crossings.saturating_add(crossings);
        crate::native::lifecycle::register_platform_owners(
            device,
            surface,
            &mut self.shared.borrow_mut().resources,
        )
        .map_err(|mechanics| {
            drop(mechanics);
            UiNativeEventLoopRunDenial::IncompleteCleanup
        })
    }

    fn install_surface(
        &mut self,
        window: UiNativeOwnedWindow,
        device: UiNativeOwnedDevice,
        surface: UiNativeOwnedPresentationSurface,
        pointer_input: Option<Box<pointer_position::UiNativePointerInputPort>>,
    ) {
        let profile = (surface.state().scale_factor(), surface.state().extent());
        let mut state = self.shared.borrow_mut();
        state.device = Some(device);
        state.presentation_surface = Some(surface);
        state.window = Some(window);
        state
            .lifecycle
            .install_initial_profile(profile.0, profile.1);
        self.pointer_input = pointer_input;
    }

    fn surface_ready_directive(
        &mut self,
    ) -> Result<super::UiNativeEventLoopDirective, UiNativeEventLoopRunDenial> {
        let state = self.shared.borrow();
        let surface = state
            .presentation_surface
            .as_ref()
            .ok_or(UiNativeEventLoopRunDenial::GraphicsPreparation)?;
        let preparation = {
            UiNativeReadinessGrant::issued(
                0,
                surface.basis_generation(),
                (surface.state().scale_factor() * 1_000.0).round() as u32,
                surface.state().extent(),
            )
        };
        drop(state);
        self.client_or_denied()?
            .invoke_native_surface_ready(preparation)
            .map_err(UiNativeEventLoopRunDenial::ClientCallback)
    }
}

/// A qualified platform must produce its pointer port: without one the first
/// button event reports its position unavailable and input observation stops
/// for the session, so a missing port is a window-creation denial, not a
/// degraded start.
#[cfg(any(target_os = "windows", target_os = "linux"))]
fn install_pointer(
    window: &UiNativeOwnedWindow,
) -> Option<Option<Box<pointer_position::UiNativePointerInputPort>>> {
    pointer_position::install_pointer_input(window).map(Some)
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn install_pointer(
    _window: &UiNativeOwnedWindow,
) -> Option<Option<Box<pointer_position::UiNativePointerInputPort>>> {
    Some(None)
}
