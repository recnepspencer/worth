use super::super::UiApplicationPresentationState;
use crate::runtime::tests::appearance_component_session_test_support as component_support;
use crate::runtime::tests::appearance_theme_test_support;

pub(super) fn owner_state_fixture(
    bundle: crate::capability::FrozenAppearanceThemeCapabilities,
    extra_token: Option<crate::capability::ThemeTokenDescriptor>,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    UiApplicationPresentationState,
    worth_ui_host_contract::UiSemanticSurfaceIdentity,
    worth_ui_dsl::UiAppearanceRoleDeclaration,
    crate::graph::UiGraphConsumedFactIndex,
) {
    let role = component_support::validation_background_role(component_support::APPEARANCE_TOKEN);
    let builder = component_support::legacy_static_paint_appearance_component_builder(&role)
        .register_appearance_theme_bundle(bundle)
        .unwrap();
    let builder = match extra_token {
        Some(token) => builder.register_theme_token(token),
        None => builder,
    };
    let application = builder
        .with_rust_authored_declaration_fixture(component_support::appearance_fixture(&role))
        .freeze()
        .map(appearance_theme_test_support::activate)
        .expect("theme-value owner fixture should prepare");
    let index = application
        .prepared_authority()
        .consumed_fact_index()
        .clone();
    let mut session = application
        .launch()
        .expect("theme-value owner fixture should launch");
    let surface = session.create_semantic_surface().unwrap();
    let capability = issue_capability(
        &session,
        &role,
        surface,
        session
            .capabilities()
            .appearance_themes()
            .unwrap()
            .initial_definition_identity()
            .as_str(),
    );
    let mut state = UiApplicationPresentationState::activate(session.capabilities());
    state
        .materialize_initial_appearance_theme_binding(capability)
        .unwrap();
    (session, state, surface, role, index)
}

