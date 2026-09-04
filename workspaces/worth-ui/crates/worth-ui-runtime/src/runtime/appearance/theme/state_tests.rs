use super::*;

#[test]
fn switching_enforces_surface_application_and_predecessor_cas() {
    let surface = next_surface();
    let other_surface = next_surface();
    let (origin, application) = admitted_source_origin("theme-origin-cas");
    let mut state = UiAppearanceThemeState::default();
    state
        .install_initial(capability("theme.initial", surface, application.clone()))
        .unwrap();
    state
        .install_initial(capability(
            "theme.other-initial",
            other_surface,
            application.clone(),
        ))
        .unwrap();

    assert_eq!(
        state.prepare_theme_switch(request(
            origin.clone(),
            surface,
            2,
            capability("theme.stale", surface, application.clone()),
        )),
        Err(UiThemeSwitchDenial::StaleBinding)
    );
    assert_eq!(
        state.prepare_theme_switch(UiThemeSwitchRequest::new(
            origin.clone(),
            other_surface,
            1,
            capability("theme.wrong-surface", surface, application.clone()),
        )),
        Err(UiThemeSwitchDenial::WrongSurfaceCapability)
    );
    assert_eq!(
        state.prepare_theme_switch(request(
            origin.clone(),
            surface,
            1,
            capability("theme.foreign-app", surface, generation(42)),
        )),
        Err(UiThemeSwitchDenial::WrongOriginSession)
    );

    let first = state
        .prepare_theme_switch(request(
            origin.clone(),
            surface,
            1,
            capability("theme.first", surface, application.clone()),
        ))
        .unwrap();
    let stale = state
        .prepare_theme_switch(request(
            origin.clone(),
            surface,
            1,
            capability("theme.second", surface, application.clone()),
        ))
        .unwrap();
    let unrelated = state
        .prepare_theme_switch(request(
            origin,
            other_surface,
            1,
            capability("theme.other-next", other_surface, application),
        ))
        .unwrap();
    state.commit_published_switch(first).unwrap();
    assert_eq!(state.prepared_switch_count(), 1);
    assert_eq!(
        state.commit_published_switch(stale),
        Err(UiThemeSwitchDenial::UnknownPreparedSwitch)
    );
    state.cancel_prepared_switch(unrelated).unwrap();
    assert_eq!(state.prepared_switch_count(), 0);
}

#[test]
fn prepared_switches_are_affine_bounded_and_cancellable() {
    let surface = next_surface();
    let (origin, application) = admitted_source_origin("theme-origin-bounded");
    let mut state = UiAppearanceThemeState::default();
    state
        .install_initial(capability("theme.initial", surface, application.clone()))
        .unwrap();
    assert_eq!(
        state.install_initial(capability("theme.duplicate", surface, application.clone())),
        Err(UiThemeInitialBindingDenial::SurfaceAlreadyBound)
    );

    let mut prepared = Vec::new();
    for index in 0..4 {
        prepared.push(
            state
                .prepare_theme_switch(request(
                    origin.clone(),
                    surface,
                    1,
                    capability(&format!("theme.{index}"), surface, application.clone()),
                ))
                .unwrap(),
        );
    }
    assert_eq!(state.prepared_switch_count(), 4);
    assert_eq!(
        state.prepare_theme_switch(request(
            origin.clone(),
            surface,
            1,
            capability("theme.overflow", surface, application.clone()),
        )),
        Err(UiThemeSwitchDenial::PreparedSwitchCapacityExceeded)
    );
    let cancelled = prepared.pop().unwrap();
    let replay = duplicate_prepared(&cancelled);
    state.cancel_prepared_switch(cancelled).unwrap();
    assert_eq!(
        state.cancel_prepared_switch(replay),
        Err(UiThemeSwitchDenial::UnknownPreparedSwitch)
    );
    assert_eq!(state.prepared_switch_count(), 3);
    assert!(state
        .prepare_theme_switch(request(
            origin.clone(),
            surface,
            1,
            capability("theme.replacement", surface, application),
        ))
        .is_ok());
}

