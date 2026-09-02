use worth_ui_host_contract::{
    UiAppearanceVisualBounds, UiMountedPortalSurfaceAppearanceMechanic,
    UiMountedSurfaceAppearanceMechanic,
};

pub(super) fn validate(mechanic: &UiMountedSurfaceAppearanceMechanic) -> bool {
    mechanic.visual_bounds() == UiAppearanceVisualBounds::from_surface_allocation(mechanic.bounds())
}

pub(super) fn validate_portal(mechanic: &UiMountedPortalSurfaceAppearanceMechanic) -> bool {
    mechanic.portal_instance() == mechanic.surface().node_receipt().mounted_instance()
        && validate(mechanic.surface())
}
