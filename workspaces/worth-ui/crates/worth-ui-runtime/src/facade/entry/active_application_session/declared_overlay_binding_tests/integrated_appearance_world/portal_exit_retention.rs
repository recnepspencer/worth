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

#[test]
fn indeterminate_exit_does_not_claim_a_sibling_presentation_completion() {
    use crate::certification_support::ScriptedSurfaceCompletion;
    use crate::facade::entry::active_application_session::{
        portal_exit_retention::UiPortalExitTerminalPending,
        UiPortalExitTerminalProgress as Progress,
    };
    use worth_ui_host_contract::{UiHostSurfaceCancellationOutcome, UiPresentationDeadline};
    use worth_ui_host_native::{
        UiNativePhysicalPresentationCorrelation as Correlation,
        UiNativePhysicalProgressClass as Class,
    };

    let mut world = World::launch();
    let initial = world.prepare();
    world.publish(initial, 1, true);
    let parent = world.open(0, "overlay.menu", None, 10);
    let child = world.open(2, "overlay.child", Some(parent), 11);
    world.host.push_native_display_presented();
    world.host.push_native_display_settled_without_effects();
    assert!(matches!(
        world
            .session
            .publish_anchor_loss_portal_dismissal(child, 41),
        Outcome::Published(_)
    ));
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
    world.host.push_in_flight(
        vec![ScriptedSurfaceCompletion::PresentationIndeterminate],
        UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun,
    );
    world.host.push_native_display_settled_without_effects();
    assert_eq!(
        world.session.progress_portal_exit_terminal(114),
        Progress::AwaitingPhysical
    );
    assert_eq!(
        world.session.complete_portal_exit_terminal_physical(115),
        Progress::AwaitingPhysical
    );
    let Some(UiPortalExitTerminalPending::Indeterminate { track, recovery }) =
        world.session.take_portal_exit_terminal_pending()
    else {
        panic!("real terminal completion must retain indeterminate recovery");
    };
    let (frame, proposal) = recovery.into_parts();
    let report = frame.report();
    let exact = Correlation::from_certification(
        report.attempt(),
        world.surfaces[0],
        report.physical_recovery_bindings()[0],
        1,
    )
    .unwrap();
    let recovery =
        crate::facade::entry::portal_dismissal::DetachedUiPortalDismissalIndeterminate::from_parts(
            world.session.session_identity(),
            frame,
            proposal,
        );
    world
        .session
        .retain_portal_exit_terminal_pending(UiPortalExitTerminalPending::Indeterminate {
            track,
            recovery,
        });
    assert!(world
        .session
        .pending_portal_exit_terminal_matches_native_physical(
            Class::PresentationRecovery,
            Some(exact)
        ));
    assert!(!world
        .session
        .pending_portal_exit_terminal_matches_native_physical(Class::Presentation, None));
    assert!(!world
        .session
        .pending_portal_exit_terminal_matches_native_physical(Class::TextAtlas, Some(exact)));

    // A second live session owns a distinct physical publication while this exit waits.
    let mut sibling_world = World::launch();
    let initial = sibling_world.prepare();
    sibling_world.publish(initial, 1, true);
    let sibling = sibling_world.prepare_surface(sibling_world.surfaces[1]);
    sibling_world.host.push_in_flight(
        vec![ScriptedSurfaceCompletion::Presented(
            worth_ui_host_contract::UiMountedSurfacePresentationCompletion::new(
                worth_ui_host_contract::UiHostSurfacePresentationMode::NativeDisplay,
                worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(1000),
                worth_ui_host_contract::UiMountedCompletedEffects::new(Vec::new()),
                Default::default(),
            ),
        )],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let crate::mounting::UiMountedFrameOutcome::InFlight(sibling) = sibling_world
        .session
        .present_prepared_mounted_frame_internal(
            sibling,
            UiPresentationDeadline::at_tick(u64::MAX),
            2,
        )
    else {
        panic!("second session owns a real pending publication");
    };
    let sibling_event = Correlation::from_certification(
        sibling.attempt(),
        sibling_world.surfaces[1],
        sibling.pending_bindings().next().unwrap(),
        1,
    )
    .unwrap();
    assert!(
        !world
            .session
            .pending_portal_exit_terminal_matches_native_physical(
                Class::Presentation,
                Some(sibling_event)
            ),
        "the driver's first-refusal predicate must let the sibling owner receive its completion"
    );
    assert!(matches!(
        sibling_world
            .session
            .complete_mounted_presentation(sibling, 3),
        crate::mounting::UiMountedFrameOutcome::Published(_)
            | crate::mounting::UiMountedFrameOutcome::Superseded(_)
    ));
    assert!(sibling_world
        .session
        .shutdown()
        .runtime_service_resource_census()
        .is_empty());
    let shutdown = world.session.shutdown();
    assert!(shutdown.runtime_service_resource_census().is_empty());
}
