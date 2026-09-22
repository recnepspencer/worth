use winit::event_loop::ActiveEventLoop;

use super::client_invocation::UiNativeEventLoopClientInvocation;
use super::{
    UiNativeApplicationReadinessGrant, UiNativeEventLoopApplication, UiNativeEventLoopClient,
    UiNativeEventLoopRunDenial,
};

impl<Client: UiNativeEventLoopClient> UiNativeEventLoopApplication<Client> {
    pub(super) fn progress_application_readiness(&mut self, event_loop: &ActiveEventLoop) {
        self.readiness_signals = self.readiness_signals.saturating_add(1);
        for ordinal in 0..self.application_readiness_owners.len() {
            let owner = self.application_readiness_owners[ordinal];
            let Ok(ready) = self.readiness.take_level(owner) else {
                continue;
            };
            let Ok(owner_ordinal) = u8::try_from(ordinal) else {
                return self.fail(event_loop, UiNativeEventLoopRunDenial::ApplicationDriver);
            };
            let physical_tick = self.physical_clock.current_tick();
            let reduced_motion = crate::native::platform::observe_reduced_motion_posture();
            let directive = self.client_or_denied().and_then(|client| {
                client
                    .invoke_application_readiness_ready(UiNativeApplicationReadinessGrant::issued(
                        owner_ordinal,
                        ready.generation(),
                        physical_tick,
                        reduced_motion,
                    ))
                    .map_err(UiNativeEventLoopRunDenial::ClientCallback)
            });
            let directive = match directive {
                Ok(directive) => directive,
                Err(denial) => return self.fail(event_loop, denial),
            };
            // An application readiness callback can submit text-atlas or
            // presentation work through the runtime shell. Probe the physical
            // owner before returning to the event loop so that work receives
            // its independent redraw wake instead of waiting for unrelated
            // ordinary readiness.
            self.request_physical_signal_redraw();
            if self.apply_client_directive(event_loop, directive) {
                return;
            }
        }
    }
}
