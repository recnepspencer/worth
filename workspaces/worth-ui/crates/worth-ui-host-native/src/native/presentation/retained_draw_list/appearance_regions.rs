use worth_ui_host_contract::{
    UiHostRealizedGeometry, UiHostRealizedOrdering, UiHostRealizedRegion,
    UiHostRealizedRegionParticipation, UiMountedCanonicalBox, UiMountedCanonicalBoxInput,
    UiMountedCoordinateSpace, UiMountedPaintCommand, UiMountedSurfaceAppearanceMechanic,
    UiMountedSurfacePaint, UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT,
};

use super::super::appearance::{UiNativeAppearanceCommand, UiNativeAppearanceCommandIdentity};
use super::UiNativeRetainedDrawList;

impl UiNativeRetainedDrawList {
    pub(super) fn realized_regions_with_appearance(&self) -> Option<Vec<UiHostRealizedRegion>> {
        let paint_order = self.order.ordered().filter(|identity| {
            !self.command(identity.command()).is_some_and(|command| {
                let UiMountedPaintCommand::PortalOverlay { mechanic, .. } = command else {
                    return false;
                };
                self.staged_appearance
                    .as_ref()
                    .is_some_and(|(_, appearance)| {
                        appearance
                            .key_for_identity(&UiNativeAppearanceCommandIdentity::PortalSurface(
                                mechanic.owner(),
                            ))
                            .is_some()
                    })
            })
        });
        let mut realized = self.regions.realized(paint_order)?;
        let Some((_, appearance)) = &self.staged_appearance else {
            return Some(realized);
        };
        for key in appearance.ordered_render_keys().ok()?.iter().copied() {
            let surface = match appearance.command(key)? {
                UiNativeAppearanceCommand::Surface(surface) => surface,
                UiNativeAppearanceCommand::PortalSurface(portal) => portal.surface(),
                UiNativeAppearanceCommand::Outline(_)
                | UiNativeAppearanceCommand::TextForeground(_)
                | UiNativeAppearanceCommand::Backdrop(_)
                | UiNativeAppearanceCommand::OverlayOrder(_)
                | UiNativeAppearanceCommand::PointerAffordance(_) => continue,
            };
            if surface_paint_alpha(surface) == 0 {
                continue;
            }
            realized.push(surface_region(
                surface,
                self.regions.current_receipt(surface.node_receipt()),
            )?);
        }
        Some(realized)
    }
}

fn surface_region(
    surface: &UiMountedSurfaceAppearanceMechanic,
    receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
) -> Option<UiHostRealizedRegion> {
    Some(UiHostRealizedRegion::observed_by_host(
        receipt,
        UiHostRealizedGeometry::observed_by_host(
            canonical_visual(surface.visual_bounds())?,
            canonical_clip(surface.clip())?,
        ),
        UiHostRealizedOrdering::observed_by_host(
            surface.surface_paint_order(),
            UiHostRealizedRegionParticipation::Paint,
        ),
    ))
}

fn surface_paint_alpha(surface: &UiMountedSurfaceAppearanceMechanic) -> u8 {
    let color_alpha = match surface.paint() {
        UiMountedSurfacePaint::Fill(fill) => fill
            .colors()
            .map(|color| color.straight_srgba()[3])
            .into_iter()
            .max()
            .unwrap(),
        UiMountedSurfacePaint::FillAndBorder { fill, border, .. } => fill
            .colors()
            .map(|color| color.straight_srgba()[3])
            .into_iter()
            .max()
            .unwrap()
            .max(border.straight_srgba()[3]),
        UiMountedSurfacePaint::Border { color, .. } => color.straight_srgba()[3],
    };
    let product = u32::from(color_alpha) * u32::from(surface.opacity().units());
    ((product + u32::from(u16::MAX) / 2) / u32::from(u16::MAX)) as u8
}

pub(super) fn canonical_visual(
    bounds: worth_ui_host_contract::UiAppearanceVisualBounds,
) -> Option<UiMountedCanonicalBox> {
    canonical(bounds.x(), bounds.y(), bounds.width(), bounds.height())
}

pub(super) fn canonical_clip(
    clip: worth_ui_host_contract::UiAppearanceClip,
) -> Option<UiMountedCanonicalBox> {
    canonical(clip.x(), clip.y(), clip.width(), clip.height())
}

pub(super) fn canonical(x: i32, y: i32, width: u32, height: u32) -> Option<UiMountedCanonicalBox> {
    let units = UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT as f32;
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: x as f32 / units,
        y: y as f32 / units,
        width: width as f32 / units,
        height: height as f32 / units,
        coordinate_space: UiMountedCoordinateSpace::Viewport,
    })
    .ok()
}
