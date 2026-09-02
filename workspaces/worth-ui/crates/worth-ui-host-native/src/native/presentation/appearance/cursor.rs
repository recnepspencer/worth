use winit::window::CursorIcon;

use worth_ui_host_contract::{UiMountedPointerAffordanceMechanic, UiPointerAffordanceFamily};

pub(crate) fn cursor_icon(mechanic: UiMountedPointerAffordanceMechanic) -> CursorIcon {
    match mechanic.family() {
        UiPointerAffordanceFamily::Default => CursorIcon::Default,
        UiPointerAffordanceFamily::Activation => CursorIcon::Pointer,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_mapping_is_sealed_family_to_os_icon() {
        let pointer = worth_ui_host_contract::UiHostPointerIdentity::new(1);
        let surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let target = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
        assert_eq!(
            cursor_icon(
                UiMountedPointerAffordanceMechanic::complete_from_runtime_mounting(
                    pointer,
                    surface,
                    target,
                    UiPointerAffordanceFamily::Default,
                )
            ),
            CursorIcon::Default
        );
        assert_eq!(
            cursor_icon(
                UiMountedPointerAffordanceMechanic::complete_from_runtime_mounting(
                    pointer,
                    surface,
                    target,
                    UiPointerAffordanceFamily::Activation,
                )
            ),
            CursorIcon::Pointer
        );
    }
}