#[test]
fn prepared_switch_cannot_cross_same_application_owner_with_colliding_reservation() {
    let surface = next_surface();
    let mut first = UiAppearanceThemeState::default();
    let mut second = UiAppearanceThemeState::default();
    let (origin, first_application) = admitted_source_origin("theme-origin-owner-affinity");
    let second_application = first_application.clone();
    first
        .install_initial(capability(
            "theme.first",
            surface,
            first_application.clone(),
        ))
        .unwrap();
    second
        .install_initial(capability(
            "theme.second",
            surface,
            second_application.clone(),
        ))
        .unwrap();
    let foreign = first
        .prepare_theme_switch(request(
            origin.clone(),
            surface,
            1,
            capability("theme.first-next", surface, first_application),
        ))
        .unwrap();
    let local = second
        .prepare_theme_switch(request(
            origin,
            surface,
            1,
            capability("theme.second-next", surface, second_application),
        ))
        .unwrap();
    assert_eq!(
        second.commit_published_switch(foreign),
        Err(UiThemeSwitchDenial::UnknownPreparedSwitch)
    );
    assert_eq!(second.prepared_switch_count(), 1);
    second.commit_published_switch(local).unwrap();
}

fn duplicate_prepared(prepared: &UiPreparedThemeSwitch) -> UiPreparedThemeSwitch {
    UiPreparedThemeSwitch {
        reservation: prepared.reservation,
        predecessor_generation: prepared.predecessor_generation,
        successor: prepared.successor.clone(),
        origin: prepared.origin.clone(),
        owner_affinity: prepared.owner_affinity,
    }
}

fn request(
    origin: UiThemeSwitchOrigin,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    expected_generation: u64,
    capability: UiThemeCapabilityReceipt,
) -> UiThemeSwitchRequest {
    UiThemeSwitchRequest::new(origin, surface, expected_generation, capability)
}

fn admitted_source_origin(
    source_name: &str,
) -> (
    UiThemeSwitchOrigin,
    crate::runtime::WorthUiActiveApplicationGenerationIdentity,
) {
    use crate::runtime::tests::active_application_session_test_support::component_candidate_submission;
    use crate::runtime::tests::appearance_component_session_test_support::source_backed_static_paint_consumer_session;

    let mut session = source_backed_static_paint_consumer_session();
    let application = session.active_generation_identity();
    let candidate = component_candidate_submission(
        &session,
        source_name,
        "workspace.component.active_session_current",
    );
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let admitted = turn.seal().unwrap();
    let origin = session
        .issue_theme_switch_origin(&admitted, UiThemeSwitchOriginFamily::SourceEditObservation)
        .unwrap();
    let _ = session.shutdown();
    (origin, application)
}

