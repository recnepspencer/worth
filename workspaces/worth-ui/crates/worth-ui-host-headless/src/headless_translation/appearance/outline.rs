use worth_ui_host_contract::UiMountedOutlineAppearanceMechanic;

pub(super) fn validate(mechanic: &UiMountedOutlineAppearanceMechanic) -> bool {
    !mechanic.participates_in_hit_testing()
}
