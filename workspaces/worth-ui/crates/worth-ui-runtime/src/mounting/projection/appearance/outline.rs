use worth_ui_host_contract::{
    UiMountedAppearanceMechanic, UiMountedOutlineAppearanceCompletionInput,
    UiMountedOutlineAppearanceMechanic,
};

use super::fact::UiMountedAppearanceNodeInput;
use super::UiMountedAppearanceLoweringDenial;
use crate::mounting::presentation::compose_opacity;

pub(super) fn lower(
    input: &UiMountedAppearanceNodeInput,
) -> Result<Option<UiMountedAppearanceMechanic>, UiMountedAppearanceLoweringDenial> {
    let Some(outline) = input.outline else {
        return Ok(None);
    };
    if outline.geometry.allocation() != input.bounds {
        return Err(UiMountedAppearanceLoweringDenial::OutlineAllocationMismatch);
    }
    let mechanic = UiMountedOutlineAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedOutlineAppearanceCompletionInput {
            issuer: input.issuer,
            node_receipt: input.node_receipt,
            surface_paint_order: input
                .surface_paint_order
                .ok_or(UiMountedAppearanceLoweringDenial::SurfacePaintOrderUnavailable)?,
            clip: input
                .clip
                .for_visual_bounds(outline.geometry.visual_bounds())?,
            geometry: outline.geometry,
            color: outline.color,
            opacity: compose_opacity(
                input.appearance_opacity,
                input.motion_opacity.unwrap_or(u16::MAX),
            ),
            projection: input.projection,
        },
    )
    .map_err(UiMountedAppearanceLoweringDenial::Outline)?;
    Ok(Some(UiMountedAppearanceMechanic::Outline(mechanic)))
}
