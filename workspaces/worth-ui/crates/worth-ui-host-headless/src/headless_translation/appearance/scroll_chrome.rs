use worth_ui_host_contract::UiMountedScrollChromeMechanic;

/// Chrome is a derived paint element: it answers no hit test, so a mechanic
/// that claims to is not the chrome this host paints.
pub(super) fn validate(mechanic: &UiMountedScrollChromeMechanic) -> bool {
    !mechanic.participates_in_hit_testing()
}
