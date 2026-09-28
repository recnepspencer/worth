//! Where a group the frame rebuilds starts from: the translation a witness
//! displayed for its commands, and the offset the group stands at.

use super::group_offset::{
    UiDisplayedCommandTranslation, UiGroupStanding, UiPublishedGroupOffset, UiScrollGroupBind,
};
use super::{UiMountedPresentationState, UiMountedScrollMotionGroup};
use crate::runtime::{motion::UiMotionTargetIdentity, scroll::UiScrollOffset};
use worth_ui_host_contract::UiMountedPaintCommandIdentity;

impl UiMountedPresentationState {
    /// The translation a witness displayed for `identity`'s Scroll regions,
    /// kept as the base a rebuilt group moves it from. A Portal moving the
    /// command is its own layer and no part of that base.
    pub(super) fn displayed_base_translation(
        &self,
        identity: UiMountedPaintCommandIdentity,
        bind: UiScrollGroupBind,
    ) -> Option<UiDisplayedCommandTranslation> {
        let transform = self.accepted_motion_layers(identity).scroll_transform()?;
        Some(UiDisplayedCommandTranslation::of_displayed_transform(
            transform.source(),
            transform.sampled(),
            bind,
        ))
    }

    /// Where the group `target` stands now; a group this frame introduces
    /// stands at its `published` offset.
    pub(super) fn group_standing(
        &self,
        target: UiMotionTargetIdentity,
        published: UiScrollOffset,
    ) -> UiGroupStanding {
        self.scroll_motion_groups.groups.get(&target).map_or_else(
            || UiGroupStanding::Published(UiPublishedGroupOffset::of(published)),
            UiMountedScrollMotionGroup::standing,
        )
    }
}
