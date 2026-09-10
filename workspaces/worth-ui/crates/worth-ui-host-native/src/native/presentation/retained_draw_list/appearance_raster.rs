use super::{UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial as Denial};
use worth_ui_host_contract::{UiMountedPaintCommand, UiMountedPaintCommandIdentity};

impl UiNativeRetainedDrawList {
    pub(in crate::native::presentation) fn complete_appearance_operations(
        &self,
        basis: crate::native::presentation::raster::UiNativeRasterBasis,
        atlas: &crate::native::text_atlas::UiNativeTextAtlas,
    ) -> Result<Vec<crate::native::presentation::UiNativeRasterOperation>, Denial> {
        let (_, appearance) = self
            .staged_appearance
            .as_ref()
            .ok_or(Denial::CommandMismatch)?;
        let keys = appearance
            .ordered_render_keys()
            .map_err(|_| Denial::CommandMismatch)?;
        let paint = self.order.ordered().map(|identity| identity.command());
        let items = self.ordered_render_items(paint, keys.iter().copied())?;
        self.raster_render_items(&items, basis, atlas, None)
    }

    pub(in crate::native::presentation) fn appearance_operations_for_damage(
        &mut self,
        clear: crate::native::presentation::RasterRect,
        basis: crate::native::presentation::raster::UiNativeRasterBasis,
        sampled_commands: &[UiMountedPaintCommandIdentity],
        atlas: &crate::native::text_atlas::UiNativeTextAtlas,
        counters: &mut super::UiNativeRetainedMutationCounters,
    ) -> Result<Vec<crate::native::presentation::UiNativeRasterOperation>, Denial> {
        let [left, top, width, height] = clear.physical_bounds();
        let damage = crate::native::presentation::appearance::UiNativeAppearanceDamageRect {
            left: left as i64,
            top: top as i64,
            right: (left + width) as i64,
            bottom: (top + height) as i64,
        };
        let mut direct = Vec::new();
        let mut text = Vec::new();
        for identity in sampled_commands {
            match self.command(*identity).ok_or(Denial::CommandMismatch)? {
                UiMountedPaintCommand::FilledRect { mechanic, .. } => {
                    direct.push(crate::native::presentation::appearance::UiNativeAppearanceCommandIdentity::Surface(mechanic.mounted_instance()));
                    direct.push(crate::native::presentation::appearance::UiNativeAppearanceCommandIdentity::Outline(mechanic.mounted_instance()));
                }
                UiMountedPaintCommand::PortalOverlay { mechanic, .. } => direct.push(
                    crate::native::presentation::appearance::UiNativeAppearanceCommandIdentity::PortalSurface(mechanic.owner()),
                ),
                UiMountedPaintCommand::SemanticText { .. } => text.push(*identity),
            }
        }
        let keys = {
            let (_, appearance) = self
                .staged_appearance
                .as_mut()
                .ok_or(Denial::CommandMismatch)?;
            let static_keys = appearance
                .replay_for_damage(damage)
                .map_err(|_| Denial::CommandMismatch)?;
            let motion_keys = appearance
                .ordered_motion_keys(direct, text)
                .map_err(|_| Denial::CommandMismatch)?;
            static_keys
                .iter()
                .chain(motion_keys.iter())
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
        };
        let items = self.ordered_render_items(sampled_commands.iter().copied(), keys)?;
        self.record_render_order_cost(counters)?;
        self.raster_render_items(&items, basis, atlas, Some(clear))
    }
}
