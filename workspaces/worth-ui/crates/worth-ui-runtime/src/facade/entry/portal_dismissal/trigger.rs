pub(super) fn dismissal_trigger(
    interaction: crate::facade::interaction::UiDismissInteraction,
    semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
) -> Option<crate::runtime::portal::UiPortalDismissalTrigger> {
    match interaction.cause() {
        crate::facade::interaction::UiDismissInteractionCause::Escape => {
            Some(crate::runtime::portal::UiPortalDismissalTrigger::Escape { semantic_surface })
        }
        crate::facade::interaction::UiDismissInteractionCause::OutsidePress(position) => {
            let basis = position.basis();
            if basis.coordinate_space()
                != worth_ui_host_contract::UiHostSurfaceCoordinateSpace::Viewport
                || basis.coordinate_unit()
                    != worth_ui_host_contract::UiHostSurfaceCoordinateUnit::LogicalPoint
            {
                return None;
            }
            let scale = worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
            let point = [
                (position.x_subpixels() as f64 / scale) as f32,
                (position.y_subpixels() as f64 / scale) as f32,
            ];
            Some(
                crate::runtime::portal::UiPortalDismissalTrigger::OutsidePress {
                    semantic_surface,
                    viewport_point_bits: point.map(f32::to_bits),
                },
            )
        }
    }
}
