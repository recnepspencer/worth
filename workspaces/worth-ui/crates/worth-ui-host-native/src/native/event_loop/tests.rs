use std::rc::Rc;

use super::{
    UiNativeEventLoopClient, UiNativeEventLoopClientCleanup, UiNativeEventLoopClientClose,
    UiNativeEventLoopClientFailure, UiNativeEventLoopDirective, UiNativeObservationReadinessGrant,
    UiNativeReadinessGrant,
};
use crate::native::presentation::UiNativePendingExternalObligation;

#[path = "tests/readiness_progress.rs"]
mod readiness_progress;
#[path = "tests/terminal_cleanup.rs"]
mod terminal_cleanup;
#[path = "tests/thread_posture.rs"]
mod thread_posture;

struct CleanupClient {
    completes: bool,
    terminal_resources_complete: bool,
}

struct PendingProbe {
    dropped: Rc<std::cell::Cell<bool>>,
    settles: Rc<std::cell::Cell<bool>>,
}

impl UiNativePendingExternalObligation for PendingProbe {
    fn poll_observation(
        &mut self,
        basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
        _device: Option<&wgpu::Device>,
    ) -> crate::native::physical_work_signal::UiNativePhysicalSignalExternalObservation {
        basis.observe(if self.settles.get() {
            crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Completed
        } else {
            crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Pending
        })
    }
}

impl Drop for PendingProbe {
    fn drop(&mut self) {
        self.dropped.set(true);
    }
}

impl UiNativeEventLoopClient for CleanupClient {
    fn install_observation_clock(
        &mut self,
        _clock: super::UiNativeObservationClock,
    ) -> Result<(), UiNativeEventLoopClientFailure> {
        Ok(())
    }
    fn observation_time_ready(
        &mut self,
    ) -> Result<super::UiNativeObservationTimeProgress, UiNativeEventLoopClientFailure> {
        Ok(super::UiNativeObservationTimeProgress::new(
            None,
            UiNativeEventLoopDirective::Continue,
        ))
    }

    fn native_surface_ready(
        &mut self,
        _grant: super::UiNativeReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientFailure> {
        unreachable!("cleanup proof never enters callbacks")
    }

    fn redraw_ready(
        &mut self,
        _grant: UiNativeReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientFailure> {
        unreachable!("cleanup proof never enters callbacks")
    }

    fn native_observations_ready(
        &mut self,
        _grant: UiNativeObservationReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientFailure> {
        unreachable!("cleanup proof never enters callbacks")
    }

    fn presentation_attribution(&self) -> Option<super::UiNativeClientPresentationAttribution> {
        None
    }

    fn close(self) -> UiNativeEventLoopClientClose {
        if self.completes {
            if self.terminal_resources_complete {
                UiNativeEventLoopClientClose::Complete
            } else {
                UiNativeEventLoopClientClose::CompleteWithObservation(
                    super::UiNativeClientShutdownObservation::from_client(0, false)
                        .with_intent_resources_empty(false),
                )
            }
        } else {
            UiNativeEventLoopClientClose::Incomplete(Box::new(self))
        }
    }
}

impl UiNativeEventLoopClientCleanup for CleanupClient {
    fn retry(self: Box<Self>) -> UiNativeEventLoopClientClose {
        UiNativeEventLoopClientClose::Incomplete(self)
    }
}
