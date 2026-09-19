use super::{authored, session::World};
use worth_ui_host_contract::*;

#[test]
fn retained_inside_press_is_ignored_but_cannot_dismiss_after_same_frame_epoch_advance() {
    use crate::facade::entry::portal_dismissal::{
        UiPortalDismissalPublicationOutcome as Outcome, UiPortalDismissalPublicationStop as Stop,
    };

    let source = authored::source().replace(
        "dismiss escape anchor_gone",
        "dismiss escape outside_press anchor_gone",
    );
    let mut world = World::launch_with_source(false, false, source);
    let initial = world.prepare();
    world.publish(initial, 1, true);
    let portal = world.open(0, "overlay.menu", None, 10);
    let observed = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();

    let sample = world.session.prepare_motion_tick(1, observed).unwrap();
    world
        .host
        .push_presentation(UiHostSurfacePresentationOutcome::Presented(
            UiMountedSurfacePresentationCompletion::new(
                UiHostSurfacePresentationMode::NativeDisplay,
                UiHostPresentationEpoch::issued_by_host(31),
                UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
                UiHostPresentationCostReport::default(),
            ),
        ));
    world.session.present_prepared_motion_tick(sample, observed);
    let current = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    assert_eq!(current.frame(), observed.frame());
    assert_eq!(current.binding(), observed.binding());
    assert_ne!(current.epoch(), observed.epoch());

    let bounds = world
        .session
        .portal
        .as_ref()
        .unwrap()
        .placement(portal)
        .unwrap()
        .prepared()
        .bounds()
        .mounted_box();
    let position = |x: f32, y: f32| {
        UiHostSurfacePosition::viewport_logical((x * 1_000.0) as i64, (y * 1_000.0) as i64)
    };
    let interaction = |point| {
        crate::facade::interaction::UiDismissInteraction::outside_press(
            observed,
            UiHostObservationSequence::new(100),
            UiHostObservationTimeBasis::PresentationRelativeTick(100),
            point,
        )
    };
    let revision = world.session.portal.as_ref().unwrap().revision();

    match world.session.publish_portal_dismissal(
        interaction(position(
            bounds.x() + bounds.width() / 2.0,
            bounds.y() + bounds.height() / 2.0,
        )),
        100,
    ) {
        Outcome::IgnoredInsideTopmostPortal => {}
        Outcome::IgnoredNoMatchingPortal => panic!("inside press found no dismissible Portal"),
        Outcome::Stopped(stop) => panic!("inside press stopped: {stop:?}"),
        Outcome::Published(_) => panic!("inside press published a dismissal"),
        Outcome::InFlight(_) => panic!("inside press started host work"),
        Outcome::Indeterminate(_) => panic!("inside press started indeterminate host work"),
    }
    assert_eq!(world.session.portal.as_ref().unwrap().revision(), revision);

    assert!(matches!(
        world.session.publish_portal_dismissal(
            interaction(position(
                bounds.x() + bounds.width() + 1.0,
                bounds.y() + bounds.height() + 1.0,
            )),
            101,
        ),
        Outcome::Stopped(Stop::StalePresentation)
    ));
    assert_eq!(world.session.portal.as_ref().unwrap().revision(), revision);
    let _ = world.session.shutdown();
}

#[test]
fn portal_entrance_dismissal_uses_the_accepted_displaced_body() {
    use crate::facade::entry::portal_dismissal::UiPortalDismissalPublicationOutcome as Outcome;
    let source = authored::source().replace(
        "dismiss escape anchor_gone",
        "dismiss escape outside_press anchor_gone",
    );
    let mut world = World::launch_with_source(false, false, source);
    let initial = world.prepare();
    world.publish(initial, 1, true);
    let portal = world.open(0, "overlay.menu", None, 10);
    let observed = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    let body = world
        .session
        .portal
        .as_ref()
        .unwrap()
        .placement(portal)
        .unwrap()
        .prepared()
        .bounds()
        .mounted_box();
    let sample = world.session.prepare_motion_tick(1, observed).unwrap();
    world
        .host
        .push_presentation(UiHostSurfacePresentationOutcome::Presented(
            UiMountedSurfacePresentationCompletion::new(
                UiHostSurfacePresentationMode::NativeDisplay,
                UiHostPresentationEpoch::issued_by_host(31),
                UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
                UiHostPresentationCostReport::default(),
            ),
        ));
    world.session.present_prepared_motion_tick(sample, observed);
    let current = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    let interaction = |y: f32, sequence| {
        crate::facade::interaction::UiDismissInteraction::outside_press(
            current,
            UiHostObservationSequence::new(sequence),
            UiHostObservationTimeBasis::PresentationRelativeTick(sequence),
            UiHostSurfacePosition::viewport_logical(
                ((body.x() + body.width() / 2.0) * 1_000.0) as i64,
                (y * 1_000.0) as i64,
            ),
        )
    };
    // The entrance translates the body down eight points. This lower strip is
    // visibly inside although it lies beyond the unsampled body rectangle.
    assert!(matches!(
        world
            .session
            .publish_portal_dismissal(interaction(body.y() + body.height() + 4.0, 100), 100),
        Outcome::IgnoredInsideTopmostPortal
    ));
    // Dismissal republishes this world's two mounted surfaces, including the
    // secondary surface whose ambient backdrop remains present.
    for _ in world.surfaces {
        world
            .host
            .push_presentation(UiHostSurfacePresentationOutcome::Presented(
                UiMountedSurfacePresentationCompletion::new(
                    UiHostSurfacePresentationMode::NativeDisplay,
                    UiHostPresentationEpoch::issued_by_host(32),
                    UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
                    UiHostPresentationCostReport::default(),
                ),
            ));
    }
    // Conversely, the old upper strip is now outside the accepted body.
    assert!(matches!(
        world
            .session
            .publish_portal_dismissal(interaction(body.y() + 4.0, 101), 101),
        Outcome::Published(_)
    ));
    let _ = world.session.shutdown();
}
