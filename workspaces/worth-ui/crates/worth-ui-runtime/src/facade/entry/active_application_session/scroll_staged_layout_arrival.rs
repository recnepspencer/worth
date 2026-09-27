//! Carrying a settle that has arrived into the layout staged beside it.
//!
//! A layout is lowered from the offset Scroll accepted when it was staged,
//! and the frame that shows it places each group it lays out anew where it
//! lowered it. A settle whose Motion has arrived while that layout waits
//! shows its content at its target, past the offset the layout was lowered
//! from, and no track is left to carry it on once the layout lands. So both
//! the staged Scroll record and the staged geometry follow the displayed
//! offset, as far as the staged bounds reach: the frame that shows the
//! layout puts the content where the reader last saw it, and the settle ends
//! there.
//!
//! A settle still traveling is left to its track, which the publication
//! rebases onto the new extent, and an owner direct input has staged is
//! left to that input, which the layout already stands on.

use super::scroll_accepted_sample_settlement::UiAcceptedScrollSettlement;
use crate::runtime::motion::UiMotionTargetIdentity;

impl super::super::WorthUiActiveApplicationSession {
    /// Stand each settlement whose Motion has arrived in the layout staged
    /// for its surface, where the host shows its content. A settlement mounted
    /// geometry cannot stage yet is carried by the frame that pays the owed
    /// settle.
    pub(super) fn carry_arrivals_into_staged_layout(
        &mut self,
        settlements: &[(UiMotionTargetIdentity, UiAcceptedScrollSettlement)],
    ) {
        for (target, settlement) in settlements {
            if self
                .motion
                .as_ref()
                .is_some_and(|motion| motion.committed_track(*target).is_some())
            {
                continue;
            }
            let owner = settlement.owner;
            let (scroll_owner, incarnation) = (owner.entry.owner(), owner.entry.incarnation());
            let displayed = settlement.offset.settled();
            let Some(scroll) = self.scroll.as_ref() else {
                return;
            };
            if scroll.has_pending_direct_owner(scroll_owner) {
                continue;
            }
            let Some(staged) = scroll.staged_layout_offset(scroll_owner, incarnation, displayed)
            else {
                continue;
            };
            if self
                .mounted
                .settle_staged_layout_pose(owner.surface, owner.owner_instance, staged)
                .is_err()
            {
                continue;
            }
            if let Some(scroll) = self.scroll.as_mut() {
                scroll.settle_staged_layout(scroll_owner, incarnation, displayed);
            }
        }
    }
}
