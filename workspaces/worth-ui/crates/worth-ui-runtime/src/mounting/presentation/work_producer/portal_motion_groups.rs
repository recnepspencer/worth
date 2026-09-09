use std::sync::Arc;

use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedInstanceIdentity, UiMountedPaintCommand,
    UiMountedPaintCommandIdentity, UiMountedPortalPresentationAffinity, UiMountedProjectionView,
    UiSemanticSurfaceIdentity,
};

use crate::runtime::{motion::UiMotionTargetIdentity, persistent_index::UiPersistentOrdMap};

use super::command_bundle::UiMountedPresentationCommandBundle;

#[derive(Clone, Default)]
pub(super) struct UiMountedPortalMotionGroups {
    targets_by_instance:
        UiPersistentOrdMap<UiMountedInstanceIdentity, Arc<[UiMotionTargetIdentity]>>,
    groups: UiPersistentOrdMap<UiMotionTargetIdentity, UiMountedPortalMotionGroup>,
}

#[derive(Clone, Default)]
struct UiMountedPortalMotionGroup {
    contributions: UiPersistentOrdMap<UiMountedInstanceIdentity, UiMountedPortalMotionContribution>,
}

#[derive(Clone)]
struct UiMountedPortalMotionContribution {
    commands: Arc<[UiMountedPaintCommandIdentity]>,
    viewport_clip: Option<UiMountedCanonicalBox>,
}

pub(in crate::mounting::presentation) struct UiMountedPortalMotionGroupView<'a> {
    group: &'a UiMountedPortalMotionGroup,
}

impl UiMountedPortalMotionGroups {
    pub(super) fn from_projection(
        projection: &UiMountedProjectionView,
        commands_by_instance: &UiPersistentOrdMap<
            UiMountedInstanceIdentity,
            UiMountedPresentationCommandBundle,
        >,
    ) -> Self {
        let affinities = projection
            .nodes()
            .iter()
            .filter_map(|node| {
                node.portal_presentation()
                    .map(|affinity| (node.mounted_instance(), affinity))
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut groups = Self::default();
        for (instance, commands) in commands_by_instance.iter() {
            groups.replace_instance(
                *instance,
                Some(commands),
                affinities.get(instance).copied(),
                projection.surface(),
            );
        }
        groups
    }

    pub(super) fn replace_instance(
        &mut self,
        instance: UiMountedInstanceIdentity,
        commands: Option<&UiMountedPresentationCommandBundle>,
        affinity: Option<UiMountedPortalPresentationAffinity>,
        surface: UiSemanticSurfaceIdentity,
    ) {
        self.remove_instance(instance);
        let Some(commands) = commands else {
            return;
        };
        let mut contributions = std::collections::BTreeMap::<
            UiMotionTargetIdentity,
            (
                Vec<UiMountedPaintCommandIdentity>,
                Option<UiMountedCanonicalBox>,
            ),
        >::new();
        for command in commands.iter() {
            let classified = match command {
                UiMountedPaintCommand::PortalOverlay { mechanic, .. } => Some((
                    target(
                        mechanic.surface(),
                        mechanic.owner(),
                        mechanic.portal_identity(),
                    ),
                    Some(mechanic.clip_bounds()),
                )),
                UiMountedPaintCommand::FilledRect { .. }
                | UiMountedPaintCommand::SemanticText { .. } => affinity.map(|affinity| {
                    (
                        target(surface, affinity.owner(), affinity.portal_identity()),
                        None,
                    )
                }),
            };
            let Some((target, clip)) = classified else {
                continue;
            };
            let entry = contributions.entry(target).or_default();
            entry.0.push(command.identity());
            if clip.is_some() {
                entry.1 = clip;
            }
        }
        if contributions.is_empty() {
            return;
        }
        let targets = contributions.keys().copied().collect::<Arc<[_]>>();
        for (target, (commands, viewport_clip)) in contributions {
            let mut group = self.groups.get(&target).cloned().unwrap_or_default();
            group.contributions.insert(
                instance,
                UiMountedPortalMotionContribution {
                    commands: commands.into(),
                    viewport_clip,
                },
            );
            self.groups.insert(target, group);
        }
        self.targets_by_instance.insert(instance, targets);
    }

    pub(super) fn group(
        &self,
        target: UiMotionTargetIdentity,
    ) -> Option<UiMountedPortalMotionGroupView<'_>> {
        self.groups
            .get(&target)
            .map(|group| UiMountedPortalMotionGroupView { group })
    }

    fn remove_instance(&mut self, instance: UiMountedInstanceIdentity) {
        let Some(targets) = self.targets_by_instance.get(&instance).cloned() else {
            return;
        };
        for target in targets.iter().copied() {
            let Some(mut group) = self.groups.get(&target).cloned() else {
                continue;
            };
            group.contributions.remove(&instance);
            if group.contributions.is_empty() {
                self.groups.remove(&target);
            } else {
                self.groups.insert(target, group);
            }
        }
        self.targets_by_instance.remove(&instance);
    }
}

impl UiMountedPortalMotionGroupView<'_> {
    pub(in crate::mounting::presentation) fn commands(
        &self,
    ) -> impl Iterator<Item = UiMountedPaintCommandIdentity> + '_ {
        self.group
            .contributions
            .iter()
            .flat_map(|(_, contribution)| contribution.commands.iter().copied())
    }

    pub(super) fn viewport_clip(&self) -> Option<UiMountedCanonicalBox> {
        self.group
            .contributions
            .iter()
            .find_map(|(_, contribution)| contribution.viewport_clip)
    }
}

fn target(
    surface: UiSemanticSurfaceIdentity,
    owner: UiMountedInstanceIdentity,
    portal_identity: u64,
) -> UiMotionTargetIdentity {
    UiMotionTargetIdentity::from_family_owner(surface, owner, portal_identity)
}
