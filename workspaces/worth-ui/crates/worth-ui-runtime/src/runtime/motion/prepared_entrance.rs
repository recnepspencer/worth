//! The first physical sample of an exact, derived Portal entrance proposal.
use super::UiMotionTargetIdentity;
use crate::mounting::presentation::UiPublishedRect;

#[derive(Clone, Copy)]
pub(crate) struct UiPreparedMotionEntrance {
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    track: super::UiMotionTrackIdentity,
    target: UiMotionTargetIdentity,
    source: Option<UiPublishedRect>,
    initial: Option<UiPublishedRect>,
    declaration: super::UiMotionDeclaration,
}

impl super::UiDerivedMotionServiceProposal {
    pub(in crate::runtime) fn prepared_entrance(&self) -> Option<UiPreparedMotionEntrance> {
        let request = self.staged.request;
        let successor = request.successor();
        if request.predecessor().visible()
            || !successor.visible()
            || !successor.target().is_portal_contents()
            || !request
                .declaration()
                .channels()
                .contains(super::UiMotionPropertyChannel::Opacity)
        {
            return None;
        }
        Some(UiPreparedMotionEntrance {
            frame: self.prepared_frame,
            track: self.staged.identity,
            target: successor.target(),
            source: successor.geometry(),
            initial: request.predecessor().geometry(),
            declaration: request.declaration(),
        })
    }
}

impl UiPreparedMotionEntrance {
    pub(crate) const fn track(self) -> super::UiMotionTrackIdentity {
        self.track
    }
    pub(crate) const fn frame(self) -> worth_ui_host_contract::UiMountedFrameIdentity {
        self.frame
    }
    pub(crate) const fn target(self) -> UiMotionTargetIdentity {
        self.target
    }
    pub(crate) const fn geometry(self) -> (Option<UiPublishedRect>, Option<UiPublishedRect>) {
        (self.source, self.initial)
    }
    /// Whether a reader who asked for less motion should be shown this
    /// entrance's end state outright instead of the travel toward it.
    ///
    /// The answer is the declaration's alone, which is only safe because an
    /// entrance exists only for Portal contents: `prepared_entrance` refuses
    /// every successor whose target is not, so scrolled content never reaches
    /// here. That matters because the two want opposite things. A Portal
    /// appearing is a thing the reader did not ask for, and under reduced
    /// motion it should simply be there. A scroll is travel the reader asked
    /// for by turning a wheel, and snapping it to the end would take away the
    /// continuity that tells them how far they went. If an entrance is ever
    /// prepared for anything but Portal contents, this has to ask what it is
    /// preparing before it asks the declaration.
    pub(crate) const fn snaps_for_reduced_motion(self) -> bool {
        self.declaration.settles_directly_under_reduced_motion()
    }
}
