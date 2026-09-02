use worth_ui_host_contract::UiMountedAppearanceMechanic;

use super::projection::{
    lower_unpublished_appearance_backdrop, UiMountedAppearanceBackdropInput,
    UiMountedAppearanceLoweringDenial,
};

pub(crate) fn lower_unpublished_backdrop(
    input: &UiMountedAppearanceBackdropInput,
) -> Result<UiMountedAppearanceMechanic, UiMountedAppearanceLoweringDenial> {
    let mechanic = lower_unpublished_appearance_backdrop(input)?;
    debug_assert!(matches!(mechanic, UiMountedAppearanceMechanic::Backdrop(_)));
    Ok(mechanic)
}
