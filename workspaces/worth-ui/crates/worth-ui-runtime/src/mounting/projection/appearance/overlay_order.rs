use worth_ui_host_contract::UiMountedOverlayOrderMechanic;

use super::fact::UiMountedAppearanceOverlayInput;
use super::UiMountedAppearanceLoweringDenial;

pub(super) fn lower(
    input: &UiMountedAppearanceOverlayInput,
) -> Result<UiMountedOverlayOrderMechanic, UiMountedAppearanceLoweringDenial> {
    if (input.portal_revision == 0 || input.backdrop_revision == 0)
        && !input.bottom_to_top.is_empty()
    {
        return Err(UiMountedAppearanceLoweringDenial::OverlayRevisionMissing);
    }
    UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
        input.semantic_surface,
        input.presentation,
        input.portal_revision,
        input.backdrop_revision,
        input.bottom_to_top.iter().cloned(),
    )
    .map_err(UiMountedAppearanceLoweringDenial::OverlayOrder)
}
