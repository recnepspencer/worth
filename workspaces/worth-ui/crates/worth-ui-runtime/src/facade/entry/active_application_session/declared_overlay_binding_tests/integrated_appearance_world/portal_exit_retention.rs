use super::session::World;
use crate::facade::entry::portal_dismissal::{
    UiPortalDismissalPublicationOutcome as Outcome, UiPortalDismissalPublicationStop as Stop,
};
use crate::runtime::portal::UiPortalLifecyclePosture;

#[test]
fn parent_close_denies_before_effects_while_child_exit_retention_settles() {
    let mut world = World::launch();
    let initial = world.prepare();
    world.publish(initial, 1, true);
    let parent = world.open(0, "overlay.menu", None, 10);
    let child = world.open(2, "overlay.child", Some(parent), 11);

    world.host.push_native_display_presented();
    world.host.push_native_display_settled_without_effects();
    match world
        .session
        .publish_anchor_loss_portal_dismissal(child, 41)
    {
        Outcome::Published(_) => {}
        Outcome::Stopped(stop) => panic!("the authored child dismissal stopped: {stop:?}"),
        _ => panic!("the authored child dismissal must publish its retained exit"),
    };
    for tick in [1, 112] {
        let presentation = world
            .session
            .mounted
            .current_presentation_for_surface(world.surfaces[0])
            .unwrap();
        let sample = world
            .session
            .prepare_motion_tick(tick, presentation)
            .unwrap();
        world.host.push_native_display_presented();
        world
            .session
            .present_prepared_motion_tick(sample, presentation);
    }
    let before_parent_close = world.session.inspect_portal_runtime_for_certification();
    assert_eq!(before_parent_close.active_portals(), 2);
    assert_eq!(before_parent_close.closing_portals(), 1);
    assert_eq!(before_parent_close.portal_exit_retentions(), 1);
    assert_retention_census(&world.session);
    let calls_before_parent_close = world.host.presentation_calls();
    let frame_before_parent_close = world.session.current_mounted_publication().unwrap().frame();

    assert!(matches!(
        world
            .session
            .publish_anchor_loss_portal_dismissal(parent, 113),
        Outcome::Stopped(Stop::Transition)
    ));
    assert_eq!(
        world.session.inspect_portal_runtime_for_certification(),
        before_parent_close,
        "ancestor denial preserves the child's exact retention and the parent Portal row"
    );
    assert_eq!(world.host.presentation_calls(), calls_before_parent_close);
    assert_eq!(
        world.session.current_mounted_publication().unwrap().frame(),
        frame_before_parent_close,
        "ancestor denial cannot replace accepted paint"
    );
    assert_retention_census(&world.session);

    world.host.push_native_display_presented();
    world.host.push_native_display_settled_without_effects();
    assert_eq!(
        world.session.progress_portal_exit_terminal(114),
        crate::facade::entry::active_application_session::UiPortalExitTerminalProgress::Published
    );
    assert!(world.host.presentation_calls() > calls_before_parent_close);
    let child_closed = world.session.inspect_portal_runtime_for_certification();
    assert_eq!(child_closed.active_portals(), 1);
    assert_eq!(child_closed.closing_portals(), 0);
    assert_eq!(child_closed.portal_exit_retentions(), 0);
    let portal = world.session.portal.as_ref().unwrap();
    assert_eq!(portal.posture(parent), UiPortalLifecyclePosture::Visible);
    assert_eq!(portal.posture(child), UiPortalLifecyclePosture::Closed);
    assert_retention_census(&world.session);
    assert!(world
        .session
        .inspect_service_proposals_for_certification()
        .is_zero());

    let shutdown = world.session.shutdown();
    assert!(shutdown.runtime_service_resource_census().is_empty());
    assert_eq!(shutdown.portal_final_active_records(), 0);
    assert_eq!(shutdown.motion_cancelled_exit_retentions(), 0);
    assert!(shutdown.motion_final_census_is_zero());
}

fn assert_retention_census(session: &crate::facade::entry::WorthUiActiveApplicationSession) {
    let portal = session.inspect_portal_runtime_for_certification();
    let census = session.runtime_service_resource_census();
    assert_eq!(
        portal.portal_exit_retentions(),
        census.portal_exit_retentions()
    );
    assert_eq!(
        usize::from(census.motion_exit_retentions()),
        census.portal_exit_retentions(),
        "Motion and Portal retain the same coordinator-owned exit"
    );
    assert_eq!(portal.closing_portals(), portal.portal_exit_retentions());
    assert!(portal.pending_track_coordinated());
}
