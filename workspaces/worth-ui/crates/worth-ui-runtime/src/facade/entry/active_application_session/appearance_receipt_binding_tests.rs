use super::role_support::{
    single_aspect_role, staged_test_host_profile, theme_bundle, theme_definition, update_theme,
};
use super::support;
use crate::runtime::tests::{
    active_application_session_test_support, appearance_component_session_test_support,
};

#[path = "appearance_receipt_binding_admission_tests.rs"]
mod admission_tests;

#[test]
fn application_replacement_rebinds_selected_definition_to_successor_generation() {
    let mut session =
        appearance_component_session_test_support::source_backed_static_paint_consumer_session();
    let role = appearance_component_session_test_support::validation_background_role(
        support::APPEARANCE_TOKEN,
    );
    let surface = session.create_semantic_surface().unwrap();
    let predecessor = session.active_generation_identity().clone();
    let before = session
        .presentation
        .active_appearance_theme_binding(surface)
        .expect("appearance surface starts with a current binding")
        .clone();
    let mut prepared = session
        .prepare_replacement(
            appearance_component_session_test_support::attached_appearance_candidate_submission(
                &session,
                "appearance-binding-successor",
                appearance_component_session_test_support::APPEARANCE_NODE_B,
            ),
        )
        .expect("appearance successor should prepare");
    let catalog = active_application_session_test_support::admit_candidate_catalog(&mut prepared);
    let lowered = session
        .lower_prepared_replacement(*prepared)
        .expect("appearance successor should lower");
    let pending = session
        .stage_prepared_replacement(lowered)
        .expect("appearance successor should stage");
    let boundary = session
        .execute_framework_turn(|_| {})
        .expect("no mounted presentation lease is active")
        .into_completion()
        .into_execution()
        .expect("empty framework turn should yield an activation boundary")
        .into_activation_boundary();
    let receipt = session
        .activate_prepared_replacement(pending, catalog, boundary, None)
        .expect("appearance successor should activate");
    let _receipt = receipt
        .into_activation()
        .expect("changed appearance consumer graph should publish");
    let successor = session.active_generation_identity().clone();
    let after = session
        .presentation
        .active_appearance_theme_binding(surface)
        .expect("successor retains the appearance surface binding");

    assert_ne!(successor, predecessor);
    assert_eq!(after.binding_generation(), before.binding_generation() + 1);
    assert_eq!(after.capability().surface(), surface);
    assert_eq!(after.capability().application(), &successor);
    assert_eq!(
        after.capability().definition(),
        before.capability().definition()
    );
    assert_eq!(
        after.capability().required_roles(),
        before.capability().required_roles()
    );
    assert_eq!(
        after.capability().host_profile(),
        before.capability().host_profile()
    );
    assert!(session
        .presentation
        .appearance_theme_resolution_view(session.capabilities(), &role, surface, &successor,)
        .is_ok());
    let _ = session.shutdown();
}

#[test]
fn evidence_only_rebind_materializes_first_appearance_binding() {
    let role = support::validation_background_role(support::APPEARANCE_TOKEN);
    let mut session =
        appearance_component_session_test_support::source_backed_static_paint_consumer_session();
    assert!(session
        .presentation
        .appearance_theme_state()
        .is_none_or(|state| state.active_bindings().next().is_none()));
    let predecessor = session.active_generation_identity().clone();
    let candidate = appearance_component_session_test_support::appearance_candidate_submission(
        &session,
        "appearance-zero-surface-evidence-successor",
        Some(&role),
    );
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let admitted = turn.seal().unwrap();
    let evidence = match session.classify_observations(admitted).unwrap() {
        crate::runtime::observation::UiChangeClassificationOutcome::EvidenceOnly(evidence) => {
            evidence
        }
        _ => panic!("equal appearance semantics must produce evidence-only succession"),
    };
    let plan = session
        .compile_preservation_rebind(
            evidence,
            crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap();
    let successor_prepared = plan.basis().candidate_generation().clone();
    let successor = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
        session.session_identity(),
        &successor_prepared,
    );
    let prepared = session
        .prepare_rebind(
            plan,
            crate::runtime::rebind::UiRebindExecutionRequest::new(1),
        )
        .unwrap();
    let receipt = match prepared.execute(1) {
        crate::runtime::rebind::UiRebindOutcome::Published(receipt) => receipt,
        _ => panic!("evidence-only succession must publish its prepared authority"),
    };
    assert_eq!(receipt.active_generation(), &successor_prepared);
    assert_eq!(session.active_generation_identity(), successor);

    let surface = session.create_semantic_surface().unwrap();
    let binding = session
        .presentation
        .active_appearance_theme_binding(surface)
        .expect("first surface creation materializes the carried admission");
    let expected_profile = worth_ui_host_native::staged_appearance_capability_report()
        .appearance_profile()
        .cloned()
        .expect("staged native profile is retained");
    let expected_definition = session
        .capabilities()
        .appearance_themes()
        .expect("appearance-capable session retains its theme bundle")
        .initial_definition_identity()
        .as_str()
        .to_owned();
    assert_eq!(binding.binding_generation(), 1);
    assert_eq!(binding.capability().application(), &successor);
    assert_eq!(
        binding.capability().definition().as_str(),
        expected_definition
    );
    assert_eq!(binding.capability().required_roles().len(), 1);
    assert_eq!(
        binding.capability().required_roles()[0].identity(),
        role.role()
    );
    assert_eq!(binding.capability().host_profile(), &expected_profile);
    assert_ne!(successor, predecessor);
    drop(receipt);
    let _ = session.shutdown();
}

