//! One shared oracle for the world's before/Portal/after stacking contract.
use super::*;
pub(super) fn assert_portal_backdrop_order(
    output: &UiUnpublishedAppearanceFrameProjection,
    world: &World,
) {
    let overlay = output
        .fragments()
        .iter()
        .find(|fragment| {
            fragment.identity()
                == UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(world.surfaces[0])
        })
        .unwrap();
    let order = overlay.work().successor().overlay_order().bottom_to_top();
    for portal in world.instances[..2].iter().copied() {
        let before = overlay
            .work()
            .successor()
            .mechanics()
            .iter()
            .find_map(|mechanic| match mechanic {
                UiMountedAppearanceMechanic::Backdrop(backdrop)
                    if backdrop.identity().declaration_projection()
                        == backdrop_projection(world, "overlay.scrim")
                        && backdrop.identity().scope()
                            == UiMountedBackdropScope::PerPortalInstance(portal) =>
                {
                    Some(backdrop.identity().clone())
                }
                _ => None,
            })
            .unwrap_or_else(|| {
                panic!(
                    "missing before Backdrop for {portal:?}; mechanics: {:?}",
                    overlay.work().successor().mechanics()
                )
            });
        let after = overlay
            .work()
            .successor()
            .mechanics()
            .iter()
            .find_map(|mechanic| match mechanic {
                UiMountedAppearanceMechanic::Backdrop(backdrop)
                    if backdrop.identity().declaration_projection()
                        == backdrop_projection(world, "overlay.after")
                        && backdrop.identity().scope()
                            == UiMountedBackdropScope::PerPortalInstance(portal) =>
                {
                    Some(backdrop.identity().clone())
                }
                _ => None,
            })
            .unwrap_or_else(|| {
                panic!(
                    "missing after Backdrop for {portal:?}; mechanics: {:?}",
                    overlay.work().successor().mechanics()
                )
            });
        let position = |participant: &UiOverlayParticipantIdentity| {
            order
                .iter()
                .position(|candidate| candidate == participant)
                .unwrap()
        };
        let before = position(&UiOverlayParticipantIdentity::Backdrop(before));
        let portal = position(&UiOverlayParticipantIdentity::Portal(portal));
        let after = position(&UiOverlayParticipantIdentity::Backdrop(after));
        assert_eq!((before + 1, portal + 1), (portal, after));
    }
    assert!(overlay
        .work()
        .successor()
        .mechanics()
        .iter()
        .all(|mechanic| {
            !matches!(mechanic, UiMountedAppearanceMechanic::Backdrop(backdrop)
            if backdrop.identity().scope()
                == UiMountedBackdropScope::PerPortalInstance(world.instances[2]))
        }));
}
