//! A command bound with no displayed base is shown where its groups stand.
//!
//! Each Scroll group stands where the host's retained commands show it, and
//! Motion and hit testing read the group there. A command bound with no
//! displayed base has no Scroll layer that shows it standing with them: a
//! frame replaced it, or brought it into a group, while a witness displayed
//! the group at a sample, so the host would draw it where the frame lays it
//! out; or a direct placement released it, so the host would keep showing it
//! through a sample its placed region no longer stands at. So each such
//! command is shown through the Scroll layer its groups' standings imply, in
//! a live slot of its own, and the host is told.
//!
//! A layer that moves nothing shows the command where the frame lays it out,
//! clipped where the frame clips it, so a command the host shows through no
//! Scroll layer gets none.

use std::collections::HashSet;

use worth_ui_host_contract::UiMountedPaintCommandIdentity;

use super::super::command_motion_layers::{UiCommandMotionLayer, UiCommandMotionLayers};
use super::super::command_motion_slot::UiDisplayedCommandMotion;
use super::super::motion_evidence::UiCommandMotionAcceptance;
use super::sampling::ActiveSamples;
use super::{UiMountedPresentationState, UiMountedScrollMotionCommand};
use crate::mounting::presentation::motion_sampling::UiPresentationMotionSampleReceipt;

impl UiMountedPresentationState {
    /// Show each command the bound groups carry with no displayed base
    /// through the Scroll layer its groups' standings imply.
    pub(super) fn show_unbased_commands_where_groups_stand(&mut self) -> Result<(), ()> {
        let mut seen = HashSet::new();
        let unbased = self
            .scroll_motion_groups
            .groups
            .values()
            .flat_map(|group| group.commands.iter())
            .filter(|command| command.base_translation.is_none() && seen.insert(command.identity))
            .cloned()
            .collect::<Vec<_>>();
        for command in &unbased {
            let identity = command.identity;
            let shown = self
                .motion_slot(identity)
                .and_then(UiCommandMotionAcceptance::displayed);
            let held = shown.map_or_else(UiCommandMotionLayers::default, |shown| shown.layers);
            let standing = self
                .standing_scroll_layer(command)?
                .filter(|(_, layer)| held.scroll_layer().is_some() || !layer.moves_nothing());
            let scroll = standing.map(|(_, layer)| layer);
            if held.scroll_layer() == scroll {
                continue;
            }
            // The outermost layer names the sample the command shows, as a
            // tick names it.
            let sample = match (shown, standing) {
                (Some(shown), _) if held.portal_only().is_some() => shown.sample,
                (_, Some((sample, _))) => sample,
                (Some(shown), None) => shown.sample,
                (None, None) => continue,
            };
            let layers = held.with_scroll(scroll);
            let change = layers
                .change(identity, self.appearance_opacity_for_command(identity))
                .map_err(|_| ())?;
            self.own_motion_slot(identity)
                .ok_or(())?
                .display(UiDisplayedCommandMotion {
                    sample,
                    change,
                    layers,
                });
            if !self.carried_motion.contains(&identity) {
                self.carried_motion.push(identity);
            }
        }
        Ok(())
    }

    /// The Scroll layer that shows `command` where each of its groups stands,
    /// and the latest sample a witness displayed one of them at; none when no
    /// witness displayed any of them.
    fn standing_scroll_layer(
        &self,
        command: &UiMountedScrollMotionCommand,
    ) -> Result<Option<(UiPresentationMotionSampleReceipt, UiCommandMotionLayer)>, ()> {
        let Some(sample) = self
            .scroll_motion_groups
            .memberships
            .get(&command.identity)
            .into_iter()
            .flat_map(|targets| targets.iter())
            .filter_map(|target| {
                self.scroll_motion_groups
                    .groups
                    .get(target)?
                    .standing()
                    .displayed_by()
            })
            .max_by_key(|sample| sample.tick())
        else {
            return Ok(None);
        };
        // No group moves: the sample holds each where it stands.
        let change = self
            .scroll_command_change(
                command,
                &ActiveSamples::new(),
                &sample,
                sample.presentation_basis(),
            )
            .map_err(|_| ())?;
        let layer = change
            .transform()
            .zip(change.clip())
            .map(|(transform, clip)| {
                UiCommandMotionLayer::scrolled(transform, clip, sample.opacity_units())
            })
            .ok_or(())?;
        Ok(Some((sample, layer)))
    }

    /// Give `command` a live slot its predecessors do not share.
    fn own_motion_slot(
        &mut self,
        command: UiMountedPaintCommandIdentity,
    ) -> Option<UiCommandMotionAcceptance> {
        if let Some(identity) = command.scroll_chrome_identity() {
            let motion = UiCommandMotionAcceptance::default();
            std::rc::Rc::make_mut(&mut self.scroll_motion_groups.chrome)
                .get_mut(&identity)?
                .motion = motion.clone();
            return Some(motion);
        }
        if command.is_appearance_surface() {
            return self.own_appearance_surface_slot(command.mounted_instance());
        }
        let instance = command.mounted_instance();
        let mut bundle = self.commands_by_instance.get(&instance)?.clone();
        let motion = bundle.own_motion_slot(command)?;
        self.commands_by_instance.insert(instance, bundle);
        Some(motion)
    }
}

#[cfg(test)]
#[path = "standing_carry_tests.rs"]
mod tests;
