use super::role_support::{
    multi_definition_theme_bundle, single_aspect_role, theme_bundle, theme_definition, update_theme,
};
use super::support;
use crate::runtime::tests::{
    active_application_session_test_support, appearance_component_session_test_support,
};

#[test]
fn application_replacement_preserves_each_surface_definition_and_uses_successor_default() {
    let role = single_aspect_role(
        "test.receipt-per-surface-replacement",
        worth_ui_dsl::UiAppearanceAspect::Background,
        support::APPEARANCE_TOKEN,
    );
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(worth_ui_host_native::appearance_capability_report());
    let mut session = support::alternate_token_appearance_component_builder(&role)
        .register_appearance_theme_bundle(multi_definition_theme_bundle())
        .unwrap()
        .with_rust_authored_declaration_fixture(support::appearance_fixture(&role))
        .freeze()
        .map(|application| {
            crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                application,
                host,
            )
        })
        .expect("multi-definition fixture should prepare")
        .launch()
        .expect("multi-definition fixture should launch");
    let first = session.create_semantic_surface().unwrap();
    let second = session.create_semantic_surface().unwrap();
    let predecessor = session.active_generation_identity().clone();
    let definition_a = issue_capability(&session, &role, first, "theme.appearance.surface-a");
    let definition_b = issue_capability(&session, &role, second, "theme.appearance.surface-b");
    session
        .presentation
        .replace_appearance_theme_binding_for_test(definition_a);
    session
        .presentation
        .replace_appearance_theme_binding_for_test(definition_b);

    let mut prepared = session
        .prepare_replacement(
            appearance_component_session_test_support::two_node_appearance_candidate_submission(
                &session,
                "appearance-per-surface-successor",
                &role,
                appearance_component_session_test_support::APPEARANCE_NODE_B,
            ),
        )
        .expect("per-surface successor should prepare");
    let catalog = active_application_session_test_support::admit_candidate_catalog(&mut prepared);
    let lowered = session
        .lower_prepared_replacement(*prepared)
        .expect("per-surface successor should lower");
    let pending = session
        .stage_prepared_replacement(lowered)
        .expect("per-surface successor should stage");
    let boundary = session
        .execute_framework_turn(|_| {})
        .unwrap()
        .into_completion()
        .into_execution()
        .unwrap()
        .into_activation_boundary();
    let _ = session
        .activate_prepared_replacement(pending, catalog, boundary, None)
        .expect("per-surface successor should activate")
        .into_activation()
        .expect("per-surface successor should publish");
    let successor = session.active_generation_identity().clone();
    let first_after = session
        .presentation
        .active_appearance_theme_binding(first)
        .unwrap();
    let second_after = session
        .presentation
        .active_appearance_theme_binding(second)
        .unwrap();

    assert_ne!(successor, predecessor);
    assert_eq!(
        first_after.capability().definition().as_str(),
        "theme.appearance.surface-a"
    );
    assert_eq!(
        second_after.capability().definition().as_str(),
        "theme.appearance.surface-b"
    );
    assert_eq!(first_after.capability().application(), &successor);
    assert_eq!(second_after.capability().application(), &successor);
    let successor_surface = session.create_semantic_surface().unwrap();
    assert_eq!(
        session
            .presentation
            .active_appearance_theme_binding(successor_surface)
            .unwrap()
            .capability()
            .definition()
            .as_str(),
        "theme.appearance.default"
    );
    let _ = session.shutdown();
}

#[test]
fn application_replacement_preserves_updated_typed_theme_value() {
    let role = single_aspect_role(
        "test.receipt-typed-replacement",
        worth_ui_dsl::UiAppearanceAspect::Background,
        support::APPEARANCE_TOKEN,
    );
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(worth_ui_host_native::appearance_capability_report());
    let mut session = support::alternate_token_appearance_component_builder(&role)
        .register_appearance_theme_bundle(theme_bundle(&[], None))
        .unwrap()
        .with_rust_authored_declaration_fixture(support::appearance_fixture(&role))
        .freeze()
        .map(|application| {
            crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                application,
                host,
            )
        })
        .expect("typed replacement fixture should prepare")
        .launch()
        .expect("typed replacement fixture should launch");
    let surface = session.create_semantic_surface().unwrap();
    update_theme(&mut session, "#405060");
    let predecessor = session.active_generation_identity().clone();

    let mut prepared = session
        .prepare_replacement(
            appearance_component_session_test_support::two_node_appearance_candidate_submission(
                &session,
                "appearance-typed-successor",
                &role,
                appearance_component_session_test_support::APPEARANCE_NODE_B,
            ),
        )
        .expect("typed successor should prepare");
    let catalog = active_application_session_test_support::admit_candidate_catalog(&mut prepared);
    let lowered = session
        .lower_prepared_replacement(*prepared)
        .expect("typed successor should lower");
    let pending = session
        .stage_prepared_replacement(lowered)
        .expect("typed successor should stage");
    let boundary = session
        .execute_framework_turn(|_| {})
        .unwrap()
        .into_completion()
        .into_execution()
        .unwrap()
        .into_activation_boundary();
    let _ = session
        .activate_prepared_replacement(pending, catalog, boundary, None)
        .expect("typed successor should activate")
        .into_activation()
        .expect("typed successor should publish");

    let successor = session.active_generation_identity().clone();
    let binding = session
        .presentation
        .active_appearance_theme_binding(surface)
        .expect("typed successor retains the surface binding");
    let view = crate::runtime::presentation_state::UiApplicationPresentationState::resolve_appearance_theme_binding(
        session.capabilities(), &role, surface, &successor, binding)
        .expect("successor binding should resolve");
    let resolved = view
        .resolve(
            &worth_ui_dsl::UiThemeSlotIdentity::new(support::APPEARANCE_TOKEN).unwrap(),
            worth_ui_dsl::UiThemeValueKind::Color,
        )
        .expect("typed successor value should resolve");

    assert_ne!(successor, predecessor);
    assert_eq!(binding.capability().application(), &successor);
    assert_eq!(
        resolved.value(),
        worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
            64, 80, 96, 255,
        ]))
    );
    let _ = session.shutdown();
}

fn issue_capability(
    session: &crate::facade::WorthUiActiveApplicationSession,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    definition_identity: &str,
) -> crate::runtime::appearance::UiThemeCapabilityReceipt {
    let themes = session.capabilities().appearance_themes().unwrap();
    let definition = theme_definition(themes, definition_identity);
    let profile = worth_ui_host_native::appearance_capability_report()
        .appearance_profile()
        .cloned()
        .expect("the staged host fixture must report its appearance profile");
    crate::runtime::appearance::UiThemeCapabilityAdmission::from_frozen_capabilities(
        themes,
        definition.identity(),
        session.capabilities().appearance_roles(),
        &profile,
    )
    .unwrap()
    .issue(
        [role.role().clone()],
        surface,
        session.active_generation_identity(),
    )
    .unwrap()
}
