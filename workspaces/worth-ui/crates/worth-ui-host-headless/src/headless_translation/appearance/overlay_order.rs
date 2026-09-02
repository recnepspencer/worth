use std::collections::BTreeSet;

use worth_ui_host_contract::{
    UiMountedOverlayOrderMechanic, UiOverlayParticipantIdentity, UiSemanticSurfaceIdentity,
};

pub(super) fn validate(
    surface: UiSemanticSurfaceIdentity,
    order: &UiMountedOverlayOrderMechanic,
) -> bool {
    order.semantic_surface() == surface
        && order.portal_revision() != 0
        && order.backdrop_revision() != 0
        && order
            .bottom_to_top()
            .iter()
            .all(|participant| match participant {
                UiOverlayParticipantIdentity::Portal(instance) => instance.diagnostic_value() != 0,
                UiOverlayParticipantIdentity::Backdrop(identity) => {
                    !identity.declaration_projection().is_empty()
                        && match identity.scope() {
                            worth_ui_host_contract::UiMountedBackdropScope::SurfaceSingleton(
                                surface,
                            ) => surface.diagnostic_value() != 0,
                            worth_ui_host_contract::UiMountedBackdropScope::PerPortalInstance(
                                instance,
                            ) => instance.diagnostic_value() != 0,
                        }
                }
            })
        && order
            .bottom_to_top()
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            .len()
            == order.bottom_to_top().len()
}
