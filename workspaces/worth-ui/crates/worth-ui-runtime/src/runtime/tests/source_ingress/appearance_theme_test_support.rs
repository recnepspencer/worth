pub(crate) fn activate(
    application: crate::facade::entry::WorthUiHostNeutralApp,
) -> crate::facade::WorthUiApp {
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(worth_ui_host_native::staged_appearance_capability_report());
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
    let definition = crate::capability::UiThemeDefinition::admit(
        crate::capability::UiThemeDefinitionIdentity::new("theme.appearance.source").unwrap(),
        1,
        &catalog,
        [(
            token,
            worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
                17, 34, 51, 255,
            ])),
        )],
    )
    .expect("appearance test theme definition");
    crate::capability::FrozenAppearanceThemeCapabilities::admit(
        catalog,
        crate::capability::UiThemeDefinitionIdentity::new("theme.appearance.source").unwrap(),
        vec![definition],
    )
    .expect("appearance test theme bundle")
}
