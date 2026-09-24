//! The latch a thumb drag holds for its lifetime.
//!
//! A thumb press captures the pointer and remembers three things the rest of
//! the drag cannot recompute: which axis was grabbed, how far into the thumb
//! the grab landed, and under which capture epoch. Every later move reads the
//! same grab offset, which is what makes the drag direct — the thumb keeps the
//! exact spot under the pointer it had at the press instead of jumping its
//! centre there.
//!
//! The latch lives beside the pointer gesture table rather than inside it,
//! because chrome is not a mounted node and cannot be an interaction target.
//! It is cleared by the same lifecycle owners that stop gestures, so capture
//! introduces no lifetime rule of its own: a modality change, a lost focus or
//! an unmounted owner ends a drag exactly when it ends a press.

use worth_ui_host_contract::{
    UiHostPointerCaptureEpoch, UiHostPointerIdentity, UiSurfaceBindingGeneration,
};

use crate::runtime::scroll::chrome::{UiScrollChromeAxis, UiScrollChromeDragPosture};

#[path = "scroll_chrome_latch/pending_capture.rs"]
mod pending_capture;
pub(crate) use pending_capture::UiScrollChromePendingCapture;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollChromeLatchDenial {
    /// One pointer already drags a thumb. A second thumb press is refused
    /// rather than silently stealing the first drag's capture.
    AlreadyLatched,
    /// No drag is in progress, so there is nothing to move or release.
    NotLatched,
    /// The report came from a pointer that does not hold the latch.
    PointerMismatch,
    /// The host re-captured this pointer, so the latch predates the capture the
    /// report belongs to and cannot answer for it.
    CaptureEpochChanged,
}

/// One thumb drag in progress.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiScrollChromeLatch {
    pointer: UiHostPointerIdentity,
    capture_epoch: UiHostPointerCaptureEpoch,
    binding: UiSurfaceBindingGeneration,
    owner: crate::runtime::scroll::UiScrollOwnerIdentity,
    owner_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    incarnation: crate::runtime::scroll::UiScrollOwnerIncarnation,
    axis: UiScrollChromeAxis,
    grab_offset_logical_points: f32,
    posture: UiScrollChromeDragPosture,
}

/// The single latch slot the pointer gesture owner holds: a thumb press is
/// either still waiting for its capture or holds the drag, never both.
#[derive(Debug)]
pub(crate) enum UiScrollChromeLatchState {
    Idle,
    Pending(UiScrollChromePendingCapture),
    Held(UiScrollChromeLatch),
}

impl UiScrollChromeLatch {
    pub(crate) const fn press(
        pointer: UiHostPointerIdentity,
        capture_epoch: UiHostPointerCaptureEpoch,
        binding: UiSurfaceBindingGeneration,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        owner_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        incarnation: crate::runtime::scroll::UiScrollOwnerIncarnation,
        axis: UiScrollChromeAxis,
        grab_offset_logical_points: f32,
    ) -> Self {
        Self {
            pointer,
            capture_epoch,
            binding,
            owner,
            owner_instance,
            incarnation,
            axis,
            grab_offset_logical_points,
            posture: UiScrollChromeDragPosture::InsideGutter,
        }
    }

    pub(crate) const fn pointer(self) -> UiHostPointerIdentity {
        self.pointer
    }

    pub(crate) const fn capture_epoch(self) -> UiHostPointerCaptureEpoch {
        self.capture_epoch
    }

    pub(crate) const fn owner(self) -> crate::runtime::scroll::UiScrollOwnerIdentity {
        self.owner
    }

    pub(crate) const fn incarnation(self) -> crate::runtime::scroll::UiScrollOwnerIncarnation {
        self.incarnation
    }

    pub(crate) const fn axis(self) -> UiScrollChromeAxis {
        self.axis
    }

    pub(crate) const fn grab_offset_logical_points(self) -> f32 {
        self.grab_offset_logical_points
    }

    pub(crate) const fn posture(self) -> UiScrollChromeDragPosture {
        self.posture
    }
}

impl UiScrollChromeLatchState {
    /// No thumb press holds or awaits the latch.
    pub(crate) const fn idle() -> Self {
        Self::Idle
    }

    /// Take the latch for a thumb press. Refused while another drag holds it,
    /// so two pointers never move one thumb.
    pub(crate) fn latch(
        &mut self,
        latch: UiScrollChromeLatch,
    ) -> Result<UiScrollChromeLatch, UiScrollChromeLatchDenial> {
        let Self::Idle = self else {
            return Err(UiScrollChromeLatchDenial::AlreadyLatched);
        };
        *self = Self::Held(latch);
        Ok(latch)
    }

    pub(crate) const fn held(&self) -> Option<UiScrollChromeLatch> {
        match self {
            Self::Held(held) => Some(*held),
            Self::Idle | Self::Pending(_) => None,
        }
    }

    pub(crate) const fn pending(&self) -> Option<UiScrollChromePendingCapture> {
        match self {
            Self::Pending(pending) => Some(*pending),
            Self::Idle | Self::Held(_) => None,
        }
    }

