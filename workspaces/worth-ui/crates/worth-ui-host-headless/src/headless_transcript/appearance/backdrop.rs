use worth_ui_host_contract::UiMountedBackdropMechanic;

use super::UiHeadlessAppearanceMechanic;

pub(super) fn translate(mechanic: &UiMountedBackdropMechanic) -> UiHeadlessAppearanceMechanic {
    UiHeadlessAppearanceMechanic::Backdrop(mechanic.clone())
}