fn capability(
    name: &str,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    application: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
) -> UiThemeCapabilityReceipt {
    let slot = crate::capability::UiThemeSlotDeclaration::new(
        crate::capability::ThemeTokenId::new("surface.base").unwrap(),
        crate::capability::ThemeTokenFamily::surface(),
        worth_ui_dsl::UiThemeValueKind::Color,
        crate::capability::ThemeTokenSource::application(),
        crate::capability::UiThemeSlotDisclosure::Public,
        crate::capability::UiThemeSlotSuccessorCompatibility::ExactMeaning,
        None,
    );
    let catalog = crate::capability::UiThemeSlotCatalog::admit(1, [slot]).unwrap();
    let definition_identity = crate::capability::UiThemeDefinitionIdentity::new(name).unwrap();
    let definition = crate::capability::UiThemeDefinition::admit(
        definition_identity.clone(),
        1,
        &catalog,
        [(
            crate::capability::ThemeTokenId::new("surface.base").unwrap(),
            worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
                1, 2, 3, 255,
            ])),
        )],
    )
    .unwrap();
    let contract = worth_ui_dsl::UiAppearanceAspectContract::component(
        [worth_ui_dsl::UiAppearanceAspect::Background],
        [],
    )
    .unwrap();
    let role_identity = worth_ui_dsl::UiAppearanceRoleIdentity::new("theme.test-role").unwrap();
    let role = worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
        role_identity.clone(),
        worth_ui_dsl::UiAppearanceRoleRevision::new(1).unwrap(),
        worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent,
        &contract,
        [(
            worth_ui_dsl::UiAppearanceAspect::Background,
            worth_ui_dsl::UiAppearancePartitionAuthoring::new([])
                .with_cell(worth_ui_dsl::UiAppearanceCell::when([]).uses_slot(
                    worth_ui_dsl::UiThemeSlotIdentity::new("surface.base").unwrap(),
                    worth_ui_dsl::UiThemeValueKind::Color,
                ))
                .compile(worth_ui_dsl::UiAppearanceAspect::Background)
                .unwrap(),
        )],
    )
    .unwrap();
    let bundle = crate::capability::FrozenAppearanceThemeCapabilities::admit(
        catalog,
        definition_identity.clone(),
        vec![definition],
    )
    .unwrap();
    let registered = crate::facade::entry::CapabilityRegistrationBuilder::new()
        .register_appearance_role(role)
        .unwrap()
        .register_appearance_theme_bundle(bundle)
        .unwrap()
        .freeze_with_registration_report()
        .into_accepted_snapshot();
    assert_eq!(
        registered
            .freeze_report()
            .registry_family_width(crate::capability::RegistryFamily::AppearanceTheme),
        Some(1)
    );
    assert!(registered
        .freeze_report()
        .has_complete_registry_family_inventory());
    let host_profile = worth_ui_host_contract::UiHostAppearanceProfileContract::admit(
        "test-host",
        1,
        [
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::SurfaceFill,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::SurfaceBorder,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::CornerRadii,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::Outline,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::TextRangeForeground,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::PortalSurface,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::Backdrop,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::OverlayOrder,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::PointerAffordance,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::Damage,
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::Clip,
        ],
        Some(worth_ui_host_contract::UiHostPrimaryPointerKind::Mouse),
        worth_ui_host_contract::UiHostAppearanceGeometryQualification::admit([
            worth_ui_host_contract::UiHostAppearanceScaleGeometryQualification::new(
                1_000,
                1,
                worth_ui_host_contract::UiAppearanceLogicalLength::new(1_000).unwrap(),
                worth_ui_host_contract::UiHostAppearanceGeometryQualificationBasis::AnalyticSignedDistancePixelCenter,
            ),
        ])
        .expect("the test geometry qualification must admit"),
    )
    .unwrap();
    UiThemeCapabilityAdmission::from_frozen_capabilities(
        registered.appearance_themes().unwrap(),
        &definition_identity,
        registered.appearance_roles(),
        &host_profile,
    )
    .unwrap()
    .issue([role_identity], surface, application)
    .unwrap()
}

fn next_surface() -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
    worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap()
}

fn generation(seed: u64) -> crate::runtime::WorthUiActiveApplicationGenerationIdentity {
    use worth_ui_dsl::{
        UiDslSemanticArtifactSpec, UiDslSemanticFamily, UiDslSemanticKey, UiDslSourceProvenance,
        UiDslStructuralToken,
    };
    let app = crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .with_rust_authored_declaration_fixture(
            crate::facade::WorthUiRustAuthoredDeclarationFixture::named(format!(
                "appearance-theme-fixture-{seed}"
            ))
            .with_semantic_artifact_spec(
                UiDslSemanticArtifactSpec::new(
                    UiDslSemanticKey::new(format!("ui.appearance.theme.{seed}")),
                    UiDslSemanticFamily::Control,
                    UiDslSourceProvenance::rust_authored("appearance/theme", 0),
                )
                .with_structural_token(UiDslStructuralToken::new("control:appearance-theme")),
            ),
        )
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .unwrap();
    crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
        crate::lifecycle::WorthUiActiveApplicationSessionIdentity::from_host_session_value(seed),
        app.generation_identity(),
    )
}
