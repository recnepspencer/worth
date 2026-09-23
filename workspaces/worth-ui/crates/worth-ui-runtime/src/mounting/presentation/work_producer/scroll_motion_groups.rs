//! Frame-owned Scroll membership and clip provenance. These inputs are derived
//! during mounted preparation; sample ticks never rediscover layout ancestry.
use crate::runtime::{
    motion::UiMotionTargetIdentity,
    scroll::{UiScrollOffset, UiScrollPresentationDeviceScale},
};
use worth_ui_host_contract::{UiMountedCanonicalBox, UiMountedInstanceIdentity};

#[derive(Clone)]
pub(crate) struct UiMountedScrollMotionClip {
    pub(crate) bounds: UiMountedCanonicalBox,
    pub(crate) owner: Option<UiMountedInstanceIdentity>,
}

#[derive(Clone)]
pub(crate) struct UiMountedScrollMotionMember {
    pub(crate) instance: UiMountedInstanceIdentity,
    pub(crate) clips: std::sync::Arc<[UiMountedScrollMotionClip]>,
}

#[derive(Clone)]
pub(crate) struct UiMountedScrollMotionGroupInput {
    pub(crate) target: UiMotionTargetIdentity,
    pub(crate) owner: UiMountedInstanceIdentity,
    pub(crate) content: UiMountedCanonicalBox,
    pub(crate) viewport: UiMountedCanonicalBox,
    pub(crate) offset: UiScrollOffset,
    pub(crate) scale: UiScrollPresentationDeviceScale,
    pub(crate) chrome: Option<crate::runtime::scroll::chrome::UiScrollAdmittedChrome>,
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
    pub(super) base_translation: [f32; 2],
}

#[derive(Clone)]
pub(super) struct UiMountedScrollMotionGroup {
    pub(super) input: UiMountedScrollMotionGroupInput,
    pub(super) commands: Arc<[UiMountedScrollMotionCommand]>,
    pub(super) thumbs: Arc<[UiMountedScrollChromeIdentity]>,
    pub(super) accepted: std::rc::Rc<
        std::cell::Cell<Option<super::super::motion_sampling::UiPresentationMotionSampleReceipt>>,
    >,
}

#[derive(Clone)]
pub(super) struct UiMountedScrollChromeSampleTarget {
    pub(super) bounds: UiMountedCanonicalBox,
    pub(super) clip: UiMountedCanonicalBox,
    pub(super) opacity: UiMountedAppearanceOpacity,
    motion: UiCommandMotionAcceptance,
}

#[derive(Clone, Default)]
pub(super) struct UiMountedScrollMotionGroups {
    pub(super) geometry_index_reserved_bytes: usize,
    pub(super) groups: Arc<BTreeMap<UiMotionTargetIdentity, UiMountedScrollMotionGroup>>,
    pub(super) chrome:
        Arc<BTreeMap<UiMountedScrollChromeIdentity, UiMountedScrollChromeSampleTarget>>,
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
}

impl UiMountedPresentationState {
    /// Bind only the candidate frame's exact membership. Committing the frame
    /// commits this index; rejecting it leaves the predecessor index intact.
    pub(in crate::mounting::presentation) fn bind_scroll_motion_groups(
        &mut self,
        frame: &crate::mounting::UiPreparedMountedFrame,
        derived: &crate::mounting::UiMountedAppearanceDerivedInput,
    ) -> Result<(), ()> {
        let mut chrome = BTreeMap::new();
        for input in &derived.scroll_chrome {
            let (identity, bounds, clip, opacity) = input.sample_target()?;
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
            chrome.insert(
                identity,
                UiMountedScrollChromeSampleTarget {
                    bounds,
                    clip,
                    opacity,
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
            let project = |bounds| {
                frame
                    .presentation_delta_source()
                    .frame()
                    .scroll_sample_viewport_bounds(input.target.semantic_surface(), bounds)
                    .map_err(|_| ())
            };
            let mut input = input.clone();
            input.content = project(input.content)?;
            input.viewport = project(input.viewport)?;
            let mut commands = Vec::new();
            for member in input.members.iter() {
                let mut clips = member
                    .clips
                    .iter()
                    .map(|clip| {
                        Ok(UiMountedScrollMotionClip {
                            bounds: project(clip.bounds)?,
                            owner: clip.owner,
                        })
                    })
                    .collect::<Result<Vec<_>, ()>>()?;
                clips.push(UiMountedScrollMotionClip {
                    bounds: input.viewport,
                    owner: Some(input.owner),
                });
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
                    let base_translation = self
                        .accepted_motion_change(identity)
                        .and_then(|change| change.transform())
                        .map(|transform| {
                            [
                                transform.sampled().x() - transform.source().x(),
                                transform.sampled().y() - transform.source().y(),
                            ]
                        })
                        .unwrap_or([0.0; 2]);
                    commands.push(UiMountedScrollMotionCommand {
                        identity,
                        clips: clips.clone(),
                        base_translation,
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
                    base_translation: [0.0; 2],
                });
            }
            let accepted = owners
                .entry(input.owner)
                .or_insert_with(|| (input.target, std::rc::Rc::new(std::cell::Cell::new(None))));
            if accepted.0 == input.target {
                for command in &commands {
                    memberships
                        .entry(command.identity)
                        .or_default()
                        .push(input.target);
                }
            }
            groups.insert(
                input.target,
                UiMountedScrollMotionGroup {
                    input: input.clone(),
                    commands: commands.into(),
                    thumbs,
                    accepted: accepted.1.clone(),
                },
            );
        }
        self.scroll_motion_groups = UiMountedScrollMotionGroups {
            geometry_index_reserved_bytes: derived
                .scroll_geometry_reservations
                .get(&self.requirement.semantic_surface())
                .copied()
                .unwrap_or(0),
            groups: Arc::new(groups),
            chrome: Arc::new(chrome),
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
        Ok(())
    }
}

#[path = "scroll_motion_groups/acceptance.rs"]
mod acceptance;
#[path = "scroll_motion_groups/sampling.rs"]
mod sampling;
pub(super) use acceptance::UiScrollGroupMotionUpdate;
#[path = "scroll_motion_groups/chrome_geometry.rs"]
mod chrome_geometry;
#[cfg(test)]
#[path = "scroll_motion_groups/composition_tests.rs"]
mod composition_tests;
#[path = "scroll_motion_groups/retention.rs"]
mod retention;
