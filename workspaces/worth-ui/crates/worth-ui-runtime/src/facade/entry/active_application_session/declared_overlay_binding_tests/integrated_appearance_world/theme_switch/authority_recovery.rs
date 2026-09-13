use super::*;

#[test]
fn six_axis_authority_and_theme_recovery_complete_ap07_journey() {
    let (mut world, option) = super::super::hostile_protocol::launch_query_world();
    super::super::hostile_protocol::establish_six_axis_target(&mut world, option);
    super::product_text::assert_current_ab(&world);
    let surface = world.surfaces[0];
    let predecessor_theme = world.session.active_theme_binding(surface).unwrap().clone();
    let predecessor_paint = world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .cloned();
    let predecessor_focus = world
        .session
        .focus
        .as_ref()
        .unwrap()
        .current_semantic_focus();

    let plan = super::switch_plan(&mut world, "theme.integrated.green", 4);
    world.host.push_rejected();
    let prepared = world
        .session
        .prepare_rebind(plan, UiRebindExecutionRequest::new(440))
        .unwrap();
    assert_eq!(
        prepared
            .prepared_frame()
            .unwrap()
            .appearance_selection_cost_report()
            .selected_instance_count(),
        4
    );
    let UiThemeSwitchOutcome::RejectedBeforeEffects(rejected) =
        UiThemeSwitchOutcome::from(prepared.execute(440))
    else {
        panic!("green switch must exercise before-effects rejection")
    };
    let retry = match rejected.detach_retry_for_native() {
        Ok(retry) => retry,
        Err(_) => panic!("the rejected green switch must retain detached retry authority"),
    };
    assert_eq!(
        world.session.active_theme_binding(surface),
        Some(&predecessor_theme)
    );
    assert_eq!(
        world
            .session
            .mounted
            .current_unpublished_appearance()
            .unwrap(),
        predecessor_paint.as_ref()
    );
    assert_eq!(
        world
            .session
            .focus
            .as_ref()
            .unwrap()
            .current_semantic_focus(),
        predecessor_focus
    );
    super::super::reconstruction::reconstruct_surface(&mut world);
    super::product_text::assert_current_ab(&world);
    world.host.push_native_display_settled_without_effects();
    assert!(matches!(
        UiThemeSwitchOutcome::from(
            retry
                .rebase_content_and_retry(&mut world.session, 441)
                .unwrap()
        ),
        UiThemeSwitchOutcome::Published(_)
    ));
    let output = world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    super::super::palette::assert_green_content(output, world.instances[1], 40_000);
    super::product_text::assert_current_ab(&world);
    super::super::hostile_protocol::reject_stale_duplicate_and_foreign_bases(&mut world, 3);

    let predecessor = world.session.active_theme_binding(surface).unwrap().clone();
    let plan = super::switch_plan(&mut world, "theme.integrated.overlay", 7);
    world
        .host
        .push_presentation(UiHostSurfacePresentationOutcome::PresentationIndeterminate);
    let host = world.host.clone();
    super::lifecycle::recover_predecessor(
        UiThemeSwitchOutcome::from(
            world
                .session
                .prepare_rebind(plan, UiRebindExecutionRequest::new(710))
                .unwrap()
                .execute(710),
        ),
        &host,
    );
    assert_eq!(
        world.session.active_theme_binding(surface),
        Some(&predecessor)
    );
    super::shutdown::clean(world, 0);
}
