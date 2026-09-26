//! Frame-owned Scroll membership and clip provenance. These inputs are derived
//! during mounted preparation; sample ticks never rediscover layout ancestry.
use crate::runtime::{
    motion::UiMotionTargetIdentity,
    scroll::{UiScrollOffset, UiScrollPresentationDeviceScale},
};
use worth_ui_host_contract::{UiMountedCanonicalBox, UiMountedInstanceIdentity};

#[derive(Clone, Copy)]
pub(crate) struct UiMountedScrollMotionClip {
    pub(crate) bounds: UiMountedCanonicalBox,
    pub(crate) owner: Option<UiMountedInstanceIdentity>,
}

#[derive(Clone)]
pub(crate) struct UiMountedScrollMotionMember {
    pub(crate) instance: UiMountedInstanceIdentity,
    /// Each clip where layout put it.
    pub(crate) clips: std::sync::Arc<[crate::mounting::UiLaidOut<UiMountedScrollMotionClip>]>,
}

#[derive(Clone)]
pub(crate) struct UiMountedScrollMotionGroupInput {
    pub(crate) target: UiMotionTargetIdentity,
    pub(crate) owner: UiMountedInstanceIdentity,
    /// The region where the frame lays it out: its owner's box, carried by
    /// the published offsets of the regions enclosing it, and its viewport.
    pub(crate) region: crate::mounting::UiLaidOut<crate::mounting::UiMountedScrollRegionBoxes>,
    /// The content box with this region and every region enclosing it at
    /// offset zero: what the group's Scroll samples are measured from, which
    /// no enclosing region's scrolling moves. A Scroll track moves content
    /// where it is laid out, so its samples and this rest share layout space
    /// wherever a Portal presents the region.
    pub(crate) rest: crate::mounting::UiLaidOut<UiMountedCanonicalBox>,
    pub(crate) offset: UiScrollOffset,
    pub(crate) scale: UiScrollPresentationDeviceScale,
    pub(crate) chrome: Option<crate::runtime::scroll::chrome::UiScrollAdmittedChrome>,
    /// Ordered by instance.
    pub(crate) members: std::sync::Arc<[UiMountedScrollMotionMember]>,
}

use super::{motion_evidence::UiCommandMotionAcceptance, UiMountedPresentationState};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};
use worth_ui_host_contract::{
    UiMountedAppearanceOpacity, UiMountedPaintCommandIdentity, UiMountedScrollChromeIdentity,
};

#[derive(Clone)]
pub(super) struct UiMountedScrollMotionCommand {
    pub(super) identity: UiMountedPaintCommandIdentity,
    pub(super) clips: Arc<[UiMountedScrollMotionClip]>,
    /// The translation a witness last displayed for this command. It shows
    /// each of the command's groups where that group stands, so samples move
    /// it from there. A command the host has only seen published has none and
    /// stands at its groups' published offsets.
    pub(super) base_translation: Option<group_offset::UiDisplayedCommandTranslation>,
}

#[derive(Clone)]
pub(super) struct UiMountedScrollMotionGroup {
    pub(super) input: UiMountedScrollMotionGroupInput,
    /// Where the bound frame shows the group, in the client viewport: what
    /// its commands clip to and its thumbs travel within.
    shown: shown_region::UiShownScrollGroup,
    pub(super) commands: Arc<[UiMountedScrollMotionCommand]>,
    pub(super) thumbs: Arc<[UiMountedScrollChromeIdentity]>,
    /// Where the group stood when it was bound, which is where every displayed
    /// base translation bound with it shows the group: at the offset of its
    /// last displayed sample, carried across rebuilds. The published offset
    /// follows only once that sample settles, and a frame published before
    /// then must not move displayed commands by the gap.
    bound_standing: group_offset::UiBoundGroupStanding,
    /// The group's sample the last admitted witness displayed, shared with
    /// the group it rebinds unless the frame placed it.
    pub(super) displayed_sample: std::rc::Rc<std::cell::Cell<Option<UiDisplayedGroupSample>>>,
}

#[derive(Clone)]
pub(super) struct UiMountedScrollChromeSampleTarget {
    pub(super) bounds: UiMountedCanonicalBox,
    pub(super) clip: UiMountedCanonicalBox,
    pub(super) opacity: UiMountedAppearanceOpacity,
    /// The Portal whose Motion carries these bars, when their region is
    /// Portal content: they enter and leave with the content they scroll.
    pub(super) portal: Option<UiMotionTargetIdentity>,
    motion: UiCommandMotionAcceptance,
}

