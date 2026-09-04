use super::support;

pub(super) fn theme_session(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
) {
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    let host_observer = host.clone();
    host.set_capabilities(worth_ui_host_native::staged_appearance_capability_report());
    let session = support::legacy_static_paint_appearance_component_builder(role)
        .register_appearance_theme_bundle(theme_bundle())
        .unwrap()
        .with_rust_authored_declaration_fixture(support::appearance_fixture(role))
        .freeze()
        .map(|application| {
            host.push_native_display_presented();
            crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                application,
                host,
            )
        })
        .expect("appearance capability fixture should prepare")
        .launch()
        .expect("appearance capability fixture should launch");
    (session, host_observer)
}

pub(super) fn publish_frame(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    now: u64,
) {
    let outcome = session
        .execute_mounted_frame(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
            now,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("appearance support frame should execute"));
    assert!(matches!(
        outcome,
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
}

fn theme_bundle() -> crate::capability::FrozenAppearanceThemeCapabilities {
    let token = crate::capability::ThemeTokenId::new(support::APPEARANCE_TOKEN).unwrap();
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
    .unwrap();
    let definition = crate::capability::UiThemeDefinition::admit(
        crate::capability::UiThemeDefinitionIdentity::new("theme.appearance.production").unwrap(),
        1,
        &catalog,
        [(
            token,
            worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
                17, 34, 51, 255,
            ])),
        )],
    )
    .unwrap();
    crate::capability::FrozenAppearanceThemeCapabilities::admit(
        catalog,
        crate::capability::UiThemeDefinitionIdentity::new("theme.appearance.production").unwrap(),
        vec![definition],
    )
    .unwrap()
}
