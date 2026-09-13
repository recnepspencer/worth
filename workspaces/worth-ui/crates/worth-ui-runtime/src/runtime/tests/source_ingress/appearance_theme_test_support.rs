pub(crate) fn activate(
    application: crate::facade::entry::WorthUiHostNeutralApp,
) -> crate::facade::WorthUiApp {
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(worth_ui_host_native::appearance_capability_report());
    crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
        application,
        host,
    )
}

pub(crate) fn bundle() -> crate::capability::FrozenAppearanceThemeCapabilities {
    let token = crate::capability::ThemeTokenId::new(
        crate::runtime::tests::appearance_component_session_test_support::APPEARANCE_TOKEN,
    )
    .expect("appearance test token identity");
    let catalog = crate::capability::UiThemeSlotCatalog::admit(
        1,
        [crate::capability::UiThemeSlotDeclaration::new(
            token.clone(),
            crate::capability::ThemeTokenFamily::surface(),
            worth_ui_dsl::UiThemeValueKind::Color,
            crate::capability::ThemeTokenSource::application(),
            crate::capability::UiThemeSlotDisclosure::Public,
            crate::capability::UiThemeSlotSuccessorCompatibility::ExactMeaning,
            None,
        )],
    )
    .expect("appearance test slot catalog");
    let definition = |identity: &str, channels| {
        crate::capability::UiThemeDefinition::admit(
            crate::capability::UiThemeDefinitionIdentity::new(identity).unwrap(),
            1,
            &catalog,
            [(
                token.clone(),
                worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels(
                    channels,
                )),
            )],
        )
        .expect("appearance test theme definition")
    };
    let definitions = vec![
        definition("theme.appearance.source", [17, 34, 51, 255]),
        definition("theme.appearance.color-405060", [64, 80, 96, 255]),
        definition("theme.appearance.color-708090", [112, 128, 144, 255]),
        definition("theme.appearance.color-8090a0", [128, 144, 160, 255]),
        definition("theme.appearance.color-90a0b0", [144, 160, 176, 255]),
    ];
    crate::capability::FrozenAppearanceThemeCapabilities::admit(
        catalog,
        crate::capability::UiThemeDefinitionIdentity::new("theme.appearance.source").unwrap(),
        definitions,
    )
    .expect("appearance test theme bundle")
}