#[derive(Clone, Default)]
pub(super) struct UiMountedScrollMotionGroups {
    /// The bind that produced these groups.
    bind: group_offset::UiScrollGroupBind,
    /// The groups the bound frame placed: each stands where the frame
    /// publishes it, whatever sample it displaced.
    placed: std::rc::Rc<std::collections::BTreeSet<UiMotionTargetIdentity>>,
    pub(super) geometry_index_reserved_bytes: usize,
    pub(super) groups: std::rc::Rc<BTreeMap<UiMotionTargetIdentity, UiMountedScrollMotionGroup>>,
    pub(super) chrome:
        std::rc::Rc<BTreeMap<UiMountedScrollChromeIdentity, UiMountedScrollChromeSampleTarget>>,
    pub(super) memberships:
        Arc<HashMap<UiMountedPaintCommandIdentity, Arc<[UiMotionTargetIdentity]>>>,
    pub(super) owners: Arc<BTreeMap<UiMountedInstanceIdentity, UiMotionTargetIdentity>>,
}

impl UiMountedScrollMotionGroups {
    pub(super) fn motion_slot(
        &self,
        identity: UiMountedScrollChromeIdentity,
    ) -> Option<&UiCommandMotionAcceptance> {
        self.chrome.get(&identity).map(|target| &target.motion)
    }

    pub(super) fn chrome_identities(
        &self,
    ) -> impl Iterator<Item = UiMountedPaintCommandIdentity> + '_ {
        self.chrome
            .keys()
            .copied()
            .map(UiMountedPaintCommandIdentity::scroll_chrome)
    }

    /// The bars Portal `target`'s Motion carries.
    pub(super) fn portal_chrome(
        &self,
        target: UiMotionTargetIdentity,
    ) -> impl Iterator<Item = UiMountedPaintCommandIdentity> + '_ {
        self.chrome
            .iter()
            .filter(move |(_, chrome)| chrome.portal == Some(target))
            .map(|(identity, _)| UiMountedPaintCommandIdentity::scroll_chrome(*identity))
    }
}