#[test]
fn appearance_resolution_denies_missing_active_theme_binding() {
    let role = single_aspect_role(
        "test.receipt-missing-binding",
        worth_ui_dsl::UiAppearanceAspect::Background,
        support::APPEARANCE_TOKEN,
    );
    let mut session = theme_capable_unbound_session(&role);
    let surface = session.create_semantic_surface().unwrap();
    assert!(session
        .presentation
        .remove_appearance_theme_binding_for_test(surface));
    let denial = session
        .presentation
        .appearance_theme_resolution_view(
            session.capabilities(),
            &role,
            surface,
            &session.active_generation_identity(),
        )
        .unwrap_err();

    assert_eq!(
        denial,
        crate::runtime::presentation_state::UiAppearanceThemeBindingDenial::MissingActiveBinding
    );
    let _ = session.shutdown();
}

#[test]
fn appearance_resolution_denies_typed_values_from_stale_binding_authority() {
    let role = single_aspect_role(
        "test.receipt-stale-binding",
        worth_ui_dsl::UiAppearanceAspect::Background,
        support::APPEARANCE_TOKEN,
    );
    let mut fixture = super::mounted_fixture(&role, &[], false);
    update_theme(&mut fixture.session, "#405060");
    super::publish_frame(&mut fixture.session, 1);

    let successor = issue_capability(&fixture.session, &role, fixture.surface, "receipt-new", 2);
    fixture
        .session
        .presentation
        .replace_appearance_theme_binding_for_test(successor);
    let denial = fixture
        .session
        .presentation
        .appearance_theme_resolution_view(
            fixture.session.capabilities(),
            &role,
            fixture.surface,
            &fixture.session.active_generation_identity(),
        )
        .unwrap_err();

    assert_eq!(
        denial,
        crate::runtime::presentation_state::UiAppearanceThemeBindingDenial::StaleTypedValues
    );
    super::query_support::shutdown(fixture.session);
}

#[test]
fn materializing_later_surface_preserves_existing_typed_theme_values() {
    let role = single_aspect_role(
        "test.receipt-later-surface",
        worth_ui_dsl::UiAppearanceAspect::Background,
        support::APPEARANCE_TOKEN,
    );
    let mut session = theme_capable_unbound_session(&role);
    let first = session.create_semantic_surface().unwrap();
    update_theme(&mut session, "#405060");
    let second = session.create_semantic_surface().unwrap();
    assert_ne!(first, second);

    let view = session
        .presentation
        .appearance_theme_resolution_view(
            session.capabilities(),
            &role,
            first,
            &session.active_generation_identity(),
        )
        .unwrap();
    let resolved = view
        .resolve(
            &worth_ui_dsl::UiThemeSlotIdentity::new(support::APPEARANCE_TOKEN).unwrap(),
            worth_ui_dsl::UiThemeValueKind::Color,
        )
        .unwrap();
    assert_eq!(
        resolved.value(),
        worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
            64, 80, 96, 255,
        ]))
    );
    let _ = session.shutdown();
}

fn theme_capable_unbound_session(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> crate::facade::WorthUiActiveApplicationSession {
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(worth_ui_host_native::staged_appearance_capability_report());
    theme_capable_application(role, host)
        .launch()
        .expect("theme-capable fixture should launch")
}

fn theme_capable_application(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    host: crate::certification_support::ScriptedPresentationHost,
) -> crate::facade::WorthUiApp {
    support::legacy_static_paint_appearance_component_builder(role)
        .register_appearance_theme_bundle(theme_bundle(&[], None))
        .unwrap()
        .with_rust_authored_declaration_fixture(support::appearance_fixture(role))
        .freeze()
        .map(|application| {
            crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                application,
                host,
            )
        })
        .expect("theme-capable fixture should prepare")
}

fn issue_capability(
    session: &crate::facade::WorthUiActiveApplicationSession,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    host_identity: &str,
    host_version: u16,
) -> crate::runtime::appearance::UiThemeCapabilityReceipt {
    issue_capability_for_definition(
        session,
        role,
        surface,
        "theme.appearance.receipts",
        host_identity,
        host_version,
    )
}

fn issue_capability_for_definition(
    session: &crate::facade::WorthUiActiveApplicationSession,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    definition_identity: &str,
    host_identity: &str,
    host_version: u16,
) -> crate::runtime::appearance::UiThemeCapabilityReceipt {
    let themes = session
        .capabilities()
        .appearance_themes()
        .expect("binding fixture has frozen appearance themes");
    let definition = theme_definition(themes, definition_identity);
    let profile = staged_test_host_profile(host_identity, host_version);
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
