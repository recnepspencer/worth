use super::UiNativeEventLoopClientInvocation;
use crate::native::event_loop::{
    UiNativeApplicationReadinessGrant, UiNativeClientPresentationAttribution,
    UiNativeEventLoopClient, UiNativeEventLoopClientCallback as Callback,
    UiNativeEventLoopClientClose, UiNativeEventLoopClientDenial as Denial,
    UiNativeEventLoopDirective, UiNativeObservationClock, UiNativeObservationReadinessGrant,
    UiNativeObservationTimeProgress, UiNativePhysicalProgressClass, UiNativePhysicalProgressGrant,
    UiNativeReadinessGrant, UiNativeReducedMotionPosture,
};

/// A client that refuses every callback with the same denial. Only the
/// callback name can differ between the nine results, so the assertions
/// below are testing exactly the pairing and nothing else.
struct RefusingClient;

impl UiNativeEventLoopClient for RefusingClient {
    fn install_observation_clock(
        &mut self,
        _clock: UiNativeObservationClock,
    ) -> Result<(), Denial> {
        Err(Denial::Unsupported)
    }

    fn observation_time_ready(&mut self) -> Result<UiNativeObservationTimeProgress, Denial> {
        Err(Denial::Unsupported)
    }

    fn install_application_readiness(
        &mut self,
        _ports: Vec<crate::UiNativeApplicationReadinessPort>,
    ) -> Result<(), Denial> {
        Err(Denial::Unsupported)
    }

    fn application_readiness_ready(
        &mut self,
        _grant: UiNativeApplicationReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, Denial> {
        Err(Denial::Unsupported)
    }

    fn native_surface_ready(
        &mut self,
        _grant: UiNativeReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, Denial> {
        Err(Denial::Unsupported)
    }

    fn redraw_ready(
        &mut self,
        _grant: UiNativeReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, Denial> {
        Err(Denial::Unsupported)
    }

    fn physical_work_progressed(
        &mut self,
        _grant: UiNativePhysicalProgressGrant,
    ) -> Result<UiNativeEventLoopDirective, Denial> {
        Err(Denial::Unsupported)
    }

    fn native_observations_ready(
        &mut self,
        _grant: UiNativeObservationReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, Denial> {
        Err(Denial::Unsupported)
    }

    fn external_close_requested(&mut self) -> Result<UiNativeEventLoopDirective, Denial> {
        Err(Denial::Unsupported)
    }

    fn presentation_attribution(
        &self,
        _observed: &crate::native::UiNativeRetainedFrameObservation,
    ) -> Option<UiNativeClientPresentationAttribution> {
        None
    }

    fn close(self) -> UiNativeEventLoopClientClose {
        UiNativeEventLoopClientClose::Complete
    }
}

fn readiness_grant() -> UiNativeReadinessGrant {
    UiNativeReadinessGrant::issued(0, 0, 1_000, [1, 1])
}

/// Each wrapper must report the callback it actually invoked. A wrapper
/// pasted from its neighbour and left naming that neighbour is the mistake
/// this test exists to kill, and it is the only mistake the one-file layout
/// makes easy.
#[test]
fn pairing_names_the_invoked_callback() {
    let mut client = RefusingClient;
    let clock = crate::native::event_loop::physical_clock::UiNativePhysicalEventClock::new()
        .observation_clock();
    let named: [(Callback, Denial); 9] = [
        refusal(client.invoke_install_observation_clock(clock)),
        refusal(client.invoke_observation_time_ready()),
        refusal(client.invoke_install_application_readiness(Vec::new())),
        refusal(client.invoke_application_readiness_ready(
            UiNativeApplicationReadinessGrant::issued(
                0,
                0,
                0,
                UiNativeReducedMotionPosture::Unavailable,
            ),
        )),
        refusal(client.invoke_native_surface_ready(readiness_grant())),
        refusal(client.invoke_redraw_ready(readiness_grant())),
        refusal(
            client.invoke_physical_work_progressed(UiNativePhysicalProgressGrant::issued(
                UiNativePhysicalProgressClass::Presentation,
                super::super::contract::UiNativePhysicalProgressCorrelation::Unattributed,
            )),
        ),
        refusal(client.invoke_native_observations_ready(
            UiNativeObservationReadinessGrant::issued(
                0,
                crate::native::event_loop::UiNativeInputReachability::default(),
            ),
        )),
        refusal(client.invoke_external_close_requested()),
    ];
    assert_eq!(
        named.map(|(callback, _)| callback),
        [
            Callback::InstallObservationClock,
            Callback::ObservationTimeReady,
            Callback::InstallApplicationReadiness,
            Callback::ApplicationReadinessReady,
            Callback::NativeSurfaceReady,
            Callback::RedrawReady,
            Callback::PhysicalWorkProgressed,
            Callback::NativeObservationsReady,
            Callback::ExternalCloseRequested,
        ]
    );
    assert!(named
        .iter()
        .all(|(_, denial)| *denial == Denial::Unsupported));
}

fn refusal<T>(
    outcome: Result<T, crate::native::event_loop::UiNativeEventLoopClientFailure>,
) -> (Callback, Denial) {
    let failure = outcome
        .err()
        .expect("the refusing client denies every callback");
    (failure.callback(), failure.denial())
}