impl UiMountedPresentationState {
    /// Bind only the candidate frame's exact membership. Committing the frame
    /// commits this index; rejecting it leaves the predecessor index intact.
    pub(in crate::mounting::presentation) fn bind_scroll_motion_groups(
        &mut self,
        frame: &crate::mounting::UiPreparedMountedFrame,
        derived: &crate::mounting::UiMountedAppearanceDerivedInput,
        placed_chrome: &[crate::mounting::UiPresented<
            crate::mounting::UiMountedAppearanceScrollChromeInput,
        >],
    ) -> Result<(), ()> {
        let bind = self.scroll_motion_groups.bind.next();
        let placed = self.directly_placed_owners(frame);
        let mut placements = direct_release::UiFramePlacements::default();
        let mut chrome = BTreeMap::new();
        for input in placed_chrome {
            let (identity, bounds, clip, opacity) = input.shown().sample_target()?;
            if !derived.scroll_motion.iter().any(|group| {
                group.target.semantic_surface() == self.requirement.semantic_surface()
                    && group.owner == identity.owner_instance()
            }) {
                continue;
            }
            let motion = self
                .scroll_motion_groups
                .chrome
                .get(&identity)
                .filter(|old| old.bounds == bounds && old.clip == clip)
                .map(|old| old.motion.clone())
                .unwrap_or_default();
            let portal = frame
                .semantic_projection()
                .region_placement(identity.owner_instance())
                .portal()
                .map(|portal| {
                    UiMotionTargetIdentity::from_portal_owner(
                        portal.surface(),
                        portal.owner(),
                        portal.portal_identity(),
                    )
                });
            chrome.insert(
                identity,
                UiMountedScrollChromeSampleTarget {
                    bounds,
                    clip,
                    opacity,
                    portal,
                    motion,
                },
            );
        }
        let mut groups = BTreeMap::new();
        let mut owners = BTreeMap::new();
        let mut memberships = HashMap::<_, Vec<_>>::new();
        for input in &derived.scroll_motion {
            if input.target.semantic_surface() != self.requirement.semantic_surface() {
                continue;
            }
            let (shown, member_clips) = shown_region::show_group(frame, input)?;
            let mut commands = Vec::new();
            for (member, clips) in input.members.iter().zip(member_clips) {
                let clips: Arc<[_]> = clips.into();
                let mut identities = self
                    .command_identities_for_instance(member.instance)
                    .collect::<Vec<_>>();
                identities.extend(
                    chrome
                        .keys()
                        .filter(|identity| identity.owner_instance() == member.instance)
                        .copied()
                        .map(UiMountedPaintCommandIdentity::scroll_chrome),
                );
                if self
                    .appearance_surface_sample_target(member.instance)
                    .is_some()
                {
                    identities.push(UiMountedPaintCommandIdentity::appearance_surface(
                        member.instance,
                    ));
                }
                for identity in identities {
                    commands.push(UiMountedScrollMotionCommand {
                        identity,
                        clips: clips.clone(),
                        base_translation: self.displayed_base_translation(identity, bind),
                    });
                }
            }
            let thumbs: Arc<[_]> = chrome
                .keys()
                .filter(|identity| {
                    identity.owner_instance() == input.owner
                        && identity.part()
                            == worth_ui_host_contract::UiMountedScrollChromePart::Thumb
                })
                .copied()
                .collect::<Vec<_>>()
                .into();
            for identity in thumbs.iter() {
                let target = chrome.get(identity).ok_or(())?;
                commands.push(UiMountedScrollMotionCommand {
                    identity: UiMountedPaintCommandIdentity::scroll_chrome(*identity),
                    clips: Arc::from([UiMountedScrollMotionClip {
                        bounds: target.clip,
                        owner: Some(input.owner),
                    }]),
                    base_translation: None,
                });
            }
            let placed_here = placed.contains(&input.owner) || self.lays_out_anew(input);
            // As unchanged commands share their slots, an unplaced group
            // shares what the host displays it at: a sample displayed beside
            // a frame in flight reaches the group that frame binds.
            let displayed_sample = owners.entry(input.owner).or_insert_with(|| {
                let carried = self
                    .scroll_motion_groups
                    .groups
                    .get(&input.target)
                    .filter(|_| !placed_here)
                    .map(|group| group.displayed_sample.clone());
                (input.target, carried.unwrap_or_default())
            });
            if displayed_sample.0 == input.target {
                for command in &commands {
                    memberships
                        .entry(command.identity)
                        .or_default()
                        .push(input.target);
                }
            }
            let standing = if placed_here {
                placements.place(
                    input.target,
                    commands.iter().map(|command| command.identity),
                );
                direct_release::placed_standing(input.offset)
            } else {
                self.group_standing(input.target, input.offset)
            };
            let bound_standing = group_offset::UiBoundGroupStanding::new(standing, bind);
            groups.insert(
                input.target,
                UiMountedScrollMotionGroup {
                    input: input.clone(),
                    shown,
                    commands: commands.into(),
                    thumbs,
                    bound_standing,
                    displayed_sample: displayed_sample.1.clone(),
                },
            );
        }
        placements.clear_bases(&mut groups);
        self.scroll_motion_groups = UiMountedScrollMotionGroups {
            bind,
            placed: placements.into_groups(),
            geometry_index_reserved_bytes: derived
                .scroll_geometry_reservations
                .get(&self.requirement.semantic_surface())
                .copied()
                .unwrap_or(0),
            groups: std::rc::Rc::new(groups),
            chrome: std::rc::Rc::new(chrome),
            memberships: Arc::new(
                memberships
                    .into_iter()
                    .map(|(id, targets)| (id, targets.into()))
                    .collect(),
            ),
            owners: Arc::new(
                owners
                    .into_iter()
                    .map(|(owner, (target, _))| (owner, target))
                    .collect(),
            ),
        };
        self.show_unbased_commands_where_groups_stand()
    }
}

#[path = "scroll_motion_groups/acceptance.rs"]
mod acceptance;
#[path = "scroll_motion_groups/sampling.rs"]
mod sampling;
pub(super) use acceptance::{UiDisplayedGroupSample, UiScrollGroupMotionUpdate};
#[path = "scroll_motion_groups/chrome_geometry.rs"]
mod chrome_geometry;
#[cfg(worth_ui_compile_probe)]
#[path = "scroll_motion_groups/compile_probe.rs"]
mod compile_probe;
#[cfg(test)]
#[path = "scroll_motion_groups/composition_tests.rs"]
mod composition_tests;
#[path = "scroll_motion_groups/direct_release.rs"]
mod direct_release;
#[path = "scroll_motion_groups/group_offset.rs"]
mod group_offset;
#[cfg(test)]
#[path = "scroll_motion_groups/group_offset_tests.rs"]
mod group_offset_tests;
#[path = "scroll_motion_groups/rebuild_base.rs"]
mod rebuild_base;
#[path = "scroll_motion_groups/retention.rs"]
mod retention;
#[path = "scroll_motion_groups/shown_region.rs"]
mod shown_region;
#[path = "scroll_motion_groups/standing_carry.rs"]
mod standing_carry;
