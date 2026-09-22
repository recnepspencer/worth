use worth_ui_host_native::{
    UiNativeClientPresentationAttribution, UiNativeEventLoopClient, UiNativeEventLoopClientClose,
    UiNativeEventLoopClientDenial, UiNativeEventLoopDirective, UiNativeObservationClock,
    UiNativeObservationReadinessGrant, UiNativeObservationTimeProgress,
    UiNativeReadinessGrant, UiNativeRetainedFrameObservation,
};
use worth_ui_native_platform::UiPreparedNativePlatform;

struct ForgedNativeClient;

impl UiNativeEventLoopClient for ForgedNativeClient {
    fn install_observation_clock(
        &mut self,
        _clock: UiNativeObservationClock,
    ) -> Result<(), UiNativeEventLoopClientDenial> {
        Ok(())
    }

    fn observation_time_ready(
        &mut self,
    ) -> Result<UiNativeObservationTimeProgress, UiNativeEventLoopClientDenial> {
        Ok(UiNativeObservationTimeProgress::new(
            None,
            UiNativeEventLoopDirective::Continue,
        ))
    }

    fn native_surface_ready(
        &mut self,
        _grant: UiNativeReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientDenial> {
        Ok(UiNativeEventLoopDirective::Continue)
    }

    fn redraw_ready(
        &mut self,
        _grant: UiNativeReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientDenial> {
        Ok(UiNativeEventLoopDirective::Close)
    }

    fn native_observations_ready(
        &mut self,
        _grant: UiNativeObservationReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientDenial> {
        Ok(UiNativeEventLoopDirective::Close)
    }

    fn presentation_attribution(
        &self,
        _observed: &UiNativeRetainedFrameObservation,
    ) -> Option<UiNativeClientPresentationAttribution> {
        None
    }

    fn close(self) -> UiNativeEventLoopClientClose {
        UiNativeEventLoopClientClose::Complete
    }
}

fn substitute_product_driver(platform: UiPreparedNativePlatform) {
    let _ = platform.run(ForgedNativeClient);
}

fn main() {}
