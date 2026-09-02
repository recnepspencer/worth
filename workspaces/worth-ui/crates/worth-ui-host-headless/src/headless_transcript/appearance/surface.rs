use worth_ui_host_contract::{
    UiMountedPortalSurfaceAppearanceMechanic, UiMountedSurfaceAppearanceMechanic,
};

use super::UiHeadlessAppearanceMechanic;

pub(super) fn translate(
    mechanic: &UiMountedSurfaceAppearanceMechanic,
) -> UiHeadlessAppearanceMechanic {
    UiHeadlessAppearanceMechanic::Surface(mechanic.clone())
}

pub(super) fn translate_portal(
    mechanic: &UiMountedPortalSurfaceAppearanceMechanic,
) -> UiHeadlessAppearanceMechanic {
    UiHeadlessAppearanceMechanic::PortalSurface(mechanic.clone())
}
