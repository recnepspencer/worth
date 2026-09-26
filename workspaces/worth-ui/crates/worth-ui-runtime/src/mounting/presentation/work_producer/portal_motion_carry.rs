//! What a frame replaces inside a Portal keeps showing through the Portal's
//! Motion.
//!
//! Every Portal tick moves the Portal's whole group through one layer, so each
//! command the Portal presents holds the same Portal layer. A command a frame
//! replaces with another meaning, or brings into the group, starts with no
//! Motion accepted, and the host retires what it showed the old command
//! through. Without the Portal's layer the host would show it at rest while
//! the rest of the Portal, and interaction reading the Portal's sample, stand
//! where the Portal's Motion puts them.

use worth_ui_host_contract::{
    UiMountedInstanceIdentity, UiMountedPaintCommandIdentity, UiMountedPresentationSampleChange,
};

use super::super::motion_sampling::UiPresentationMotionSampleReceipt;
use super::command_motion_layers::UiPortalMotionLayer;
use super::command_motion_slot::UiDisplayedCommandMotion;
use super::UiMountedPresentationState;
use crate::runtime::motion::UiMotionTargetIdentity;

#[cfg(test)]
#[path = "portal_motion_carry_tests.rs"]
mod tests;

/// The Portal layer the host shows a Portal's group through, and the sample
/// that placed it.
#[derive(Clone, Copy)]
pub(super) struct UiShownPortalMotion {
    sample: UiPresentationMotionSampleReceipt,
    layer: UiPortalMotionLayer,
}

impl UiMountedPresentationState {
    /// How the host shows what the Portal `target` presents, read from the
    /// command of its group the latest tick placed.
    pub(super) fn shown_portal_motion(
        &self,
        target: UiMotionTargetIdentity,
    ) -> Option<UiShownPortalMotion> {
        self.portal_motion_group(target)?
            .commands()
            .filter_map(|command| {
                let displayed = self.motion_slot(command)?.displayed()?;
                Some(UiShownPortalMotion {
                    sample: displayed.sample,
                    layer: displayed.layers.portal_only()?,
                })
            })
            .max_by_key(|shown| shown.sample.tick())
    }

    /// Each Portal presenting a command of `instance`, with how the host shows
    /// it now.
    pub(super) fn shown_portal_motions(
        &self,
        instance: UiMountedInstanceIdentity,
    ) -> Vec<(UiMotionTargetIdentity, UiShownPortalMotion)> {
        self.portal_motion_groups
            .targets_of(instance)
            .filter_map(|target| Some((target, self.shown_portal_motion(target)?)))
            .collect()
    }

    /// Show each command of `instance` that a Portal presents, and that this
    /// successor gave a live slot of its own, through the Portal layer
    /// `predecessor` shows that Portal through. A slot the successor shares
    /// with `predecessor` is what the host already shows, never the frame's.
    pub(super) fn carry_portal_motion(
        &mut self,
        predecessor: &Self,
        instance: UiMountedInstanceIdentity,
    ) {
        let carried = self
            .portal_motion_groups
            .targets_of(instance)
            .filter_map(|target| Some((target, predecessor.shown_portal_motion(target)?)))
            .flat_map(|(target, shown)| {
                self.portal_commands_of(target, instance)
                    .into_iter()
                    .map(move |command| (command, shown))
            })
            .filter(|(command, _)| {
                self.motion_slot(*command).is_some_and(|slot| {
                    !predecessor
                        .motion_slot(*command)
                        .is_some_and(|shared| shared.is(slot))
                })
            })
            .collect::<Vec<_>>();
        for (command, shown) in carried {
            self.show_through_portal(command, shown);
        }
    }

    /// Show `instance`'s appearance surface, whose geometry this frame
    /// rebound to a new live slot, through the Portal presenting it: as
    /// `before` showed that Portal before the rebind, or else as its group
    /// shows it now.
    pub(super) fn carry_surface_portal_motion(
        &mut self,
        instance: UiMountedInstanceIdentity,
        before: &[(UiMotionTargetIdentity, UiShownPortalMotion)],
    ) {
        let surface = UiMountedPaintCommandIdentity::appearance_surface(instance);
        let shown = self
            .portal_motion_groups
            .targets_of(instance)
            .filter(|target| {
                self.portal_commands_of(*target, instance)
                    .contains(&surface)
            })
            .find_map(|target| {
                before
                    .iter()
                    .find(|(shown, _)| *shown == target)
                    .map(|(_, shown)| *shown)
                    .or_else(|| self.shown_portal_motion(target))
            });
        if let Some(shown) = shown {
            self.show_through_portal(surface, shown);
        }
    }

    fn portal_commands_of(
        &self,
        target: UiMotionTargetIdentity,
        instance: UiMountedInstanceIdentity,
    ) -> Vec<UiMountedPaintCommandIdentity> {
        self.portal_motion_group(target)
            .into_iter()
            .flat_map(|group| group.commands().collect::<Vec<_>>())
            .filter(|command| command.mounted_instance() == instance)
            .collect()
    }

    /// Show `command`, which no Motion has been accepted for, through `shown`.
    fn show_through_portal(
        &mut self,
        command: UiMountedPaintCommandIdentity,
        shown: UiShownPortalMotion,
    ) {
        let Some(slot) = self
            .motion_slot(command)
            .filter(|slot| slot.displayed().is_none())
            .cloned()
        else {
            return;
        };
        let change = shown
            .layer
            .change(command, self.appearance_opacity_for_command(command));
        slot.display(UiDisplayedCommandMotion {
            sample: shown.sample,
            change,
            layers: shown.layer.layers(),
        });
        self.carried_motion.push(command);
    }

    /// Add the change showing each command this frame carried a Portal's
    /// Motion into, unless `overrides` already shows it. The host retires what
    /// it showed a command through when a frame replaces it.
    pub(in crate::mounting::presentation) fn add_carried_motion_overrides(
        &self,
        overrides: &mut Vec<UiMountedPresentationSampleChange>,
    ) {
        for command in &self.carried_motion {
            if overrides.iter().any(|change| change.command() == *command) {
                continue;
            }
            overrides.extend(self.command_sample_change(*command));
        }
    }
}