    pub(crate) fn begin_pending(
        &mut self,
        pending: UiScrollChromePendingCapture,
    ) -> Result<(), UiScrollChromeLatchDenial> {
        let Self::Idle = self else {
            return Err(UiScrollChromeLatchDenial::AlreadyLatched);
        };
        *self = Self::Pending(pending);
        Ok(())
    }

    pub(crate) fn take_pending(&mut self) -> Option<UiScrollChromePendingCapture> {
        let pending = self.pending()?;
        *self = Self::Idle;
        Some(pending)
    }

    pub(crate) fn move_pending(
        &mut self,
        pointer: UiHostPointerIdentity,
        capture_epoch: UiHostPointerCaptureEpoch,
        point: [f32; 2],
        released: bool,
    ) -> Result<UiScrollChromePendingCapture, UiScrollChromeLatchDenial> {
        let pending = self
            .pending()
            .ok_or(UiScrollChromeLatchDenial::NotLatched)?;
        if pending.pointer() != pointer {
            return Err(UiScrollChromeLatchDenial::PointerMismatch);
        }
        if pending.capture_epoch() != capture_epoch {
            return Err(UiScrollChromeLatchDenial::CaptureEpochChanged);
        }
        if pending.released() {
            return Err(UiScrollChromeLatchDenial::NotLatched);
        }
        let moved = if released {
            pending.released_at(point)
        } else {
            pending.moved(point)
        };
        *self = Self::Pending(moved);
        Ok(moved)
    }

    /// Record where the drag now is and hand back the latch the move must be
    /// mapped through. Leaving the gutter changes the posture, never the
    /// capture: the drag keeps running outside the track.
    pub(crate) fn moved(
        &mut self,
        pointer: UiHostPointerIdentity,
        capture_epoch: UiHostPointerCaptureEpoch,
        posture: UiScrollChromeDragPosture,
    ) -> Result<UiScrollChromeLatch, UiScrollChromeLatchDenial> {
        let held = self.matching(pointer, capture_epoch)?;
        let moved = UiScrollChromeLatch { posture, ..held };
        *self = Self::Held(moved);
        Ok(moved)
    }

    /// End the drag on release, handing back the latch that ended.
    pub(crate) fn release(
        &mut self,
        pointer: UiHostPointerIdentity,
        capture_epoch: UiHostPointerCaptureEpoch,
    ) -> Result<UiScrollChromeLatch, UiScrollChromeLatchDenial> {
        let held = self.matching(pointer, capture_epoch)?;
        *self = Self::Idle;
        Ok(held)
    }

    /// Drop the drag because a lifecycle owner ended it. Reports the latch that
    /// was dropped so the caller can settle the appearance it was holding.
    pub(crate) fn cancel(&mut self) -> Option<UiScrollChromeLatch> {
        self.cancel_where(|_| true, |_| true)
    }

    /// Drop the drag when its own region occurrence is gone.
    pub(crate) fn cancel_instance(
        &mut self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<UiScrollChromeLatch> {
        self.cancel_where(
            |pending| pending.owner_instance() == instance,
            |held| held.owner_instance == instance,
        )
    }

    /// Drop the drag when the surface binding it was presented under is gone.
    pub(crate) fn cancel_binding(
        &mut self,
        binding: UiSurfaceBindingGeneration,
    ) -> Option<UiScrollChromeLatch> {
        self.cancel_where(
            |pending| pending.binding() == binding,
            |held| held.binding == binding,
        )
    }

    /// Return to idle when the slot's occupant matches, reporting a held
    /// latch that was dropped.
    fn cancel_where(
        &mut self,
        pending_matches: impl FnOnce(&UiScrollChromePendingCapture) -> bool,
        held_matches: impl FnOnce(&UiScrollChromeLatch) -> bool,
    ) -> Option<UiScrollChromeLatch> {
        let (cancel, dropped) = match self {
            Self::Idle => (false, None),
            Self::Pending(pending) => (pending_matches(pending), None),
            Self::Held(held) => {
                let matches = held_matches(held);
                (matches, matches.then_some(*held))
            }
        };
        if cancel {
            *self = Self::Idle;
        }
        dropped
    }

    /// Whether a wheel or a key on this axis of this region must be ignored,
    /// because a drag already owns that axis for the length of its capture.
    pub(crate) fn suppresses_axis(
        &self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        axis: UiScrollChromeAxis,
    ) -> bool {
        match self {
            Self::Idle => false,
            Self::Held(held) => held.owner == owner && held.axis == axis,
            Self::Pending(pending) => {
                !pending.released() && pending.owner() == owner && pending.axis() == axis
            }
        }
    }

    fn matching(
        &self,
        pointer: UiHostPointerIdentity,
        capture_epoch: UiHostPointerCaptureEpoch,
    ) -> Result<UiScrollChromeLatch, UiScrollChromeLatchDenial> {
        let held = self.held().ok_or(UiScrollChromeLatchDenial::NotLatched)?;
        if held.pointer != pointer {
            return Err(UiScrollChromeLatchDenial::PointerMismatch);
        }
        if held.capture_epoch != capture_epoch {
            return Err(UiScrollChromeLatchDenial::CaptureEpochChanged);
        }
        Ok(held)
    }
}

#[cfg(test)]
#[path = "scroll_chrome_latch/tests.rs"]
mod tests;
