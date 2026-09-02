use worth_ui_host_contract::UiMountedOutlineAppearanceMechanic;

use super::UiHeadlessAppearanceMechanic;

pub(super) fn translate(
    mechanic: &UiMountedOutlineAppearanceMechanic,
) -> UiHeadlessAppearanceMechanic {
    UiHeadlessAppearanceMechanic::Outline(mechanic.clone())
}
