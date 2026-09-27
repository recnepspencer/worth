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
    let point = crate::units::viewport_position_for_test(
        body.x() + body.width() / 2.0,
        body.y() + body.height() + 4.0,
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

fn center([x, y, width, height]: [f32; 4]) -> UiHostSurfacePosition {
    crate::units::viewport_position_for_test(x + width / 2.0, y + height / 2.0)
}

/// Press outside at the center of `bounds`, against what is shown now, once
/// a Motion sample on screen is what places the Portal there.
fn press_where_sampled(
    world: &mut World,
    portal: crate::runtime::portal::UiPortalIdentity,
    sequence: u64,
    tick: u64,
    bounds: [f32; 4],
) -> Outcome<'_> {
    let current = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap()
        .basis();
    let target = crate::runtime::motion::UiMotionTargetIdentity::from_portal_owner(
        world.surfaces[0],
        portal.owner().mounted_instance_identity(),
        portal.diagnostic_value(),
    );
    assert!(
        world
            .session
            .mounted
            .committed_motion_geometry_for_target(target, current)
            .unwrap()
            .is_some(),
        "a Motion sample on screen places the popover"
    );
    let press = crate::facade::interaction::UiDismissInteraction::outside_press(
        current,
        UiHostObservationSequence::new(sequence),
        UiHostObservationTimeBasis::PresentationRelativeTick(tick),
        center(bounds),
    );
    world.session.publish_portal_dismissal(press, tick)
}

/// Dismissal reads the placement the accepted frame committed. Once a frame
/// moves an open popover with its anchor, a press where the popover now
/// stands is inside it, and a press where it stood before is outside.
#[test]
fn a_moved_popover_is_dismissed_where_it_left_and_kept_where_it_stands() {
    use super::geometry::{install_owner_in_viewport, MOVED_TARGET_BOX, VIEWPORT};
    use super::portal_placement_succession::committed_bounds;

    let (mut world, portal) = opened();
    let opened_at = committed_bounds(&world, portal);

    install_owner_in_viewport(
        &mut world.session,
        world.surfaces,
        world.instances,
        20,
        MOVED_TARGET_BOX,
        VIEWPORT,
    );
    let moved = world.prepare();
    world.publish_as_issued(moved, 20);
    let moved_to = committed_bounds(&world, portal);
    let [x, y, width, height] = moved_to;
    let [left_x, left_y] = [
        opened_at[0] + opened_at[2] / 2.0,
        opened_at[1] + opened_at[3] / 2.0,
    ];
    assert!(
        !(x..x + width).contains(&left_x) || !(y..y + height).contains(&left_y),
        "the popover moved off the place it opened"
    );

    let revision = world.session.portal.as_ref().unwrap().revision();
    assert!(matches!(
        press_where_sampled(&mut world, portal, 1, 30, moved_to),
        Outcome::IgnoredInsideTopmostPortal
    ));
    assert_eq!(world.session.portal.as_ref().unwrap().revision(), revision);

    for _ in world.surfaces {
        world.host.push_native_display_as_issued();
    }
    assert!(
        matches!(
            press_where_sampled(&mut world, portal, 2, 31, opened_at),
            Outcome::Published(_)
        ),
        "where the popover opened is outside it once it has moved"
    );
    assert!(world
        .session
        .shutdown()
        .runtime_service_resource_census()
        .is_empty());
}