pub(super) fn issue_capability(
    session: &crate::facade::WorthUiActiveApplicationSession,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    definition: &str,
) -> crate::runtime::appearance::UiThemeCapabilityReceipt {
    let themes = session.capabilities().appearance_themes().unwrap();
    let profile = worth_ui_host_native::staged_appearance_capability_report()
        .appearance_profile()
        .cloned()
        .unwrap();
    crate::runtime::appearance::UiThemeCapabilityAdmission::from_frozen_capabilities(
        themes,
        &crate::capability::UiThemeDefinitionIdentity::new(definition).unwrap(),
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

pub(super) fn single_bundle(
    initial: [u8; 4],
) -> crate::capability::FrozenAppearanceThemeCapabilities {
    let token = theme_token();
    let catalog = catalog([slot(token.clone(), None)]);
    let identity =
        crate::capability::UiThemeDefinitionIdentity::new("theme.truth.initial").unwrap();
    let definition = crate::capability::UiThemeDefinition::admit(
        identity.clone(),
        1,
        &catalog,
        [(token, typed_color(initial))],
    )
    .unwrap();
    crate::capability::FrozenAppearanceThemeCapabilities::admit(catalog, identity, vec![definition])
        .unwrap()
}

pub(super) fn multi_bundle() -> crate::capability::FrozenAppearanceThemeCapabilities {
    let token = theme_token();
    let catalog = catalog([
        slot(token.clone(), None),
        slot(global_alias_token(), Some(token.as_str())),
    ]);
    let initial =
        crate::capability::UiThemeDefinitionIdentity::new("theme.truth.surface_a").unwrap();
    let first = crate::capability::UiThemeDefinition::admit(
        crate::capability::UiThemeDefinitionIdentity::new("theme.truth.surface_a").unwrap(),
        1,
        &catalog,
        [(token.clone(), typed_color([17, 34, 51, 255]))],
    )
    .unwrap();
    let second = crate::capability::UiThemeDefinition::admit(
        crate::capability::UiThemeDefinitionIdentity::new("theme.truth.surface_b").unwrap(),
        1,
        &catalog,
        [(token, typed_color([68, 85, 102, 255]))],
    )
    .unwrap();
    crate::capability::FrozenAppearanceThemeCapabilities::admit(
        catalog,
        initial,
        vec![first, second],
    )
    .unwrap()
}

pub(super) fn conflicting_bundle() -> crate::capability::FrozenAppearanceThemeCapabilities {
    let terminal = crate::capability::ThemeTokenId::new("theme.truth.terminal").unwrap();
    let catalog = catalog([
        slot(terminal.clone(), None),
        slot(theme_token(), Some(terminal.as_str())),
        slot(second_legacy_token(), Some(terminal.as_str())),
    ]);
    let identity =
        crate::capability::UiThemeDefinitionIdentity::new("theme.truth.conflict").unwrap();
    let definition = crate::capability::UiThemeDefinition::admit(
        identity.clone(),
        1,
        &catalog,
        [(terminal, typed_color([68, 85, 102, 255]))],
    )
    .unwrap();
    crate::capability::FrozenAppearanceThemeCapabilities::admit(catalog, identity, vec![definition])
        .unwrap()
}

fn catalog(
    slots: impl IntoIterator<Item = crate::capability::UiThemeSlotDeclaration>,
) -> crate::capability::UiThemeSlotCatalog {
    crate::capability::UiThemeSlotCatalog::admit(1, slots).unwrap()
}

fn slot(
    token: crate::capability::ThemeTokenId,
    alias_target: Option<&str>,
) -> crate::capability::UiThemeSlotDeclaration {
    crate::capability::UiThemeSlotDeclaration::new(
        token,
        crate::capability::ThemeTokenFamily::surface(),
        worth_ui_dsl::UiThemeValueKind::Color,
        crate::capability::ThemeTokenSource::application(),
        crate::capability::UiThemeSlotDisclosure::Public,
        crate::capability::UiThemeSlotSuccessorCompatibility::ExactMeaning,
        alias_target.map(|target| crate::capability::ThemeTokenId::new(target).unwrap()),
    )
}

pub(super) fn global_alias_descriptor() -> crate::capability::ThemeTokenDescriptor {
    crate::capability::ThemeTokenDescriptor::alias(
        global_alias_token(),
        crate::capability::ThemeTokenFamily::surface(),
        crate::capability::ThemeTokenSource::application(),
        crate::capability::ThemeTokenAlias::to(theme_token()),
    )
}

pub(super) fn second_legacy_descriptor() -> crate::capability::ThemeTokenDescriptor {
    crate::capability::ThemeTokenDescriptor::define(
        second_legacy_token(),
        crate::capability::ThemeTokenFamily::surface(),
        crate::capability::ThemeTokenSource::application(),
        legacy_color("#112233"),
    )
}

pub(super) fn theme_token() -> crate::capability::ThemeTokenId {
    crate::capability::ThemeTokenId::new(component_support::APPEARANCE_TOKEN).unwrap()
}

pub(super) fn global_alias_token() -> crate::capability::ThemeTokenId {
    crate::capability::ThemeTokenId::new("theme.appearance_consumer.alias").unwrap()
}

pub(super) fn second_legacy_token() -> crate::capability::ThemeTokenId {
    crate::capability::ThemeTokenId::new("theme.appearance_consumer.second").unwrap()
}

pub(super) fn legacy_color(hex: &str) -> crate::capability::ThemeTokenValue {
    crate::capability::ThemeTokenValue::color(crate::capability::ThemeColorValue::hex(hex).unwrap())
}

pub(super) fn typed_color(channels: [u8; 4]) -> worth_ui_dsl::UiThemeValue {
    worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels(channels))
}

pub(super) fn theme_change(
    token: crate::capability::ThemeTokenId,
    revision: u64,
    value: crate::capability::ThemeTokenValue,
) -> crate::facade::entry::UiNativeThemeTokenValueChange {
    crate::facade::entry::UiNativeThemeTokenValueChange::successor(token, revision, value).unwrap()
}
