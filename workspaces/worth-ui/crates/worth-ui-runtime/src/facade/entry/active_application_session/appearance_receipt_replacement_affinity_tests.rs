use super::*;

#[test]
fn stale_predecessor_typed_values_deny_replacement_before_publication() {
    let role = single_aspect_role(
        "test.receipt-stale-typed-replacement",
        worth_ui_dsl::UiAppearanceAspect::Background,
        support::APPEARANCE_TOKEN,
    );
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(worth_ui_host_native::staged_appearance_capability_report());
    let mut session = support::legacy_static_paint_appearance_component_builder(&role)
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
        .expect("stale typed replacement fixture should prepare")
        .launch()
        .expect("stale typed replacement fixture should launch");
    let surface = session.create_semantic_surface().unwrap();
    update_theme(&mut session, "#405060");
    let predecessor = session.active_generation_identity().clone();
    let predecessor_value = resolved_color(&session, &role, surface);
    let current_capability = session
        .presentation
        .active_appearance_theme_binding(surface)
        .unwrap()
        .capability()
        .clone();
    let stale_current_binding =
        issue_capability(&session, &role, surface, "theme.appearance.surface-a");
    assert_ne!(stale_current_binding, current_capability);
    let stale_capability = stale_current_binding.clone();
    session
        .presentation
        .replace_appearance_theme_binding_for_test(stale_current_binding);

    let mut prepared = session
        .prepare_replacement(
            appearance_component_session_test_support::two_node_appearance_candidate_submission(
                &session,
                "appearance-stale-typed-successor",
                &role,
                appearance_component_session_test_support::APPEARANCE_NODE_B,
            ),
        )
        .expect("stale typed successor should prepare");
    let _catalog = active_application_session_test_support::admit_candidate_catalog(&mut prepared);
    let lowered = session
        .lower_prepared_replacement(*prepared)
        .expect("stale typed successor should lower");
    let successor_prepared = lowered.summary().candidate_generation().clone();
    let successor = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
        session.session_identity(),
        &successor_prepared,
    );
    let themes = session
        .capabilities()
        .appearance_themes()
        .expect("stale typed fixture has frozen themes");
    let profile = worth_ui_host_native::staged_appearance_capability_report()
        .appearance_profile()
        .cloned()
        .expect("native staged profile is available");
    let admission =
        crate::runtime::appearance::UiThemeCapabilityAdmission::from_frozen_capabilities(
            themes,
            themes.initial_definition_identity(),
            session.capabilities().appearance_roles(),
            &profile,
        )
        .unwrap()
        .prepare([role.role().clone()], successor.clone())
        .unwrap();
    let bindings = session
        .presentation
        .appearance_theme_state()
        .unwrap()
        .active_bindings()
        .cloned()
        .collect::<Vec<_>>();
    let rebinding = crate::runtime::appearance::prepare_theme_generation_rebinding(
        themes,
        session.capabilities().appearance_roles(),
        &profile,
        &predecessor,
        successor.clone(),
        &[role.role().clone()],
        bindings.iter(),
    )
    .unwrap();
    let denial = match session
        .presentation
        .prepare_appearance_replacement_succession(
            &predecessor,
            &successor,
            session.appearance_theme_admission.as_ref(),
            Some(admission),
            Some(&rebinding),
            Some(themes),
        ) {
        Ok(_) => panic!("stale typed values must deny before replacement publication"),
        Err(denial) => denial,
    };

    assert_eq!(
        denial,
        crate::runtime::presentation_state::UiAppearanceGenerationSuccessionDenial::
            StaleTypedValues(surface)
    );
    assert_eq!(session.active_generation_identity(), predecessor);
    assert_eq!(
        session
            .presentation
            .active_appearance_theme_binding(surface)
            .unwrap()
            .capability(),
        &stale_capability
    );
    session
        .presentation
        .replace_appearance_theme_binding_for_test(current_capability);
    assert_eq!(resolved_color(&session, &role, surface), predecessor_value);
    let _ = session.shutdown();
}
