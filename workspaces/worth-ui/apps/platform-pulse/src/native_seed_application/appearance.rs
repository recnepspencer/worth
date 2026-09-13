use worth_ui::facade::appearance::*;
use worth_ui::facade::declaration::{ThemeTokenFamily, ThemeTokenId, ThemeTokenSource};

pub(super) const GREEN: &str = "theme.platform_pulse.native_seed.green";

pub(super) fn role() -> UiAppearanceRoleDeclaration {
    UiAppearanceRole::authoring(
        UiAppearanceRoleIdentity::new("platform.pulse.native_seed.background").unwrap(),
    )
    .applies_to_component(UiDslComponentReference::new(super::COMPONENT).unwrap())
    .cover(
        UiAppearanceAspect::Background,
        UiAppearancePartitionAuthoring::new([]).with_cell(UiAppearanceCell::when([]).uses_slot(
            UiThemeSlotIdentity::new(super::TOKEN).unwrap(),
            UiThemeValueKind::Color,
        )),
    )
    .unwrap()
    .build()
    .unwrap()
}

pub(super) fn theme() -> FrozenAppearanceThemeCapabilities {
    let token = ThemeTokenId::new(super::TOKEN).unwrap();
    let catalog = UiThemeSlotCatalog::admit(
        1,
        [UiThemeSlotDeclaration::new(
            token.clone(),
            ThemeTokenFamily::surface(),
            UiThemeValueKind::Color,
            ThemeTokenSource::application(),
            UiThemeSlotDisclosure::Public,
            UiThemeSlotSuccessorCompatibility::ExactMeaning,
            None,
        )],
    )
    .unwrap();
    let identity = UiThemeDefinitionIdentity::new("theme.platform_pulse.native_seed").unwrap();
    let definition = UiThemeDefinition::admit(
        identity.clone(),
        1,
        &catalog,
        [(
            token.clone(),
            UiThemeValue::Color(UiThemeColor::parse("#2f81f7").unwrap()),
        )],
    )
    .unwrap();
    let green = UiThemeDefinition::admit(
        UiThemeDefinitionIdentity::new(GREEN).unwrap(),
        1,
        &catalog,
        [(
            token,
            UiThemeValue::Color(UiThemeColor::parse("#3fb950").unwrap()),
        )],
    )
    .unwrap();
    FrozenAppearanceThemeCapabilities::admit(catalog, identity, vec![definition, green]).unwrap()
}
