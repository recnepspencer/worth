//! A placement ends the Scroll sample it displaces.
//!
//! A page or a drag places a region's offset directly, and publishing that
//! placement retires the region's sampler: no later tick moves its content
//! from the sample the host last showed. A frame that lays a group's content
//! out anew places it too: publishing that layout rebases the region's
//! settle, or ends it, at the content the frame publishes. Either way the
//! region's group stands where the frame publishes it, and each command it
//! carries is bound with no displayed base, to be shown where its groups
//! stand.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::Arc;

use worth_ui_host_contract::{UiMountedInstanceIdentity, UiMountedPaintCommandIdentity};

use super::group_offset::{UiGroupStanding, UiPublishedGroupOffset};
use super::{UiMountedPresentationState, UiMountedScrollMotionGroup};
use crate::runtime::motion::UiMotionTargetIdentity;
use crate::runtime::scroll::UiScrollOffset;

/// The groups a frame places, and the commands each displaced a Scroll
/// sample from.
#[derive(Default)]
pub(super) struct UiFramePlacements {
    groups: BTreeSet<UiMotionTargetIdentity>,
    commands: HashSet<UiMountedPaintCommandIdentity>,
}

impl UiFramePlacements {
    pub(super) fn place(
        &mut self,
        target: UiMotionTargetIdentity,
        commands: impl IntoIterator<Item = UiMountedPaintCommandIdentity>,
    ) {
        self.groups.insert(target);
        self.commands.extend(commands);
    }

    /// Bound with no displayed base: each group of a released command stands
    /// where the frame publishes it.
    pub(super) fn clear_bases(
        &self,
        groups: &mut BTreeMap<UiMotionTargetIdentity, UiMountedScrollMotionGroup>,
    ) {
        if self.commands.is_empty() {
            return;
        }
        for group in groups.values_mut() {
            if !group
                .commands
                .iter()
                .any(|command| self.commands.contains(&command.identity))
            {
                continue;
            }
            let mut commands = group.commands.to_vec();
            for command in &mut commands {
                if self.commands.contains(&command.identity) {
                    command.base_translation = None;
                }
            }
            group.commands = Arc::from(commands);
        }
    }

    /// The groups the frame placed.
    pub(super) fn into_groups(self) -> std::rc::Rc<BTreeSet<UiMotionTargetIdentity>> {
        std::rc::Rc::new(self.groups)
    }
}

/// A displaced group stands where the frame publishes it.
pub(super) fn placed_standing(published: UiScrollOffset) -> UiGroupStanding {
    UiGroupStanding::Published(UiPublishedGroupOffset::of(published))
}

impl UiMountedPresentationState {
    /// Whether the frame binding `input` lays its group's content out
    /// somewhere other than the bound group's box at rest. A region
    /// enclosing the group carries its content without moving its rest.
    pub(super) fn lays_out_anew(&self, input: &super::UiMountedScrollMotionGroupInput) -> bool {
        self.scroll_motion_groups
            .groups
            .get(&input.target)
            .is_some_and(|group| group.input.rest != input.rest)
    }

    /// Whether the bound frame placed the group `target`.
    pub(in crate::mounting::presentation) fn placed_scroll_group(
        &self,
        target: UiMotionTargetIdentity,
    ) -> bool {
        self.scroll_motion_groups.placed.contains(&target)
    }

    /// Every command of a group the bound frame placed. The host retires the
    /// Scroll sample of each command a delta touches, and only those, so a
    /// frame that places a group re-issues its commands even where they are
    /// unchanged: a page to where the content already stands still displaces
    /// every sample a tick presented beside the frame while it was in flight.
    pub(in crate::mounting::presentation) fn placed_scroll_commands(
        &self,
    ) -> impl Iterator<Item = UiMountedPaintCommandIdentity> + '_ {
        self.scroll_motion_groups
            .placed
            .iter()
            .filter_map(|target| self.scroll_motion_groups.groups.get(target))
            .flat_map(|group| group.commands.iter().map(|command| command.identity))
    }

    /// The regions on this surface whose offset `frame` places directly.
    pub(super) fn directly_placed_owners(
        &self,
        frame: &crate::mounting::UiPreparedMountedFrame,
    ) -> BTreeSet<UiMountedInstanceIdentity> {
        frame
            .direct_scroll()
            .iter()
            .filter_map(|record| record.pose())
            .filter(|(surface, _, _)| *surface == self.requirement.semantic_surface())
            .map(|(_, owner, _)| owner)
            .collect()
    }
}
