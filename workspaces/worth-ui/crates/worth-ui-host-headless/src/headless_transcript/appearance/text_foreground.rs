use worth_ui_host_contract::UiMountedTextForegroundAppearanceMechanic;

use super::UiHeadlessAppearanceMechanic;

pub(super) fn translate(
    mechanic: &UiMountedTextForegroundAppearanceMechanic,
) -> UiHeadlessAppearanceMechanic {
    UiHeadlessAppearanceMechanic::TextForeground(mechanic.clone())
}
