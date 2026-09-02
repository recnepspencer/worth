use worth_ui_host_contract::{
    UiMountedAppearanceMechanic, UiMountedPortalSurfaceAppearanceMechanic,
    UiMountedSurfaceAppearanceCompletionInput, UiMountedSurfaceAppearanceMechanic,
};

use super::fact::UiMountedAppearanceNodeInput;
use super::opacity_composition::compose;
use super::UiMountedAppearanceLoweringDenial;

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
            clip: input.clip,
            layer: input.layer,
            radii: input.radii,
            paint,
            opacity: compose(input.appearance_opacity, input.motion_opacity),
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
