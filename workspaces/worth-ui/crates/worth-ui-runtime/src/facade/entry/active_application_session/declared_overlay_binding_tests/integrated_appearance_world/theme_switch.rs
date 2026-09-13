use super::session::World;
use crate::runtime::appearance::{
    UiThemeSwitchOriginFamily, UiThemeSwitchOutcome, UiThemeSwitchRequest,
};
use crate::runtime::rebind::{UiRebindExecutionPolicy, UiRebindExecutionRequest};
use worth_ui_host_contract::*;
#[path = "theme_switch/authority_recovery.rs"]
mod authority_recovery;
#[path = "theme_switch/equal.rs"]
mod equal;
#[path = "theme_switch/lifecycle.rs"]
mod lifecycle;
#[path = "theme_switch/product_text.rs"]
mod product_text;
#[path = "theme_switch/shutdown.rs"]
mod shutdown;

#[test]
fn portal_motion_modality_complete_ap07_journey() {
    let (mut world, selection_option) = super::hostile_protocol::launch_query_world();
    super::hostile_protocol::establish_six_axis_target(&mut world, selection_option);
    super::captured_geometry::move_and_stage_restored_target(&mut world, 3);
    super::captured_geometry::assert_moved_capture(&world);
    let parent = world.open_inspecting_frame(0, "overlay.menu", None, 18, |frame| {
        super::captured_geometry::assert_restored_portal_selection(frame);
    });
    super::captured_geometry::assert_restored_geometry_damage(&world);
    super::captured_geometry::assert_restored_capture(&world);
    super::captured_geometry::release_target(&mut world, 4, true);
    super::focus_modality::exercise_window_focus(&mut world, 5);
    world.open_inspecting_frame(1, "overlay.menu", None, 19, |frame| {
        super::focus_modality::assert_inactive_portal_selection(frame);
    });
    super::focus_modality::assert_inactive_publication(&world);
    super::captured_geometry::press_target(&mut world, 6);
    let child = world.open(2, "overlay.child", Some(parent), 20);
    super::captured_geometry::release_target(&mut world, 7, false);
    super::hostile_protocol::close_owner_snapshot(&mut world);
    world.sample(world.surfaces[0], 1, 31, 0);
    world.sample(world.surfaces[0], 71, 33, 57_343);
    let output = world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    super::assert_portal_backdrop_order(output, &world);
    lifecycle::finish_motion(&mut world);
    shutdown::with_portal_exit(world, parent, child);
}

fn switch_plan(
    world: &mut World,
    definition: &str,
    revision: u64,
) -> crate::runtime::rebind::UiRebindPlan {
    let source = crate::runtime::WorthUiReloadDebounce::default()
        .debounce(
            crate::runtime::WorthUiSourceProvider::in_memory("integrated-overlay")
                .with_file("app/main.wui", super::authored::ap07_source()),
            &[crate::runtime::WorthUiWatcherEvent::provider_revision(
                "integrated-overlay",
            )],
            revision,
        )
        .unwrap()
        .attempt_candidate_for_certification(world.session.application.capabilities())
        .unwrap();
    let mut turn = world.session.begin_observation_turn().unwrap();
    turn.admit_source(source).unwrap();
    let observations = turn.seal().unwrap();
    let origin = world
        .session
        .issue_theme_switch_origin(
            &observations,
            UiThemeSwitchOriginFamily::SourceEditObservation,
        )
        .unwrap();
    world.session.classify_observations(observations).unwrap();
    let surface = world.surfaces[0];
    let capability = world
        .session
        .admit_appearance_theme(
            surface,
            &crate::capability::UiThemeDefinitionIdentity::new(definition).unwrap(),
        )
        .unwrap();
    let generation = world
        .session
        .active_theme_binding(surface)
        .unwrap()
        .binding_generation();
    let crate::runtime::observation::UiChangeClassificationOutcome::Changed(change) = world
        .session
        .prepare_theme_switch(UiThemeSwitchRequest::new(
            origin, surface, generation, capability,
        ))
        .unwrap()
    else {
        panic!("distinct definition has a binding settlement even when values equal");
    };
    let scope = world.session.resolve_affected_scope(change).unwrap();
    assert_eq!(
        scope.cost().lookup_receipts(),
        0,
        "theme changes are indexed dependencies, not source facts"
    );
    let changed_slots = match definition {
        "theme.integrated.equal" => 0,
        "theme.integrated.unused" => 2,
        "theme.integrated.backdrop" => 2,
        "theme.integrated.green" => 6,
        "theme.integrated.overlay" => 6,
        _ => unreachable!("the integrated theme matrix is closed"),
    };
    assert_eq!(
        scope.cost().theme_slots_compared(),
        changed_slots,
        "changed-slot comparison count for {definition}"
    );
    if changed_slots == 0 {
        assert_eq!(scope.cost().index_probes(), 0);
        assert_eq!(scope.cost().graph_and_mounted_entries(), 0);
    } else {
        assert!(scope.cost().index_probes() > 0);
    }
    world
        .session
        .compile_rebind_plan(
            scope.resolve_identity_lifecycle().unwrap(),
            UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap()
}
