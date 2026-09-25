//! Real input admission preserves event-time meaning, not raw coordinates.
use super::{authored, input_completion_order::drain_outside_press, session::World};
use crate::certification_support::{
    ScriptedPresentationAcknowledgement, ScriptedPresentationOutcome,
};
use crate::facade::entry::portal_dismissal::{
    UiPortalDismissalPublicationOutcome as Outcome, UiPortalDismissalPublicationStop as Stop,
};
use worth_ui_host_contract::*;

fn opened() -> (World, crate::runtime::portal::UiPortalIdentity) {
    let source = authored::source().replace(
        "dismiss escape anchor_gone",
        "dismiss escape outside_press anchor_gone",
    );
    let mut world = World::launch_with_source(false, false, source);
    let initial = world.prepare();
    world.publish(initial, 1, true);
    let portal = world.open(0, "overlay.menu", None, 10);
    (world, portal)
}

pub(super) fn advance_motion(world: &mut World, tick: u64, epoch: u64) {
    let basis = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    let sample = world.session.prepare_motion_tick(tick, basis).unwrap();
    world
        .host
        .push_presentation(ScriptedPresentationOutcome::Presented(
            ScriptedPresentationAcknowledgement::new(
                UiHostSurfacePresentationMode::NativeDisplay,
                UiHostPresentationEpoch::issued_by_host(epoch),
                UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
                Default::default(),
            ),
        ));
    world.session.present_prepared_motion_tick(sample, basis);
    assert_eq!(
        world
            .session
            .mounted
            .current_presentation_for_surface(world.surfaces[0])
            .unwrap()
            .basis()
            .epoch(),
        UiHostPresentationEpoch::issued_by_host(epoch)
    );
}

#[test]
fn inside_decision_cannot_turn_into_outside_when_accepted_geometry_moves() {
    let (mut world, portal) = opened();
    let body = world
        .session
        .portal
        .as_ref()
        .unwrap()
        .placement(portal)
        .unwrap()
        .prepared()
        .bounds()
        .rect()
        .canonical_box();
    advance_motion(&mut world, 1, 31);
    let basis = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    // Entrance starts eight points below the final body. This point is in
    // the accepted lower strip, but outside the body once entrance completes.
    let point = UiHostSurfacePosition::viewport_logical(
        ((body.x() + body.width() / 2.0) * 1_000.0) as i64,
        ((body.y() + body.height() + 4.0) * 1_000.0) as i64,
    );
    let admitted = drain_outside_press(&mut world, basis.basis(), point, 1);
    advance_motion(&mut world, 10_000, 32);
    let revision = world.session.portal.as_ref().unwrap().revision();
    assert!(matches!(
        world
            .session
            .publish_admitted_portal_dismissal(admitted, 10_001),
        Outcome::IgnoredInsideTopmostPortal
    ));
    assert_eq!(world.session.portal.as_ref().unwrap().revision(), revision);
    let current = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    let fresh = crate::facade::interaction::UiDismissInteraction::outside_press(
        current.basis(),
        UiHostObservationSequence::new(2),
        UiHostObservationTimeBasis::PresentationRelativeTick(10_002),
        point,
    );
    for _ in world.surfaces {
        world.host.push_native_display_presented();
    }
    assert!(
        matches!(
            world.session.publish_portal_dismissal(fresh, 10_002),
            Outcome::Published(_)
        ),
        "same coordinates must really be outside the new accepted geometry"
    );
    assert!(world
        .session
        .shutdown()
        .runtime_service_resource_census()
        .is_empty());
}

#[test]
fn admitted_dismissal_cannot_retarget_a_new_topmost_modal() {
    let (mut world, parent) = opened();
    let basis = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    let admitted = drain_outside_press(
        &mut world,
        basis.basis(),
        UiHostSurfacePosition::viewport_logical(790_000, 590_000),
        1,
    );
    let child = world.open(2, "overlay.child", Some(parent), 20);
    let revision = world.session.portal.as_ref().unwrap().revision();
    assert!(matches!(
        world
            .session
            .publish_admitted_portal_dismissal(admitted, 21),
        Outcome::Stopped(Stop::InteractionCancelled)
    ));
    assert_eq!(world.session.portal.as_ref().unwrap().revision(), revision);
    assert!(world
        .session
        .portal
        .as_ref()
        .unwrap()
        .placement(parent)
        .is_some());
    assert!(world
        .session
        .portal
        .as_ref()
        .unwrap()
        .placement(child)
        .is_some());
    assert_eq!(
        world.host.pending_presentation_count(),
        0,
        "cancellation starts no host effects"
    );
    assert!(world
        .session
        .shutdown()
        .runtime_service_resource_census()
        .is_empty());
}

#[test]
fn admitted_dismissal_is_bound_to_its_application_generation() {
    let (mut origin, _) = opened();
    let basis = origin
        .session
        .mounted
        .current_presentation_for_surface(origin.surfaces[0])
        .unwrap();
    let admitted = drain_outside_press(
        &mut origin,
        basis.basis(),
        UiHostSurfacePosition::viewport_logical(790_000, 590_000),
        1,
    );
    let (mut successor, portal) = opened();
    let revision = successor.session.portal.as_ref().unwrap().revision();
    assert!(matches!(
        successor
            .session
            .publish_admitted_portal_dismissal(admitted, 21),
        Outcome::Stopped(Stop::InteractionCancelled)
    ));
    assert_eq!(
        successor.session.portal.as_ref().unwrap().revision(),
        revision
    );
    assert!(successor
        .session
        .portal
        .as_ref()
        .unwrap()
        .placement(portal)
        .is_some());
    assert!(origin
        .session
        .shutdown()
        .runtime_service_resource_census()
        .is_empty());
    assert!(successor
        .session
        .shutdown()
        .runtime_service_resource_census()
        .is_empty());
}
