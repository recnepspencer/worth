use worth_ui_host_contract::UiMountedPointerAffordanceMechanic;

pub(super) fn validate(mechanic: &UiMountedPointerAffordanceMechanic) -> bool {
    mechanic.pointer().value() != 0
        && mechanic.surface().diagnostic_value() != 0
        && mechanic.target().diagnostic_value() != 0
}
