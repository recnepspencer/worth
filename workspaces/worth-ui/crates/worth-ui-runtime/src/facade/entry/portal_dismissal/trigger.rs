pub(super) fn dismissal_trigger(
    interaction: crate::facade::interaction::UiDismissInteraction,
    semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
) -> Option<crate::runtime::portal::UiPortalDismissalTrigger> {
    match interaction.cause() {
        crate::facade::interaction::UiDismissInteractionCause::Escape => {
            Some(crate::runtime::portal::UiPortalDismissalTrigger::Escape { semantic_surface })
        }
        crate::facade::interaction::UiDismissInteractionCause::OutsidePress(position) => Some(
            crate::runtime::portal::UiPortalDismissalTrigger::OutsidePress {
                semantic_surface,
                point: crate::mounting::presentation::UiPlatformPoint::from_host_position(position)
                    .ok()?,
            },
        ),
    }
}
