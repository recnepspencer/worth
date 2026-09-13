use super::{session::World, *};
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
            ScriptedSurfaceCompletion::Presented(UiMountedSurfacePresentationCompletion::new(
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
