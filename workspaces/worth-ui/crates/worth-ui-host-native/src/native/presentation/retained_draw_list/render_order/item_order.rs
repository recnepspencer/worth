//! Shared render-key derivation for complete ordering and top-paint observation.
use super::*;

impl UiNativeRetainedDrawList {
    pub(super) fn paint_render_entry(
        &self,
        appearance: &crate::native::presentation::appearance::UiNativeAppearanceRetained,
        overlay: Option<&worth_ui_host_contract::UiMountedOverlayOrderMechanic>,
        identity: UiMountedPaintCommandIdentity,
    ) -> Result<Option<(RenderOrderKey, UiNativeRetainedRenderItem)>, Denial> {
        let command = self.command(identity).ok_or(Denial::CommandMismatch)?;
        let portal_group = match command {
            UiMountedPaintCommand::PortalOverlay { mechanic, .. } => Some(mechanic.owner()),
            UiMountedPaintCommand::SemanticText { mechanic, .. } => mechanic.portal_group(),
        };
        if portal_group.is_some_and(|portal| !super::portal_is_issued(overlay, portal)) {
            return Ok(None);
        }
        let replaced = match command {
            UiMountedPaintCommand::PortalOverlay { mechanic, .. } => appearance
                .key_for_identity(&UiNativeAppearanceCommandIdentity::PortalSurface(
                    mechanic.owner(),
                ))
                .is_some(),
            UiMountedPaintCommand::SemanticText { .. } => false,
        };
        if replaced {
            return Ok(None);
        }
        {
            let order = match command {
                UiMountedPaintCommand::PortalOverlay { mechanic, .. } => (
                    1,
                    portal_anchor_rank(self, overlay, mechanic.owner())?,
                    0,
                    0,
                    0,
                    0,
                    0,
                ),
                UiMountedPaintCommand::SemanticText { mechanic, .. } => {
                    match mechanic.portal_group() {
                        Some(portal) => (
                            1,
                            portal_anchor_rank(self, overlay, portal)?,
                            1,
                            mechanic.layer_semantic_order(),
                            2,
                            paint_rank(self, identity)?,
                            0,
                        ),
                        None => (0, paint_rank(self, identity)?, 2, 0, 0, 0, 0),
                    }
                }
            };
            Ok(Some((order, UiNativeRetainedRenderItem::Paint(identity))))
        }
    }

    pub(super) fn appearance_render_entry(
        &self,
        appearance: &crate::native::presentation::appearance::UiNativeAppearanceRetained,
        overlay: Option<&worth_ui_host_contract::UiMountedOverlayOrderMechanic>,
        appearance_rank: usize,
        key: UiNativeAppearanceCommandKey,
    ) -> Result<Option<(RenderOrderKey, UiNativeRetainedRenderItem)>, Denial> {
        let command = appearance.command(key).ok_or(Denial::CommandMismatch)?;
        let portal_group = match command {
            UiNativeAppearanceCommand::Surface(mechanic) => mechanic.portal_group(),
            UiNativeAppearanceCommand::Outline(mechanic) => mechanic.portal_group(),
            UiNativeAppearanceCommand::PortalSurface(mechanic) => Some(mechanic.portal_instance()),
            _ => None,
        };
        if portal_group.is_some_and(|portal| !super::portal_is_issued(overlay, portal)) {
            return Ok(None);
        }
        let order = match command {
            UiNativeAppearanceCommand::Surface(mechanic) => appearance_surface_order(
                self,
                overlay,
                mechanic.portal_group(),
                mechanic.surface_paint_order(),
                0,
                appearance_rank,
                key,
            )?,
            UiNativeAppearanceCommand::Outline(mechanic) => appearance_surface_order(
                self,
                overlay,
                mechanic.portal_group(),
                mechanic.surface_paint_order(),
                1,
                appearance_rank,
                key,
            )?,
            UiNativeAppearanceCommand::PortalSurface(mechanic) => (
                1,
                portal_anchor_rank(self, overlay, mechanic.portal_instance())?,
                0,
                0,
                0,
                0,
                key.value(),
            ),
            UiNativeAppearanceCommand::Backdrop(mechanic) => backdrop_order(
                overlay.ok_or(Denial::CommandMismatch)?,
                mechanic.identity(),
                appearance_rank,
                key,
            )?,
            // Chrome sits at the top of its surface's own content layer: above
            // every scrolled rectangle, below the overlay layer that carries
            // portals and their backdrops.
            UiNativeAppearanceCommand::ScrollChrome(mechanic) => (
                0,
                usize::MAX,
                0,
                0,
                mechanic.identity().part().paint_ordinal() as u8,
                appearance_rank,
                key.value(),
            ),
            UiNativeAppearanceCommand::TextForeground(_)
            | UiNativeAppearanceCommand::OverlayOrder(_)
            | UiNativeAppearanceCommand::PointerAffordance(_) => return Ok(None),
        };
        Ok(Some((order, UiNativeRetainedRenderItem::Appearance(key))))
    }
}
