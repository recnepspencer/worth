use super::super::{context, Context};
use super::support::{surface_affinity, surface_at};
use crate::*;

#[test]
fn structural_portal_order_is_content_but_empty_overlay_work_is_not() {
    let context = context();
    let portal =
        UiOverlayParticipantIdentity::Portal(UiMountedInstanceIdentity::mint_unbound().unwrap());
    assert!(fragment(&context, vec![portal], vec![]).is_ok());
    assert_eq!(
        fragment(&context, vec![], vec![]),
        Err(UiUnpublishedAppearanceFrameProjectionDenial::FragmentIdentityMismatch)
    );
}

#[test]
fn presentation_order_comes_from_surface_overlay_independent_of_fragment_order() {
    let context = context();
    let portal =
        UiOverlayParticipantIdentity::Portal(UiMountedInstanceIdentity::mint_unbound().unwrap());
    let overlay = fragment(&context, vec![portal.clone()], vec![]).unwrap();
    let node = super::support::removed_surface_fragment(&context);
    for fragments in [
        vec![node.clone(), overlay.clone()],
        vec![overlay.clone(), node.clone()],
    ] {
        let projection = UiUnpublishedAppearanceFrameProjection::from_runtime_mounting(
            context.frame,
            context.attempt,
            fragments,
        )
        .unwrap();
        let work = UiMountedAppearancePresentationWork::from_runtime_mounting(
            &projection,
            context.frame,
            context.attempt,
            context.requirement,
            [],
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            work.overlay_order_update().unwrap().bottom_to_top(),
            std::slice::from_ref(&portal)
        );
    }
    let projection = UiUnpublishedAppearanceFrameProjection::from_runtime_mounting(
        context.frame,
        context.attempt,
        [node],
    )
    .unwrap();
    let work = UiMountedAppearancePresentationWork::from_runtime_mounting(
        &projection,
        context.frame,
        context.attempt,
        context.requirement,
        [],
    )
    .unwrap()
    .unwrap();
    assert!(
        work.overlay_order_update().is_none(),
        "node-only work cannot clear a retained Portal stack"
    );

    let successor = UiMountedAppearanceFrame::from_runtime_mounting(
        context.frame,
        context.surface,
        [],
        UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
            context.surface,
            context.attempt,
            2,
            2,
            [],
        )
        .unwrap(),
    )
    .unwrap();
    let removal = UiMountedAppearanceWork::from_runtime_mounting(
        UiMountedAppearanceWorkPosture::Delta,
        Some(context.predecessor),
        Some(UiMountedAppearancePredecessorManifest::from_runtime_mounting([], [portal]).unwrap()),
        successor,
        [],
        [],
        true,
    )
    .unwrap();
    let removal = UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(context.surface),
        removal,
        [],
        context.requirement,
        surface_affinity(&context),
    )
    .unwrap();
    let projection = UiUnpublishedAppearanceFrameProjection::from_runtime_mounting(
        context.frame,
        context.attempt,
        [removal],
    )
    .unwrap();
    let work = UiMountedAppearancePresentationWork::from_runtime_mounting(
        &projection,
        context.frame,
        context.attempt,
        context.requirement,
        [],
    )
    .unwrap()
    .unwrap();
    assert!(
        work.overlay_order_update()
            .unwrap()
            .bottom_to_top()
            .is_empty(),
        "an explicit overlay removal must clear the retained stack"
    );
}

#[test]
fn structural_portal_does_not_authorize_missing_backdrops_or_unordered_portal_paint() {
    let context = context();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let portal = UiOverlayParticipantIdentity::Portal(instance);
    let backdrop = UiMountedBackdropIdentity::from_runtime_mounting(
        "required.scrim",
        UiMountedBackdropScope::PerPortalInstance(instance),
        1,
    )
    .unwrap();
    assert_eq!(
        fragment(
            &context,
            vec![
                portal.clone(),
                UiOverlayParticipantIdentity::Backdrop(backdrop)
            ],
            vec![]
        ),
        Err(UiUnpublishedAppearanceFrameProjectionDenial::FragmentIdentityMismatch)
    );
    let foreign_instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let paint = UiMountedAppearanceMechanic::PortalSurface(
        UiMountedPortalSurfaceAppearanceMechanic::complete_from_runtime_mounting(
            foreign_instance,
            surface_at(context.frame, foreign_instance),
        )
        .unwrap(),
    );
    assert_eq!(
        fragment(&context, vec![portal], vec![paint]),
        Err(UiUnpublishedAppearanceFrameProjectionDenial::FragmentIdentityMismatch)
    );
}

fn fragment(
    context: &Context,
    participants: Vec<UiOverlayParticipantIdentity>,
    mechanics: Vec<UiMountedAppearanceMechanic>,
) -> Result<UiUnpublishedAppearanceFragment, UiUnpublishedAppearanceFrameProjectionDenial> {
    let order_changed = !participants.is_empty();
    let order = UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
        context.surface,
        context.attempt,
        1,
        1,
        participants,
    )
    .unwrap();
    let changes = mechanics
        .iter()
        .cloned()
        .map(UiMountedAppearanceMechanicChange::Insert)
        .collect::<Vec<_>>();
    let successor = UiMountedAppearanceFrame::from_runtime_mounting(
        context.frame,
        context.surface,
        mechanics,
        order,
    )
    .unwrap();
    let posture = if changes.is_empty() && !order_changed {
        UiMountedAppearanceWorkPosture::Unchanged
    } else {
        UiMountedAppearanceWorkPosture::Delta
    };
    let work = UiMountedAppearanceWork::from_runtime_mounting(
        posture,
        Some(context.predecessor),
        Some(UiMountedAppearancePredecessorManifest::from_runtime_mounting([], []).unwrap()),
        successor,
        changes,
        [],
        order_changed,
    )
    .unwrap();
    UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(context.surface),
        work,
        [],
        context.requirement,
        surface_affinity(context),
    )
}
