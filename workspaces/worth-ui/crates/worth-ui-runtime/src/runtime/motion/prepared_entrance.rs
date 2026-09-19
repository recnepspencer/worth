//! The first physical sample of an exact, derived Portal entrance proposal.
use super::{UiMotionSemanticGeometry, UiMotionTargetIdentity};

#[derive(Clone, Copy)]
pub(crate) struct UiPreparedMotionEntrance {
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    track: super::UiMotionTrackIdentity,
    target: UiMotionTargetIdentity,
    source: Option<UiMotionSemanticGeometry>,
    initial: Option<UiMotionSemanticGeometry>,
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
    pub(crate) const fn geometry(
        self,
    ) -> (
        Option<UiMotionSemanticGeometry>,
        Option<UiMotionSemanticGeometry>,
    ) {
        (self.source, self.initial)
    }
    pub(crate) fn snaps_for_reduced_motion(self) -> bool {
        self.declaration.decorative()
            && self.declaration.reduced_motion()
                == super::UiMotionReducedMotionPolicy::SystemRespecting
    }
}
