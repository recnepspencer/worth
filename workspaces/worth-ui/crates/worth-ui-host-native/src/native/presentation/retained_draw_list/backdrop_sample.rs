use super::{UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial as Denial};
use worth_ui_host_contract::{
    UiMountedBackdropMechanic, UiMountedPaintCommand, UiMountedPresentationOpacity,
};

impl UiNativeRetainedDrawList {
    pub(super) fn backdrop_sample_opacity(
        &self,
        backdrop: &UiMountedBackdropMechanic,
    ) -> Result<UiMountedPresentationOpacity, Denial> {
        let Some(target) = backdrop.motion_target() else {
            return Ok(backdrop.opacity());
        };
        let mut matching = self.commands.identities_for_instance(target.owner()).filter(|identity| matches!(
            self.command(*identity), Some(UiMountedPaintCommand::PortalOverlay { mechanic, .. })
            if mechanic.surface() == backdrop.semantic_surface() && mechanic.portal_identity() == target.portal_identity()
        ));
        let identity = matching.next().ok_or(Denial::CommandMismatch)?;
        if matching.next().is_some() {
            return Err(Denial::CommandMismatch);
        }
        Ok(self
            .sample_override(identity)
            .map_or(backdrop.opacity(), |sample| {
                UiMountedPresentationOpacity::from_runtime_appearance_motion(
                    worth_ui_host_contract::UiMountedAppearanceOpacity::from_units(
                        backdrop.opacity().appearance_units(),
                    ),
                    sample.opacity().motion_units(),
                )
            }))
    }

    pub(super) fn backdrop_sample_damage(
        &self,
        sample: &worth_ui_host_contract::UiMountedPresentationSample,
    ) -> Vec<worth_ui_host_contract::UiMountedLogicalDamage> {
        let Some((_, appearance)) = &self.staged_appearance else {
            return Vec::new();
        };
        sample.changes().iter().filter_map(|change| match self.command(change.command()) {
            Some(UiMountedPaintCommand::PortalOverlay { mechanic, .. }) => Some(
                worth_ui_host_contract::UiMountedPortalPresentationAffinity::from_runtime_mounting(mechanic.owner(), mechanic.portal_identity())
            ),
            _ => None,
        }).flat_map(|target| appearance.backdrops_for_motion(target)).filter_map(|backdrop| {
            let extent = backdrop.extent();
            super::appearance_regions::canonical(extent.x(), extent.y(), extent.width(), extent.height())
                .and_then(|bounds| super::appearance_regions::canonical_clip(backdrop.clip()).and_then(|clip| bounds.intersection(clip)))
                .map(worth_ui_host_contract::UiMountedLogicalDamage::from_runtime_mounting)
        }).collect()
    }
}
