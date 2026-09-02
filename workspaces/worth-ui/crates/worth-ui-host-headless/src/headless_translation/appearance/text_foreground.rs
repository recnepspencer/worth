use worth_ui_host_contract::UiMountedTextForegroundAppearanceMechanic;

pub(super) fn validate(mechanic: &UiMountedTextForegroundAppearanceMechanic) -> bool {
    mechanic
        .node_receipt()
        .mounted_instance()
        .diagnostic_value()
        != 0
}
