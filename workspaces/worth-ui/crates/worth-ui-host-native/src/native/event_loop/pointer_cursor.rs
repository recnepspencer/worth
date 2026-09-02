use winit::window::CursorIcon;

use worth_ui_host_contract::UiMountedPointerAffordanceMechanic;

/// Translate only the sealed pointer-affordance family. Bounds, paint, and
/// component kind are intentionally unavailable to this mapping.
pub(crate) fn cursor_icon(mechanic: UiMountedPointerAffordanceMechanic) -> CursorIcon {
    crate::native::presentation::appearance::cursor::cursor_icon(mechanic)
}

#[cfg(test)]
mod tests {
    use super::cursor_icon;
    use winit::window::CursorIcon;
    use worth_ui_host_contract::{
        UiHostPointerIdentity, UiMountedInstanceIdentity, UiMountedPointerAffordanceMechanic,
        UiPointerAffordanceFamily, UiSemanticSurfaceIdentity,
    };

    #[test]
    fn maps_activation_affordance_at_event_loop_boundary() {
        let mechanic = UiMountedPointerAffordanceMechanic::complete_from_runtime_mounting(
            UiHostPointerIdentity::new(1),
            UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            UiMountedInstanceIdentity::mint_unbound().unwrap(),
            UiPointerAffordanceFamily::Activation,
        );

        assert_eq!(cursor_icon(mechanic), CursorIcon::Pointer);
    }
}
