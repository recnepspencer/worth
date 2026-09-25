//! What a Motion sample reads of one command it moves: where the command
//! clips, what of it shows, and the opacity it rests at.

use worth_ui_host_contract::{
    UiMountedAppearanceOpacity, UiMountedCanonicalBox, UiMountedPaintCommandIdentity,
};

use super::motion_sample::UiMountedMotionSampleWorkDenial as Denial;
use super::UiMountedPresentationState;

pub(super) struct UiMotionSampleCommand {
    pub(super) clip: UiMountedCanonicalBox,
    pub(super) visible: Option<UiMountedCanonicalBox>,
}

impl UiMountedPresentationState {
    /// The command `identity` names, as a Motion sample moves it: a scroll
    /// bar at its bound target, an appearance surface at its painted
    /// geometry, any other command where it paints.
    pub(super) fn motion_sample_command(
        &self,
        identity: UiMountedPaintCommandIdentity,
    ) -> Result<UiMotionSampleCommand, Denial> {
        if let Some(chrome) = identity.scroll_chrome_identity() {
            let target = self
                .scroll_motion_groups
                .chrome
                .get(&chrome)
                .ok_or(Denial::UnknownTargetCommands)?;
            return Ok(UiMotionSampleCommand {
                clip: target.clip,
                visible: Some(target.bounds),
            });
        }
        if identity.is_appearance_surface() {
            let target = self
                .appearance_surface_sample_target(identity.mounted_instance())
                .ok_or(Denial::UnknownTargetCommands)?;
            return Ok(UiMotionSampleCommand {
                clip: target.geometry().clip(),
                visible: Some(target.geometry().bounds()),
            });
        }
        let command = self
            .command_option(identity)
            .ok_or(Denial::UnknownTargetCommands)?;
        Ok(UiMotionSampleCommand {
            clip: command.clip_bounds(),
            visible: super::command_visible_bounds(command),
        })
    }

    /// The opacity `identity` rests at, which every Motion moving it scales:
    /// a scroll bar's own, any other command's appearance opacity.
    pub(super) fn resting_opacity(
        &self,
        identity: UiMountedPaintCommandIdentity,
    ) -> Result<UiMountedAppearanceOpacity, Denial> {
        match identity.scroll_chrome_identity() {
            Some(chrome) => self
                .scroll_motion_groups
                .chrome
                .get(&chrome)
                .map(|target| target.opacity)
                .ok_or(Denial::UnknownTargetCommands),
            None => Ok(self.appearance_opacity_for_command(identity)),
        }
    }
}
