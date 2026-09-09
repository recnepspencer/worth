use worth_ui_host_contract::{
    UiMountedAppearanceMechanic, UiMountedPortalSurfaceAppearanceMechanic,
    UiMountedSurfaceAppearanceCompletionInput, UiMountedSurfaceAppearanceMechanic,
};

use super::fact::UiMountedAppearanceNodeInput;
use super::UiMountedAppearanceLoweringDenial;
use crate::mounting::presentation::compose_opacity;

pub(super) fn lower(
    input: &UiMountedAppearanceNodeInput,
) -> Result<Option<UiMountedAppearanceMechanic>, UiMountedAppearanceLoweringDenial> {
    let Some(paint) = input.surface_paint.clone() else {
        return Ok(None);
    };
    let mechanic = UiMountedSurfaceAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedSurfaceAppearanceCompletionInput {
            issuer: input.issuer,
            node_receipt: input.node_receipt,
            bounds: input.bounds,
            clip: input.clip.for_visual_bounds(
                worth_ui_host_contract::UiAppearanceVisualBounds::from_surface_allocation(
                    input.bounds,
                ),
            )?,
            surface_paint_order: input
                .surface_paint_order
                .ok_or(UiMountedAppearanceLoweringDenial::SurfacePaintOrderUnavailable)?,
            radii: input.radii,
            border_edges: input.surface_border_edges,
            border_omissions: input.surface_border_omissions.clone(),
            paint,
            opacity: compose_opacity(
                input.appearance_opacity,
                input.motion_opacity.unwrap_or(u16::MAX),
            ),
            projection: input.projection,
        },
    )
    .map_err(UiMountedAppearanceLoweringDenial::Surface)?;
    if let Some(portal_instance) = input.portal_instance {
        return UiMountedPortalSurfaceAppearanceMechanic::complete_from_runtime_mounting(
            portal_instance,
            mechanic,
        )
        .map(UiMountedAppearanceMechanic::PortalSurface)
        .map(Some)
        .map_err(UiMountedAppearanceLoweringDenial::PortalSurface);
    }
    Ok(Some(UiMountedAppearanceMechanic::Surface(mechanic)))
}
