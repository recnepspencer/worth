use worth_ui_host_contract::{
    UiMountedAppearanceMechanic, UiMountedBackdropCompletionInput, UiMountedBackdropMechanic,
};

use super::fact::UiMountedAppearanceBackdropInput;
use super::opacity_composition::compose;
use super::UiMountedAppearanceLoweringDenial;

pub(crate) fn lower(
    input: &UiMountedAppearanceBackdropInput,
) -> Result<UiMountedAppearanceMechanic, UiMountedAppearanceLoweringDenial> {
    UiMountedBackdropMechanic::complete_from_runtime_mounting(UiMountedBackdropCompletionInput {
        identity: input.identity.clone(),
        semantic_surface: input.semantic_surface,
        placement: input.placement,
        extent: input.extent,
        clip: input.clip,
        background: input.background,
        opacity: compose(input.appearance_opacity, input.motion_opacity),
        attribution: input.attribution,
    })
    .map(UiMountedAppearanceMechanic::Backdrop)
    .map_err(UiMountedAppearanceLoweringDenial::Backdrop)
}
