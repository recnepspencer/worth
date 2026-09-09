use worth_ui_host_contract::{
    UiMountedAppearanceMechanic, UiMountedBackdropCompletionInput, UiMountedBackdropMechanic,
};

use super::fact::UiMountedAppearanceBackdropInput;
use super::UiMountedAppearanceLoweringDenial;
use crate::mounting::presentation::compose_opacity;

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
        opacity: compose_opacity(
            input.appearance_opacity,
            input.motion_opacity.unwrap_or(u16::MAX),
        ),
        attribution: input.attribution,
    })
    .map(UiMountedAppearanceMechanic::Backdrop)
    .map_err(UiMountedAppearanceLoweringDenial::Backdrop)
}
