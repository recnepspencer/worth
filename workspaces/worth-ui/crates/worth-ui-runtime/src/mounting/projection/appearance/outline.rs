use worth_ui_host_contract::{
    UiMountedAppearanceMechanic, UiMountedOutlineAppearanceCompletionInput,
    UiMountedOutlineAppearanceMechanic,
};

use super::fact::UiMountedAppearanceNodeInput;
use super::opacity_composition::compose;
use super::UiMountedAppearanceLoweringDenial;

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
            clip: input.clip,
            geometry: outline.geometry,
            color: outline.color,
            opacity: compose(input.appearance_opacity, input.motion_opacity),
            projection: input.projection,
        },
    )
    .map_err(UiMountedAppearanceLoweringDenial::Outline)?;
    Ok(Some(UiMountedAppearanceMechanic::Outline(mechanic)))
}
