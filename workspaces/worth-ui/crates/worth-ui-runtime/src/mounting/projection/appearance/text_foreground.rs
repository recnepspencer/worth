use worth_ui_host_contract::{
    UiMountedAppearanceMechanic, UiMountedTextForegroundAppearanceCompletionInput,
    UiMountedTextForegroundAppearanceMechanic,
};

use super::fact::UiMountedAppearanceNodeInput;
use super::opacity_composition::compose;
use super::UiMountedAppearanceLoweringDenial;

pub(super) fn lower(
    input: &UiMountedAppearanceNodeInput,
) -> Result<Vec<UiMountedAppearanceMechanic>, UiMountedAppearanceLoweringDenial> {
    input
        .text_foregrounds
        .iter()
        .map(|foreground| {
            UiMountedTextForegroundAppearanceMechanic::complete_from_runtime_mounting(
                UiMountedTextForegroundAppearanceCompletionInput {
                    issuer: input.issuer,
                    node_receipt: input.node_receipt,
                    paint_span: foreground.span,
                    foreground: foreground.foreground,
                    opacity: compose(input.appearance_opacity, input.motion_opacity),
                    projection: input.projection,
                },
            )
            .map(UiMountedAppearanceMechanic::TextForeground)
            .map_err(UiMountedAppearanceLoweringDenial::TextForeground)
        })
        .collect()
}
