use crate::runtime::appearance::UiThemeSwitchOutcome;
use crate::runtime::rebind::UiRebindExecutionRequest;

#[test]
fn equal_and_unused_theme_switches_settle_without_appearance_work() {
    let registration = super::super::hostile_protocol::selection_registration();
    let mut world = super::super::session::World::launch_ap07(
        registration,
        super::super::authored::ap07_source(),
    );
    world.session.unmount_instance(world.instances[1]).unwrap();
    let regions = super::super::hostile_protocol::regions(&world);
    super::super::geometry::install_without_target(
        &mut world.session,
        world.surfaces,
        world.instances,
        regions,
    );
    let initial = world.prepare();
    world.publish(initial, 1, true);

    let neighbor = world.surfaces[1];
    let neighbor_binding = world
        .session
        .active_theme_binding(neighbor)
        .unwrap()
        .clone();
    let neighbor_presentation = world
        .session
        .mounted
        .current_presentation_for_surface(neighbor)
        .unwrap();

    accept_no_work(&mut world, "theme.integrated.equal", 1, 101);
    accept_no_work(&mut world, "theme.integrated.unused", 2, 201);
    assert_eq!(
        world.session.active_theme_binding(neighbor),
        Some(&neighbor_binding)
    );
    assert_eq!(
        world
            .session
            .mounted
            .current_presentation_for_surface(neighbor),
        Some(neighbor_presentation)
    );

    let receipt = world.session.shutdown();
    assert!(receipt.runtime_service_resource_census().is_empty());
    assert!(receipt.intent_resource_census().is_empty());
    assert!(receipt.rebind().is_empty());
    assert!(receipt.mounted_presentation().query_close_complete());
    assert!(receipt
        .mounted_presentation()
        .query_transition_trace_complete());
    assert_eq!(world.host.native_in_flight_count(), 0);
    assert_eq!(world.host.pending_presentation_count(), 0);
}

fn accept_no_work(
    world: &mut super::super::session::World,
    definition: &str,
    revision: u64,
    now: u64,
) {
    let surface = world.surfaces[0];
    let predecessor = world.session.active_theme_binding(surface).unwrap().clone();
    let plan = super::switch_plan(world, definition, revision);
    world.host.push_native_display_settled_without_effects();
    let prepared = world
        .session
        .prepare_rebind(plan, UiRebindExecutionRequest::new(now))
        .unwrap();
    assert_eq!(
        prepared
            .prepared_frame()
            .unwrap()
            .appearance_selection_cost_report()
            .selected_instance_count(),
        0
    );
    prepared
        .prepared_frame()
        .unwrap()
        .assert_no_unpublished_appearance_for_test();
    assert!(matches!(
        UiThemeSwitchOutcome::from(prepared.execute(now)),
        UiThemeSwitchOutcome::Published(_)
    ));
    assert_ne!(
        world.session.active_theme_binding(surface),
        Some(&predecessor),
        "the {definition} binding settles despite zero mounted appearance work"
    );
    assert!(world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .is_none());
}
