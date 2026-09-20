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

/// The single latch slot the pointer gesture owner holds.
#[derive(Debug, Default)]
pub(crate) struct UiScrollChromeLatchState {
    held: Option<UiScrollChromeLatch>,
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
    /// Take the latch for a thumb press. Refused while another drag holds it,
    /// so two pointers never move one thumb.
    pub(crate) fn latch(
        &mut self,
        latch: UiScrollChromeLatch,
    ) -> Result<UiScrollChromeLatch, UiScrollChromeLatchDenial> {
        if self.held.is_some() {
            return Err(UiScrollChromeLatchDenial::AlreadyLatched);
        }
        self.held = Some(latch);
        Ok(latch)
    }

    pub(crate) const fn held(&self) -> Option<UiScrollChromeLatch> {
        self.held
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
        self.held = Some(moved);
        Ok(moved)
    }

    /// End the drag on release, handing back the latch that ended.
    pub(crate) fn release(
        &mut self,
        pointer: UiHostPointerIdentity,
        capture_epoch: UiHostPointerCaptureEpoch,
    ) -> Result<UiScrollChromeLatch, UiScrollChromeLatchDenial> {
        let held = self.matching(pointer, capture_epoch)?;
        self.held = None;
        Ok(held)
    }

    /// Drop the drag because a lifecycle owner ended it. Reports the latch that
    /// was dropped so the caller can settle the appearance it was holding.
    pub(crate) fn cancel(&mut self) -> Option<UiScrollChromeLatch> {
        self.held.take()
    }

