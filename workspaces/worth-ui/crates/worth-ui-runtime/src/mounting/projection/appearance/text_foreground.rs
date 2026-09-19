use worth_ui_host_contract::{
    UiMountedAppearanceMechanic, UiMountedTextForegroundAppearanceCompletionInput,
    UiMountedTextForegroundAppearanceMechanic,
};

use super::fact::UiMountedAppearanceNodeInput;
use super::UiMountedAppearanceLoweringDenial;
use crate::mounting::presentation::compose_opacity;

pub(super) fn lower(
    input: &UiMountedAppearanceNodeInput,
) -> Result<
    Vec<(
        UiMountedAppearanceMechanic,
        std::sync::Arc<[super::UiMountedAppearanceTextGeometry]>,
    )>,
    UiMountedAppearanceLoweringDenial,
> {
    input
        .text_foregrounds
        .iter()
        .map(|foreground| {
            UiMountedTextForegroundAppearanceMechanic::complete_from_runtime_mounting(
                UiMountedTextForegroundAppearanceCompletionInput {
                    issuer: input.issuer,
                    node_receipt: input.node_receipt,
                    command: foreground.command,
                    paint_span: foreground.span,
                    foreground: foreground.foreground,
                    opacity: compose_opacity(
                        input.appearance_opacity,
                        foreground.motion_opacity.unwrap_or(u16::MAX),
                    ),
                    projection: input.projection,
                },
            )
            .map(|mechanic| {
                (
                    UiMountedAppearanceMechanic::TextForeground(mechanic),
                    std::sync::Arc::clone(&foreground.geometry),
                )
            })
            .map_err(UiMountedAppearanceLoweringDenial::TextForeground)
        })
        .collect()
}
