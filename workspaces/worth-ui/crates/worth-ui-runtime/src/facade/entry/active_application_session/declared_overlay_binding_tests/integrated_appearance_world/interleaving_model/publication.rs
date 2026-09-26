//! How the interleaving model publishes: as issued, refused before effects,
//! or held open by the host until a later step completes it.

use super::Model;
use crate::certification_support::ScriptedPresentationHost;
use crate::mounting::UiMountedFrameOutcome;
use worth_ui_host_contract::UiPresentationDeadline;

impl Model {
    pub(super) fn publish(&mut self) -> Option<super::Step> {
        if self.in_flight.is_some() {
            return None;
        }
        self.publish_as_issued();
        Some(super::Step::Publish)
    }

    /// What the shell does once presentation is pending: a settle that
    /// changed what the host must hold, a staged layout, or a staged direct
    /// placement wakes it.
    pub(super) fn publish_owed(&mut self) {
        if self.in_flight.is_none()
            && self
                .scroll
                .world
                .session
                .mounted
                .projection_changes_pending()
        {
            self.publish_as_issued();
        }
    }

    /// Publish the surface; the host paints exactly what the frame carries.
    pub(super) fn publish_as_issued(&mut self) {
        let outcome = self.present_surface(ScriptedPresentationHost::push_native_display_as_issued);
        assert!(
            matches!(outcome, UiMountedFrameOutcome::Published(_)),
            "the host takes the publication as issued"
        );
        self.staged = false;
    }

    /// Prepare the surface's frame and present it, the host answering each
    /// surface it carries with `answer`.
    fn present_surface(&mut self, answer: fn(&ScriptedPresentationHost)) -> UiMountedFrameOutcome {
        let surface = self.scroll.surface();
        let world = &mut self.scroll.world;
        let frame = world.prepare_surface(surface);
        for _ in frame.surfaces() {
            answer(&world.host);
        }
        world.session.present_prepared_mounted_frame_internal(
            frame,
            UiPresentationDeadline::at_tick(u64::MAX),
            self.tick,
        )
    }

    pub(super) fn reject_publication(&mut self) -> Option<super::Step> {
        if self.in_flight.is_some() {
            return None;
        }
        let outcome = self.present_surface(ScriptedPresentationHost::push_rejected);
        assert!(
            matches!(outcome, UiMountedFrameOutcome::RejectedBeforeEffects(_)),
            "a host refusal before effects is reported as one"
        );
        Some(super::Step::RejectPublication)
    }

    pub(super) fn begin_in_flight(&mut self) -> Option<super::Step> {
        if self.in_flight.is_some() {
            return None;
        }
        self.in_flight = Some(self.scroll.hold_presentation_open(self.tick));
        Some(super::Step::BeginInFlight)
    }

    pub(super) fn complete_in_flight(&mut self) -> Option<super::Step> {
        let pending = self.in_flight.take()?;
        self.scroll.complete(pending, self.tick);
        self.staged = false;
        Some(super::Step::CompleteInFlight)
    }
}