    /// Drop the drag when its own region occurrence is gone.
    pub(crate) fn cancel_instance(
        &mut self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<UiScrollChromeLatch> {
        self.held
            .filter(|held| held.owner_instance == instance)
            .and_then(|_| self.held.take())
    }

    /// Drop the drag when the surface binding it was presented under is gone.
    pub(crate) fn cancel_binding(
        &mut self,
        binding: UiSurfaceBindingGeneration,
    ) -> Option<UiScrollChromeLatch> {
        self.held
            .filter(|held| held.binding == binding)
            .and_then(|_| self.held.take())
    }

    /// Whether a wheel or a key on this axis of this region must be ignored,
    /// because a drag already owns that axis for the length of its capture.
    pub(crate) fn suppresses_axis(
        &self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        axis: UiScrollChromeAxis,
    ) -> bool {
        self.held
            .is_some_and(|held| held.owner == owner && held.axis == axis)
    }

    fn matching(
        &self,
        pointer: UiHostPointerIdentity,
        capture_epoch: UiHostPointerCaptureEpoch,
    ) -> Result<UiScrollChromeLatch, UiScrollChromeLatchDenial> {
        let held = self.held.ok_or(UiScrollChromeLatchDenial::NotLatched)?;
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
mod tests {
    use super::*;

    fn owner(plan_region_index: u32) -> crate::runtime::scroll::UiScrollOwnerIdentity {
        crate::runtime::scroll::UiScrollOwnerIdentity::declared_region(
            worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().expect("surface"),
            crate::graph::UiGraphNodeIdentity::new(3_161),
            1,
            plan_region_index,
        )
    }

    fn latch_for(
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        axis: UiScrollChromeAxis,
        pointer: u64,
        epoch: u64,
    ) -> UiScrollChromeLatch {
        UiScrollChromeLatch::press(
            UiHostPointerIdentity::new(pointer),
            UiHostPointerCaptureEpoch::new(epoch),
            UiSurfaceBindingGeneration::mint_unbound().expect("binding"),
            owner,
            worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().expect("instance"),
            crate::runtime::scroll::UiScrollOwnerIncarnation::new(1).expect("incarnation"),
            axis,
            4.0,
        )
    }

    /// A drag owns its axis for the length of its capture, so a wheel on that
    /// axis of that region is ignored while it runs. The other axis, and every
    /// other region, keep scrolling.
    #[test]
    fn a_captured_axis_suppresses_only_its_own_axis_of_its_own_region() {
        let dragged = owner(4);
        let other = owner(5);
        let mut state = UiScrollChromeLatchState::default();
        state
            .latch(latch_for(dragged, UiScrollChromeAxis::Block, 1, 1))
            .expect("the first thumb press latches");

        assert!(state.suppresses_axis(dragged, UiScrollChromeAxis::Block));
        assert!(!state.suppresses_axis(dragged, UiScrollChromeAxis::Inline));
        assert!(!state.suppresses_axis(other, UiScrollChromeAxis::Block));
    }

    /// Nothing is suppressed when nothing is captured, so a released drag stops
    /// silencing the wheel it was silencing.
    #[test]
    fn a_released_drag_suppresses_nothing() {
        let dragged = owner(4);
        let mut state = UiScrollChromeLatchState::default();
        state
            .latch(latch_for(dragged, UiScrollChromeAxis::Block, 1, 1))
            .expect("latched");
        state
            .release(
                UiHostPointerIdentity::new(1),
                UiHostPointerCaptureEpoch::new(1),
            )
            .expect("the latching pointer releases it");
        assert!(!state.suppresses_axis(dragged, UiScrollChromeAxis::Block));
    }

    /// Leaving the gutter changes the posture and keeps the capture: a drag
    /// that wandered off the track is still the drag that owns the thumb.
    #[test]
    fn dragging_outside_the_gutter_keeps_the_capture() {
        let mut state = UiScrollChromeLatchState::default();
        state
            .latch(latch_for(owner(4), UiScrollChromeAxis::Block, 1, 1))
            .expect("latched");
        let moved = state
            .moved(
                UiHostPointerIdentity::new(1),
                UiHostPointerCaptureEpoch::new(1),
                UiScrollChromeDragPosture::OutsideGutter,
            )
            .expect("a move under the same capture is admitted");
        assert_eq!(moved.posture(), UiScrollChromeDragPosture::OutsideGutter);
        assert_eq!(moved.grab_offset_logical_points(), 4.0);
        assert!(state.held().is_some());
    }

    /// Two pointers never move one thumb, and a report from a pointer that does
    /// not hold the latch is refused rather than redirected.
    #[test]
    fn only_the_latching_pointer_under_its_own_capture_moves_the_thumb() {
        let mut state = UiScrollChromeLatchState::default();
        state
            .latch(latch_for(owner(4), UiScrollChromeAxis::Block, 1, 1))
            .expect("latched");
        assert_eq!(
            state.latch(latch_for(owner(5), UiScrollChromeAxis::Inline, 2, 1)),
            Err(UiScrollChromeLatchDenial::AlreadyLatched)
        );
        assert_eq!(
            state.moved(
                UiHostPointerIdentity::new(2),
                UiHostPointerCaptureEpoch::new(1),
                UiScrollChromeDragPosture::InsideGutter,
            ),
            Err(UiScrollChromeLatchDenial::PointerMismatch)
        );
        assert_eq!(
            state.moved(
                UiHostPointerIdentity::new(1),
                UiHostPointerCaptureEpoch::new(2),
                UiScrollChromeDragPosture::InsideGutter,
            ),
            Err(UiScrollChromeLatchDenial::CaptureEpochChanged)
        );
    }

    /// A modality change ends a captured thumb exactly as it ends a press.
    #[test]
    fn a_cancellation_ends_the_drag() {
        let dragged = owner(4);
        let mut state = UiScrollChromeLatchState::default();
        state
            .latch(latch_for(dragged, UiScrollChromeAxis::Block, 1, 1))
            .expect("latched");
        assert!(state.cancel().is_some());
        assert!(state.held().is_none());
        assert!(!state.suppresses_axis(dragged, UiScrollChromeAxis::Block));
        assert_eq!(
            state.release(
                UiHostPointerIdentity::new(1),
                UiHostPointerCaptureEpoch::new(1)
            ),
            Err(UiScrollChromeLatchDenial::NotLatched)
        );
    }
}
