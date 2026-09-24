//! The physically accepted translation of a Scroll region's content group.

use super::UiMountedMotionSampler;
use crate::runtime::motion::{UiMotionTargetIdentity, UiMotionTargetScope};

impl UiMountedMotionSampler {
    pub(crate) fn rebase_presented_scroll_extent(
        &mut self,
        target: UiMotionTargetIdentity,
        tick: u64,
    ) -> Result<(), super::super::UiPresentationGeometrySamplingDenial> {
        assert_eq!(target.scope(), UiMotionTargetScope::ScrollContents);
        self.note_owner_change(target);
        let state = self
            .tracks
            .get_mut(&target)
            .expect("extent retarget was installed");
        state.rebase_presented_extent(tick)
    }

    /// The accepted translation of one Scroll region's scrolled content, in the
    /// sample's own coordinate space. This is the sole authority for displayed
    /// scrolled geometry: it reports only what the host has already accepted,
    /// never the semantic target the content is still travelling toward.
    ///
    /// Only a Scroll-content target answers. An ordinary component target and a
    /// Portal-content target name different moving things on the same mounted
    /// instance, so neither can stand in for this group.
    pub(crate) fn accepted_scroll_group_translation(
        &self,
        target: UiMotionTargetIdentity,
    ) -> Option<[f32; 2]> {
        if target.scope() != UiMotionTargetScope::ScrollContents {
            return None;
        }
        let state = self.tracks.get(&target)?;
        if !state.presented {
            return None;
        }
        let components = state.current?.geometry()?.components();
        Some([components[0], components[1]])
    }
}

impl UiMountedMotionSampler {
    /// Forget one Scroll content group's track outright, active or settled, so
    /// direct pointer control of that group meets no sample fighting it. The
    /// retirement is remembered until the next commit, because a tick prepared
    /// before it would otherwise reinstate the track.
    ///
    /// Only a Scroll-content target is retired this way; direct control has no
    /// claim on a component or Portal track sharing the mounted instance.
    pub(crate) fn retire_scroll_group_track(&mut self, target: UiMotionTargetIdentity) -> bool {
        if target.scope() != UiMotionTargetScope::ScrollContents {
            return false;
        }
        self.note_owner_change(target);
        self.tracks.remove(&target).is_some()
    }
}
