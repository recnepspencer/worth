//! Where the host shows what an instance paints as itself.

use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedInstanceIdentity, UiMountedPaintCommand,
    UiMountedPaintCommandIdentity, UiMountedPresentationTransform,
};

use super::super::command_bundle::UiMountedPresentationCommandBundle;
use super::UiMountedPresentationState;
use crate::mounting::presentation::truth_geometry::carried_box;

/// One command an instance paints as itself, as the host shows it now.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiShownOwnPaint {
    pub(crate) command: UiMountedPaintCommandIdentity,
    /// The transform the host shows the command through, if one moves it.
    pub(crate) transform: Option<UiMountedPresentationTransform>,
    /// Where the Motion showing the command shows a rect drawn beside it:
    /// `None` when that rect is drawn in a space the Motion does not map.
    pub(crate) carried: Option<UiMountedCanonicalBox>,
    /// The clip the Motion showing the command shows it through.
    pub(crate) clip: Option<UiMountedCanonicalBox>,
}

impl UiMountedPresentationState {
    /// Each command `instance` paints as itself -- its text and its
    /// appearance surface, which move with it, unlike a Portal overlay or a
    /// Scroll bar it owns -- with where the accepted Motion change showing
    /// that command shows `beside`, a rect drawn in the same layout.
    pub(in crate::mounting::presentation) fn shown_own_paint(
        &self,
        instance: UiMountedInstanceIdentity,
        beside: UiMountedCanonicalBox,
    ) -> Vec<UiShownOwnPaint> {
        let text = self
            .commands_by_instance
            .get(&instance)
            .into_iter()
            .flat_map(UiMountedPresentationCommandBundle::iter)
            .filter(|command| matches!(command, UiMountedPaintCommand::SemanticText { .. }))
            .map(UiMountedPaintCommand::identity);
        let surface = self
            .appearance_surfaces
            .get(&instance)
            .map(|_| UiMountedPaintCommandIdentity::appearance_surface(instance));
        text.chain(surface)
            .map(|command| {
                let change = self.accepted_motion_change(command);
                let transform = change.and_then(|change| change.transform());
                UiShownOwnPaint {
                    command,
                    transform,
                    carried: match transform {
                        Some(transform) => carried_box(beside, transform),
                        None => Some(beside),
                    },
                    clip: change.and_then(|change| change.clip()),
                }
            })
            .collect()
    }
}
