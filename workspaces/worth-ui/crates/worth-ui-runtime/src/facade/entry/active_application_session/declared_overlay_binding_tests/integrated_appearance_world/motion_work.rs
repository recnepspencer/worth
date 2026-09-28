use super::{session::World, *};
use crate::certification_support::ScriptedPresentationAcknowledgement;
use crate::certification_support::ScriptedSurfaceCompletion;

#[test]
fn accepted_motion_reports_hit_index_work_without_pointer_records_through_delayed_settlement() {
    let mut world = World::launch();
    let frame = world.prepare();
    world.publish(frame, 1, true);
    let parent = world.open(0, "overlay.menu", None, 10);
    world.open(1, "overlay.menu", None, 11);
    world.open(2, "overlay.child", Some(parent), 12);
    let surface = world.surfaces[0];
    let frame = world.prepare_surface_with_current_portals(surface);
    world.publish(frame, 30, false);
    assert_eq!(
        world.session.interaction_state().pointer_presence_records(),
        0
    );
    assert_eq!(world.session.last_motion_sampling_cost(), None);
    let basis = world
        .session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let prepared = world.session.prepare_motion_tick(1, basis).unwrap();
    world.host.push_rejected();
    world.session.present_prepared_motion_tick(prepared, basis);
    assert_eq!(world.session.last_motion_sampling_cost(), None);
    assert_eq!(
        world
            .session
            .mounted
            .current_presentation_for_surface(surface),
        Some(basis)
    );
    world.sample(surface, 1, 31, 0);
    let accepted = world.session.last_motion_sampling_cost().unwrap();
    assert_cost(accepted);
    let basis = world
        .session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let prepared = world.session.prepare_motion_tick(71, basis).unwrap();
    world.host.push_in_flight(
        vec![
            ScriptedSurfaceCompletion::Pending,
            ScriptedSurfaceCompletion::Presented(ScriptedPresentationAcknowledgement::new(
                UiHostSurfacePresentationMode::NativeDisplay,
                UiHostPresentationEpoch::issued_by_host(33),
                UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
                UiHostPresentationCostReport::default(),
            )),
        ],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    world.session.present_prepared_motion_tick(prepared, basis);
    assert!(world.session.mounted.motion_sample_presentation_pending());
    assert_eq!(world.session.last_motion_sampling_cost(), Some(accepted));
    world.session.complete_motion_sample_presentation();
    assert!(world.session.mounted.motion_sample_presentation_pending());
    assert_eq!(world.session.last_motion_sampling_cost(), Some(accepted));
    world.session.complete_motion_sample_presentation();
    assert!(!world.session.mounted.motion_sample_presentation_pending());
    assert_cost(world.session.last_motion_sampling_cost().unwrap());
    assert_eq!(
        world
            .session
            .inspect_motion_presentation_for_certification()
            .opacity_units(),
        Some(57_343)
    );
    assert_eq!(
        world.session.interaction_state().pointer_presence_records(),
        0
    );
    let _ = world.session.shutdown();
    assert_eq!(world.host.pending_presentation_count(), 0);
}

fn assert_cost(cost: crate::facade::mounted::UiPresentationMotionSamplingCost) {
    assert_eq!(
        cost.tracks_considered(),
        3,
        "three declared Portal entrances are sampled"
    );
    assert!(cost.hit_index_work().motion_members_visited() > 0);
    assert!(cost.hit_index_work().motion_rows_projected() > 0);
    assert_eq!(cost.hit_index_work().reconstructed_rows(), 0);
}

/// A frame issued while a Portal's tick is in flight lands after the tick, so
/// content it lays out anew inside that Portal must stand where the tick left
/// the Portal, not where the frame's predecessor showed it.
#[test]
fn portal_content_laid_out_while_a_tick_is_in_flight_lands_at_the_ticks_layer() {
    let mut world = World::launch();
    let frame = world.prepare();
    world.publish(frame, 1, true);
    let parent = world.open(0, "overlay.menu", None, 10);
    world.open(1, "overlay.menu", None, 11);
    world.open(2, "overlay.child", Some(parent), 12);
    let surface = world.surfaces[0];
    let frame = world.prepare_surface_with_current_portals(surface);
    world.publish(frame, 30, false);
    world.sample(surface, 1, 31, 0);
    // The owner lays the Portal's content out anew and its frame is prepared
    // before the tick is issued; the frame is presented while the tick is in
    // flight.
    super::geometry::install_child_in_viewport(
        &mut world.session,
        world.surfaces,
        world.instances,
        40,
        [8.0, 12.0, 300.0, 90.0],
        super::geometry::VIEWPORT,
    );
    let frame = world.prepare_surface_with_current_portals(surface);

    let basis = world
        .session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let prepared = world.session.prepare_motion_tick(71, basis).unwrap();
    world.host.push_in_flight(
        vec![
            ScriptedSurfaceCompletion::Pending,
            ScriptedSurfaceCompletion::Presented(ScriptedPresentationAcknowledgement::new(
                UiHostSurfacePresentationMode::NativeDisplay,
                UiHostPresentationEpoch::issued_by_host(33),
                UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
                UiHostPresentationCostReport::default(),
            )),
        ],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    world.session.present_prepared_motion_tick(prepared, basis);
    let tick = world.host.last_motion_samples();
    assert!(world.session.mounted.motion_sample_presentation_pending());

    let pending = world.begin_in_flight(frame, 40);
    world.session.complete_motion_sample_presentation();
    world.session.complete_motion_sample_presentation();
    assert!(!world.session.mounted.motion_sample_presentation_pending());
    let _ = world.session.complete_mounted_presentation(pending, 41);

    // Everything the frame states of the Portals, what it laid out anew
    // and what it left alone, stands where the tick left them.
    let stated = world.host.last_appearance_samples();
    assert!(stated.len() >= tick.len());
    for change in &stated {
        assert_eq!(change.opacity().motion_units(), 57_343, "{change:?}");
    }
    let current = world
        .session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    for moved in &tick {
        let shown = stated
            .iter()
            .find(|change| change.command() == moved.command())
            .expect("the frame states each command the tick moved");
        assert_eq!(shown.transform(), moved.transform());
        let accepted = world
            .session
            .mounted
            .accepted_motion_for_command(current.basis(), moved.command())
            .unwrap()
            .unwrap();
        assert_eq!(accepted.tick(), 71, "interaction reads what the host shows");
    }
    let _ = world.session.shutdown();
}
