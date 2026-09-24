use super::{mounting_fixture, support, test_support, theme_session};
use crate::runtime::appearance::{
    UiThemeSwitchOrigin, UiThemeSwitchOriginFamily, UiThemeSwitchRequest,
};
use crate::runtime::rebind::{UiRebindExecutionPolicy, UiRebindExecutionRequest, UiRebindOutcome};

#[test]
fn live_theme_switch_rejects_without_changing_paint_then_retries_only_its_surface() {
    let role = support::validation_background_role_with_axis(
        support::APPEARANCE_TOKEN,
        worth_ui_dsl::UiAppearanceStateAxis::Validation,
    );
    let (mut session, host) = theme_session(&role);
    let (surface, graph_node) = mounting_fixture::mount(&mut session, 1_000);
    let neighbor = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            neighbor,
            crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
            crate::facade::mounted::UiSurfaceBindingProfile::new(
                1_000,
                crate::facade::mounted::UiSurfaceBindingCoordinatePosture::LogicalPoints,
                1,
            )
            .unwrap(),
        )
        .unwrap();
    let graph = session.mounted_graph_node(graph_node).unwrap();
    session.mount_instance(graph, neighbor).unwrap();
    crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(
        &mut session, neighbor);
    let origin = close_origin(&mut session, &role, "theme-live-initial");
    session.advance_mounted_identity_frame().unwrap();
    host.push_native_display_presented();
    let outcome = session
        .execute_mounted_frame(
            session.mounted_frame_request(),
            worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
            1,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("initial public frame prepares"));
    assert!(matches!(
        outcome,
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
    drop(outcome);
    let predecessor = session.mounted.current_publication().unwrap().clone();
    let predecessor_binding = session.active_theme_binding(surface).unwrap().clone();
    let neighbor_binding = session.active_theme_binding(neighbor).unwrap().clone();
    let owners = session.appearance_owner_snapshot.as_ref().unwrap().clone();
    let replay_origin = origin.clone();
    let plan = switch_plan(
        &mut session,
        origin,
        surface,
        "theme.appearance.production-405060",
    );
    host.push_rejected();
    let prepared = session
        .prepare_rebind(plan, UiRebindExecutionRequest::new(2))
        .unwrap();
    assert_eq!(
        prepared
            .prepared_frame()
            .unwrap()
            .appearance_selection_cost_report()
            .selected_instance_count(),
        1,
        "one concrete occurrence on the switched surface"
    );
    let UiRebindOutcome::RejectedBeforeEffects(rejected) = prepared.execute(2) else {
        panic!("scripted theme rejection must preserve predecessor");
    };
    let retry = rejected
        .detach_retry_for_native()
        .unwrap_or_else(|_| panic!("theme retry detaches"));
    assert_eq!(session.mounted.current_publication(), Some(&predecessor));
    assert_eq!(
        session.active_theme_binding(surface),
        Some(&predecessor_binding)
    );
    let current_owners = session.appearance_owner_snapshot.as_ref().unwrap();
    assert_eq!(current_owners.turn(), owners.turn());
    assert_eq!(current_owners.generation(), owners.generation());
    assert_eq!(current_owners.validation(), owners.validation());
    host.push_native_display_settled_without_effects();
    assert!(matches!(
        retry.rebase_content_and_retry(&mut session, 3).unwrap(),
        UiRebindOutcome::Published(_)
    ));
    assert_eq!(
        session
            .active_theme_binding(surface)
            .unwrap()
            .binding_generation(),
        predecessor_binding.binding_generation() + 1
    );
    assert_eq!(
        session.active_theme_binding(neighbor),
        Some(&neighbor_binding)
    );
    let replay_capability = session
        .active_theme_binding(surface)
        .unwrap()
        .capability()
        .clone();
    let replay_generation = session
        .active_theme_binding(surface)
        .unwrap()
        .binding_generation();
    assert!(matches!(
        session.prepare_theme_switch(UiThemeSwitchRequest::new(
            replay_origin,
            surface,
            replay_generation,
            replay_capability
        )),
        Err(
            crate::facade::appearance::UiThemeSwitchPreparationDenial::Admission(
                crate::runtime::appearance::UiThemeSwitchDenial::DuplicateOrigin
            )
        )
    ));
    let output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    test_support::assert_unpublished_surface(output, [64, 80, 96, 255]);
    assert_eq!(
        output.fragments()[0].work().successor().semantic_surface(),
        surface
    );
    assert_eq!(
        session
            .presentation
            .appearance_theme_state()
            .unwrap()
            .prepared_switch_count(),
        0
    );

    let origin = close_origin(&mut session, &role, "theme-live-recovery");
    let plan = switch_plan(&mut session, origin, surface, "theme.appearance.production");
    host.push_native_display_settled_without_effects();
    assert!(matches!(
        session
            .prepare_rebind(plan, UiRebindExecutionRequest::new(4))
            .unwrap()
            .execute(4),
        UiRebindOutcome::Published(_)
    ));
    test_support::assert_unpublished_surface(
        session
            .mounted
            .current_unpublished_appearance()
            .unwrap()
            .unwrap(),
        [17, 34, 51, 255],
    );
    assert_eq!(
        session.active_theme_binding(neighbor),
        Some(&neighbor_binding)
    );
    assert_eq!(
        session
            .presentation
            .appearance_theme_state()
            .unwrap()
            .prepared_switch_count(),
        0
    );
    let _ = session.shutdown();
}

fn close_origin(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    name: &str,
) -> UiThemeSwitchOrigin {
    let source = support::appearance_candidate_submission(session, name, Some(role));
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(source).unwrap();
    let admitted = turn.seal().unwrap();
    let origin = session
        .issue_theme_switch_origin(&admitted, UiThemeSwitchOriginFamily::SourceEditObservation)
        .unwrap();
    session.classify_observations(admitted).unwrap();
    origin
}

fn switch_plan(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    origin: UiThemeSwitchOrigin,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    definition: &str,
) -> crate::runtime::rebind::UiRebindPlan {
    let capability = session
        .admit_appearance_theme(
            surface,
            &crate::capability::UiThemeDefinitionIdentity::new(definition).unwrap(),
        )
        .unwrap();
    let expected = session
        .active_theme_binding(surface)
        .unwrap()
        .binding_generation();
    let crate::runtime::observation::UiChangeClassificationOutcome::Changed(change) = session
        .prepare_theme_switch(UiThemeSwitchRequest::new(
            origin, surface, expected, capability,
        ))
        .unwrap()
    else {
        panic!("a different admitted definition is a classified binding change");
    };
    let scope = session.resolve_affected_scope(change).unwrap();
    session
        .compile_rebind_plan(
            scope.resolve_identity_lifecycle().unwrap(),
            UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap()
}
