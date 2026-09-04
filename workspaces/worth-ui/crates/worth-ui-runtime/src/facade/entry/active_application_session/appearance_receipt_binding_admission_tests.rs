use super::*;

#[test]
fn initial_binding_uses_explicit_default_and_current_surface_authority() {
    let role = single_aspect_role(
        "test.receipt-initial-binding",
        worth_ui_dsl::UiAppearanceAspect::Background,
        support::APPEARANCE_TOKEN,
    );
    let mut session = theme_capable_unbound_session(&role);
    let surface = session.create_semantic_surface().unwrap();
    let binding = session
        .presentation
        .active_appearance_theme_binding(surface)
        .expect("surface creation materializes the prepared admission");
    let expected_profile = worth_ui_host_native::staged_appearance_capability_report()
        .appearance_profile()
        .cloned()
        .expect("staged host reports its appearance profile");
    let expected_definition = theme_definition(
        session.capabilities().appearance_themes().unwrap(),
        "theme.appearance.receipts",
    )
    .identity()
    .clone();

    assert_eq!(binding.binding_generation(), 1);
    assert_eq!(binding.capability().surface(), surface);
    assert_eq!(
        binding.capability().application(),
        &session.active_generation_identity()
    );
    assert_eq!(binding.capability().definition(), &expected_definition);
    assert_eq!(binding.capability().host_profile(), &expected_profile);
    assert_eq!(binding.capability().required_roles().len(), 1);
    assert_eq!(
        binding.capability().required_roles()[0].identity(),
        role.role()
    );
    assert_eq!(
        binding.capability().required_roles()[0].revision(),
        role.revision()
    );
    let _ = session.shutdown();
}

#[test]
fn mounted_allocation_carries_initial_binding_to_the_successor_generation() {
    let role = single_aspect_role(
        "test.receipt-allocation-succession",
        worth_ui_dsl::UiAppearanceAspect::Background,
        support::APPEARANCE_TOKEN,
    );
    let fixture = super::super::mounted_fixture(&role, &[], false);
    let successor = fixture.session.active_generation_identity();
    let binding = fixture
        .session
        .presentation
        .active_appearance_theme_binding(fixture.surface)
        .expect("allocation succession retains the surface binding");

    assert_ne!(successor, fixture.generation_before_allocation);
    assert_eq!(
        binding.binding_generation(),
        fixture.binding_generation_before_allocation
    );
    assert_eq!(binding.capability().application(), &successor);
    assert_eq!(
        binding.capability().definition().as_str(),
        "theme.appearance.receipts"
    );
    assert!(fixture
        .session
        .presentation
        .appearance_theme_resolution_view(
            fixture.session.capabilities(),
            &role,
            fixture.surface,
            &successor,
        )
        .is_ok());
    let _ = fixture.session.shutdown();
}

#[test]
fn missing_host_appearance_profile_denies_before_surface_registration() {
    let role = single_aspect_role(
        "test.receipt-missing-host-profile",
        worth_ui_dsl::UiAppearanceAspect::Background,
        support::APPEARANCE_TOKEN,
    );
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    let application = theme_capable_application(&role, host.clone());
    let denial = match application.launch() {
        Ok(_) => panic!("missing host appearance profile must deny before activation"),
        Err(denial) => denial,
    };

    assert_eq!(
        denial,
        crate::runtime::WorthUiRuntimeLaunchDenial::AppearanceThemeAdmission(
            crate::runtime::appearance::UiThemeCapabilityReceiptDenial::MissingHostProfile,
        )
    );
    assert_eq!(host.native_registration_count(), 0);
}
