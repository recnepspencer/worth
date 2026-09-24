use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;

impl<Client: super::UiNativeEventLoopClient> super::UiNativeEventLoopApplication<Client> {
    pub(super) fn commit_readiness(&mut self, event_loop: &ActiveEventLoop) {
        let basis = self.shared.borrow().presentation_surface().map(|surface| {
            (
                (surface.state().scale_factor() * 1_000.0).round() as u32,
                surface.state().extent(),
            )
        });
        let Some((scale_factor_milli, client_physical_size)) = basis else {
            return self.fail(
                event_loop,
                super::UiNativeEventLoopRunDenial::GraphicsPreparation,
            );
        };
        if self
            .readiness
            .commit_latest(
                self.readiness_owner,
                scale_factor_milli,
                client_physical_size,
            )
            .is_err()
        {
            return self.fail(
                event_loop,
                super::UiNativeEventLoopRunDenial::ApplicationDriver,
            );
        }
        let window = self
            .shared
            .borrow()
            .window
            .as_ref()
            .map(|window| Arc::clone(window));
        match crate::native::readiness::signal_committed(
            &self.readiness,
            self.readiness_owner,
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
            Ok(crate::native::readiness::UiNativeReadinessSignalDisposition::NoWork) => {
                unreachable!("committed application readiness always carries work")
            }
            Err(()) => self.fail(
                event_loop,
                super::UiNativeEventLoopRunDenial::ApplicationDriver,
            ),
        }
    }
}
