use worth_ui_host_contract::UiMountedPointerAffordanceMechanic;

use super::UiHeadlessAppearanceMechanic;

pub(super) fn translate(
    mechanic: &UiMountedPointerAffordanceMechanic,
) -> UiHeadlessAppearanceMechanic {
    UiHeadlessAppearanceMechanic::Pointer(*mechanic)
}
