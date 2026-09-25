//! Finish an admitted thumb press only after its in-flight pixels have settled.

use crate::runtime::interaction::gesture::UiScrollChromeLatch;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::facade::entry) enum UiScrollChromePendingCompletion {
    Idle,
    AwaitingPhysical,
    AwaitingReconciliation,
    Captured,
    Released,
    MoveDeferred(super::scroll_chrome_interaction::UiScrollChromeInteractionDenial),
    Cancelled,
}

impl super::WorthUiActiveApplicationSession {
    pub(in crate::facade::entry) fn finish_pending_scroll_chrome_capture(
        &mut self,
    ) -> UiScrollChromePendingCompletion {
        use UiScrollChromePendingCompletion as Outcome;

        let Some(pending) = self.interaction.scroll_chrome_latch_mut().pending() else {
            return Outcome::Idle;
        };
        if self.mounted.motion_sample_presentation_pending() {
            return Outcome::AwaitingPhysical;
        }
        let surface = pending.owner().semantic_surface();
        if self
            .mounted
            .current_presentation_for_surface(surface)
            .filter(|current| current.host_surface() == pending.presentation().host_surface())
            .is_none()
            || self.mounted.current_surface_for_binding(pending.binding()) != Some(surface)
        {
            self.interaction.scroll_chrome_latch_mut().take_pending();
            return Outcome::Cancelled;
        }
        let Some(region) = self
            .presented_scroll_chrome_facts(surface)
            .into_iter()
            .find(|region| region.owner() == pending.owner())
            .filter(|region| {
                region.owner_instance() == pending.owner_instance()
                    && region.incarnation() == pending.incarnation()
            })
        else {
            self.interaction.scroll_chrome_latch_mut().take_pending();
            return Outcome::Cancelled;
        };
        let target = super::scroll_direct_control::scroll_content_motion_target(
            region.owner(),
            region.mounted_instance(),
        );
        let owner_offset = self
            .scroll
            .as_ref()
            .and_then(|scroll| scroll.offset(region.owner(), region.incarnation()).ok());
        if !self.accepted_scroll_sample_settled(
            target,
            surface,
            region.mounted_offset(),
            owner_offset,
        ) {
            return Outcome::AwaitingReconciliation;
        }
        let Some(thumb) = region.facts().axis(pending.axis()).map(|axis| axis.thumb()) else {
            self.interaction.scroll_chrome_latch_mut().take_pending();
            return Outcome::Cancelled;
        };
        let grab = crate::runtime::scroll::chrome::grab_offset_logical_points(
            pending.axis(),
            thumb,
            pending.press_point(),
        );
        if pending.latest_point() != pending.press_point() {
            let Some(placed) = region.facts().offset_for_thumb_position(
                pending.axis(),
                pending.latest_point(),
                grab,
                region.mounted_offset(),
            ) else {
                self.interaction.scroll_chrome_latch_mut().take_pending();
                return Outcome::Cancelled;
            };
            if let Err(denial) = self.place_scroll_chrome_offset(
                region.owner(),
                region.incarnation(),
                region.mounted_instance(),
                region.slot(),
                placed,
                crate::runtime::scroll::UiScrollDeltaCause::ChromeThumbDrag,
            ) {
                // Only outstanding publication work can become placeable on
                // the next frame. Structural geometry refusals (including
                // exhausted revisions) cannot recover by spinning readiness.
                if retryable_placement_denial(denial) {
                    return Outcome::MoveDeferred(denial);
                }
                self.interaction.scroll_chrome_latch_mut().take_pending();
                return Outcome::Cancelled;
            }
        }
        let latch = UiScrollChromeLatch::press(
            pending.pointer(),
            pending.capture_epoch(),
            pending.binding(),
            pending.owner(),
            pending.owner_instance(),
            pending.incarnation(),
            pending.axis(),
            grab,
        );
        self.interaction.scroll_chrome_latch_mut().take_pending();
        self.interaction
            .scroll_chrome_latch_mut()
            .latch(latch)
            .expect("pending capture and held latch are mutually exclusive");
        self.take_direct_scroll_control(region.owner(), region.mounted_instance());
        if pending.released() {
            self.release_scroll_chrome(pending.pointer(), pending.capture_epoch())
                .expect("pending capture retained its pointer and capture epoch");
            Outcome::Released
        } else {
            Outcome::Captured
        }
    }
}

fn retryable_placement_denial(
    denial: super::scroll_chrome_interaction::UiScrollChromeInteractionDenial,
) -> bool {
    use super::scroll_chrome_interaction::UiScrollChromeInteractionDenial as Denial;
    use crate::mounting::UiMountedOccurrenceGeometryDenial as Geometry;
    matches!(
        denial,
        Denial::UnpresentedLayout
            | Denial::Geometry(Geometry::PresentationInFlight | Geometry::UnpublishedScrollEffect)
    )
}

#[cfg(test)]
mod tests {
    use super::super::scroll_chrome_interaction::UiScrollChromeInteractionDenial as Denial;
    use super::*;
    use crate::mounting::UiMountedOccurrenceGeometryDenial as Geometry;

    #[test]
    fn revision_exhaustion_does_not_arm_unbounded_capture_retry() {
        assert!(retryable_placement_denial(Denial::UnpresentedLayout));
        assert!(retryable_placement_denial(Denial::Geometry(
            Geometry::PresentationInFlight
        )));
        assert!(!retryable_placement_denial(Denial::Geometry(
            Geometry::StateRevisionExhausted
        )));
    }
}
