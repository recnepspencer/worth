//! Publishing a prepared Scroll settle from the session, and installing the
//! Motion track it commits.
//!
//! This is the whole of what the session decides: which frame the settle is
//! published against, and that the Motion service is there to receive it. Every
//! other refusal belongs to the service-proposal lane and arrives as the same
//! product-facing stop every other proposal stops with. In particular "this
//! application declares no Motion service" is not decided here twice: an
//! application with no Motion installation stops at
//! [`UiScrollSettleStop::MotionUnavailable`], and one whose service support
//! does not admit the family is refused by preflight as
//! `UnsupportedFamily(Motion)` inside the lane.

use super::super::WorthUiActiveApplicationSession;
use crate::runtime::scroll::UiScrollSettleStop;

impl WorthUiActiveApplicationSession {
    /// Publish `transition` through the Scroll settle service-proposal lane and
    /// install the committed track into mounted presentation sampling.
    ///
    /// A settle submits into the publication already on screen: the notch
    /// changed no mounted content, so re-minting a frame would claim work that
    /// never happened.
    pub(in crate::facade::entry) fn publish_scroll_settle(
        &mut self,
        transition: &crate::runtime::scroll::UiPreparedScrollSettleTransition,
    ) -> Result<(), UiScrollSettleStop> {
        let application = self.active_generation_identity();
        let publication = self
            .mounted
            .current_publication()
            .cloned()
            .ok_or(UiScrollSettleStop::NoPublishedFrame)?;
        let motion = self
            .motion
            .as_mut()
            .ok_or(UiScrollSettleStop::MotionUnavailable)?;
        let commit = self
            .application
            .publish_scroll_settle_service_proposal(transition, &publication, application, motion)
            .map_err(|denial| UiScrollSettleStop::Proposal(denial.into()))?;
        self.install_committed_motion(Some(commit));
        Ok(())
    }

    /// Why the most recent coarse notch's settle went unpublished, if it did.
    ///
    /// The wheel observation itself reports only `SettleUnpublished`; this is
    /// the reason behind it, kept until the next coarse notch settles or stops.
    /// A notch whose settle published clears it.
    pub fn last_scroll_settle_stop(&self) -> Option<&UiScrollSettleStop> {
        self.last_scroll_settle_stop.as_ref()
    }
}
