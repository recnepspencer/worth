//! A validated thumb press waiting for an already-submitted physical sample.
//!
//! The press retains its event-time identity. Only its grab offset is deferred:
//! that offset must come from the accepted thumb after the pending sample has
//! physically completed, never from the older pose at report admission.

use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostPointerCaptureEpoch, UiHostPointerIdentity,
    UiMountedInstanceIdentity, UiSurfaceBindingGeneration,
};

use crate::runtime::scroll::{
    chrome::UiScrollChromeAxis, UiScrollOwnerIdentity, UiScrollOwnerIncarnation,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiScrollChromePendingCapture {
    pointer: UiHostPointerIdentity,
    capture_epoch: UiHostPointerCaptureEpoch,
    binding: UiSurfaceBindingGeneration,
    presentation: UiHostObservationPresentationBasis,
    owner: UiScrollOwnerIdentity,
    owner_instance: UiMountedInstanceIdentity,
    incarnation: UiScrollOwnerIncarnation,
    axis: UiScrollChromeAxis,
    press_point: crate::mounting::presentation::UiPlatformPoint,
    latest_point: crate::mounting::presentation::UiPlatformPoint,
    released: bool,
}

impl UiScrollChromePendingCapture {
    pub(crate) const fn new(
        pointer: UiHostPointerIdentity,
        capture_epoch: UiHostPointerCaptureEpoch,
        binding: UiSurfaceBindingGeneration,
        presentation: UiHostObservationPresentationBasis,
        owner: UiScrollOwnerIdentity,
        owner_instance: UiMountedInstanceIdentity,
        incarnation: UiScrollOwnerIncarnation,
        axis: UiScrollChromeAxis,
        press_point: crate::mounting::presentation::UiPlatformPoint,
    ) -> Self {
        Self {
            pointer,
            capture_epoch,
            binding,
            presentation,
            owner,
            owner_instance,
            incarnation,
            axis,
            press_point,
            latest_point: press_point,
            released: false,
        }
    }

    pub(crate) const fn pointer(self) -> UiHostPointerIdentity {
        self.pointer
    }
    pub(crate) const fn capture_epoch(self) -> UiHostPointerCaptureEpoch {
        self.capture_epoch
    }
    pub(crate) const fn binding(self) -> UiSurfaceBindingGeneration {
        self.binding
    }
    pub(crate) const fn presentation(self) -> UiHostObservationPresentationBasis {
        self.presentation
    }
    pub(crate) const fn owner(self) -> UiScrollOwnerIdentity {
        self.owner
    }
    pub(crate) const fn owner_instance(self) -> UiMountedInstanceIdentity {
        self.owner_instance
    }
    pub(crate) const fn incarnation(self) -> UiScrollOwnerIncarnation {
        self.incarnation
    }
    pub(crate) const fn axis(self) -> UiScrollChromeAxis {
        self.axis
    }
    pub(crate) const fn press_point(self) -> crate::mounting::presentation::UiPlatformPoint {
        self.press_point
    }
    pub(crate) const fn latest_point(self) -> crate::mounting::presentation::UiPlatformPoint {
        self.latest_point
    }
    pub(crate) const fn released(self) -> bool {
        self.released
    }

    pub(crate) const fn moved(
        self,
        latest_point: crate::mounting::presentation::UiPlatformPoint,
    ) -> Self {
        Self {
            latest_point,
            ..self
        }
    }

    pub(crate) const fn released_at(
        self,
        latest_point: crate::mounting::presentation::UiPlatformPoint,
    ) -> Self {
        Self {
            latest_point,
            released: true,
            ..self
        }
    }
}
