use worth_ui_host_contract::UiMountedBackdropMechanic;

pub(super) fn validate(mechanic: &UiMountedBackdropMechanic) -> bool {
    !mechanic.participates_in_hit_testing()
}
