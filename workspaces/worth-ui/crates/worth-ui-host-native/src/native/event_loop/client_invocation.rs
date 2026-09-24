//! The event loop's sole route to its client's fallible callbacks.
//!
//! A client returns only a [`UiNativeEventLoopClientDenial`]: it has no way
//! to say *which* callback refused, because it has no way to know. The loop
//! does know, so the loop attaches the name here. Keeping all callback pairings
//! in one file is what makes the correspondence auditable at a glance, and
//! `pairing_names_the_invoked_callback` below is what makes it falsifiable.

use super::{
    UiNativeApplicationReadinessGrant, UiNativeEventLoopApplication, UiNativeEventLoopClient,
    UiNativeEventLoopClientCallback as Callback, UiNativeEventLoopClientDenial,
    UiNativeEventLoopClientFailure, UiNativeEventLoopDirective, UiNativeEventLoopRunDenial,
    UiNativeObservationClock, UiNativeObservationReadinessGrant, UiNativeObservationTimeProgress,
    UiNativePhysicalProgressGrant, UiNativeReadinessGrant,
};

type Refused<T> = Result<T, UiNativeEventLoopClientFailure>;

fn refusing<T>(
    callback: Callback,
) -> impl FnOnce(Result<T, UiNativeEventLoopClientDenial>) -> Refused<T> {
    move |outcome| {
        outcome.map_err(|denial| UiNativeEventLoopClientFailure::refused(callback, denial))
    }
}

pub(super) trait UiNativeEventLoopClientInvocation: UiNativeEventLoopClient {
    fn invoke_install_observation_clock(&mut self, clock: UiNativeObservationClock) -> Refused<()> {
        refusing(Callback::InstallObservationClock)(self.install_observation_clock(clock))
    }

    fn invoke_observation_time_ready(&mut self) -> Refused<UiNativeObservationTimeProgress> {
        refusing(Callback::ObservationTimeReady)(self.observation_time_ready())
    }

    fn invoke_install_application_readiness(
        &mut self,
        ports: Vec<crate::UiNativeApplicationReadinessPort>,
    ) -> Refused<()> {
        refusing(Callback::InstallApplicationReadiness)(self.install_application_readiness(ports))
    }

    fn invoke_application_readiness_ready(
        &mut self,
        grant: UiNativeApplicationReadinessGrant,
    ) -> Refused<UiNativeEventLoopDirective> {
        refusing(Callback::ApplicationReadinessReady)(self.application_readiness_ready(grant))
    }

    fn invoke_native_surface_ready(
        &mut self,
        grant: UiNativeReadinessGrant,
    ) -> Refused<UiNativeEventLoopDirective> {
        refusing(Callback::NativeSurfaceReady)(self.native_surface_ready(grant))
    }

    fn invoke_redraw_ready(
        &mut self,
        grant: UiNativeReadinessGrant,
    ) -> Refused<UiNativeEventLoopDirective> {
        refusing(Callback::RedrawReady)(self.redraw_ready(grant))
    }

    fn invoke_physical_work_progressed(
        &mut self,
        grant: UiNativePhysicalProgressGrant,
    ) -> Refused<UiNativeEventLoopDirective> {
        refusing(Callback::PhysicalWorkProgressed)(self.physical_work_progressed(grant))
    }

    fn invoke_native_observations_ready(
        &mut self,
        grant: UiNativeObservationReadinessGrant,
    ) -> Refused<UiNativeEventLoopDirective> {
        refusing(Callback::NativeObservationsReady)(self.native_observations_ready(grant))
    }

    fn invoke_external_close_requested(&mut self) -> Refused<UiNativeEventLoopDirective> {
        refusing(Callback::ExternalCloseRequested)(self.external_close_requested())
    }
    fn invoke_native_input_retention_exhausted(
        &mut self,
        grant: crate::UiNativeInputRecoveryGrant,
    ) -> Refused<(
        crate::UiNativeInputRecoveryAcknowledgement,
        UiNativeEventLoopDirective,
    )> {
        refusing(Callback::NativeInputRetentionExhausted)(
            self.native_input_retention_exhausted(grant),
        )
    }
}

impl<Client: UiNativeEventLoopClient + ?Sized> UiNativeEventLoopClientInvocation for Client {}

impl<Client: UiNativeEventLoopClient> UiNativeEventLoopApplication<Client> {
    /// The loop's client, or the denial for a loop that no longer holds one.
    ///
    /// A loop without a client and a client that refused were both
    /// `ApplicationDriver` before the callback axis existed. They are
    /// different facts, so they are now different denials.
    pub(super) fn client_or_denied(&mut self) -> Result<&mut Client, UiNativeEventLoopRunDenial> {
        self.client
            .as_mut()
            .ok_or(UiNativeEventLoopRunDenial::ApplicationDriver)
    }
}

#[cfg(test)]
#[path = "client_invocation/tests.rs"]
mod tests;
