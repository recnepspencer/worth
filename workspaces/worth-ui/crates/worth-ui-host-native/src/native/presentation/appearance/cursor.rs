use winit::window::CursorIcon;

use worth_ui_host_contract::{UiMountedPointerAffordanceMechanic, UiPointerAffordanceFamily};

pub(crate) fn accept_cursor(
    state: &mut crate::native::UiNativeHostState,
    attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    cursor: Option<CursorIcon>,
) {
    let Some(cursor) = cursor else {
        return;
    };
    // A late superseded completion may supply an inherited pointer change,
    // but must not undo a newer accepted pointer change on this window.
    if state
        .accepted_cursor
        .is_some_and(|(accepted, _)| accepted.diagnostic_value() >= attempt.diagnostic_value())
    {
        return;
    }
    state.accepted_cursor = Some((attempt, cursor));
    if let Some(window) = &state.window {
        window.set_cursor(cursor);
    }
}

pub(crate) fn completed_cursor(
    view: &worth_ui_host_contract::UiMountedFrameConsumptionView<'_>,
) -> Option<CursorIcon> {
    let work = view.appearance_work()?;
    work.fragments().iter().find_map(|fragment| {
        completed_fragment_cursor(fragment.identity(), fragment.work().successor().mechanics())
    })
}

pub(super) fn completed_fragment_cursor(
    identity: worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity,
    mechanics: &[worth_ui_host_contract::UiMountedAppearanceMechanic],
) -> Option<CursorIcon> {
    if !matches!(
        identity,
        worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity::SurfacePointer { .. }
    ) {
        return None;
    }
    Some(
        mechanics
            .iter()
            .find_map(|mechanic| match mechanic {
                worth_ui_host_contract::UiMountedAppearanceMechanic::Pointer(pointer) => {
                    Some(cursor_icon(*pointer))
                }
                _ => None,
            })
            .unwrap_or(CursorIcon::Default),
    )
}

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
