use winit::event_loop::ActiveEventLoop;

use super::client_invocation::UiNativeEventLoopClientInvocation;
use super::{
    physical_progression, UiNativeEventLoopApplication, UiNativeEventLoopClient,
    UiNativeEventLoopRunDenial, UiNativeReadinessGrant,
};

impl<Client: UiNativeEventLoopClient> UiNativeEventLoopApplication<Client> {
    pub(super) fn redraw(&mut self, event_loop: &ActiveEventLoop) {
        self.prepare_pending_resize(event_loop);
        if event_loop.exiting() {
            return;
        }
        if self.progress_ready_physical_client(event_loop) {
            return;
        }
        self.request_physical_signal_redraw();
        let Ok(work) = self.readiness.take(self.readiness_owner) else {
            self.notify_native_observations_ready(event_loop);
            return;
        };
        self.redraw_turns += 1;
        let surface_basis_generation = self
            .shared
            .borrow()
            .presentation_surface()
            .map(|surface| surface.basis_generation())
            .unwrap_or(0);
        let readiness = UiNativeReadinessGrant::issued(
            work.generation,
            surface_basis_generation,
            work.scale_factor_milli,
            work.client_physical_size,
        );
        let directive = self.client_or_denied().and_then(|client| {
            client
                .invoke_redraw_ready(readiness)
                .map_err(UiNativeEventLoopRunDenial::ClientCallback)
        });
        self.request_physical_signal_redraw();
        if self.finish_client_progress(event_loop, directive) {
            return;
        }
        self.first_frame_presented = true;
    }

    /// Hands the client the physical work that is ready, if any, and
    /// reports whether the client's directive ended this turn.
    pub(super) fn progress_ready_physical_client(&mut self, event_loop: &ActiveEventLoop) -> bool {
        let physical = physical_progression::progress_ready_physical_work(
            &mut self.readiness,
            self.physical_readiness_owner,
            &self.shared,
        );
        physical
            .application_progress_grant()
            .is_some_and(|grant| self.progress_physical_client(event_loop, grant))
    }

    fn progress_physical_client(
        &mut self,
        event_loop: &ActiveEventLoop,
        grant: super::UiNativePhysicalProgressGrant,
    ) -> bool {
        let directive = self.client_or_denied().and_then(|client| {
            client
                .invoke_physical_work_progressed(grant)
                .map_err(UiNativeEventLoopRunDenial::ClientCallback)
        });
        self.finish_client_progress(event_loop, directive)
    }

    fn finish_client_progress(
        &mut self,
        event_loop: &ActiveEventLoop,
        directive: Result<super::UiNativeEventLoopDirective, UiNativeEventLoopRunDenial>,
    ) -> bool {
        let directive = match directive {
            Ok(directive) => directive,
            Err(denial) => {
                self.fail(event_loop, denial);
                return true;
            }
        };
        self.apply_qualified_surface_basis_successor(event_loop)
            || self.apply_client_directive(event_loop, directive)
            || self.notify_native_observations_ready(event_loop)
    }
}
