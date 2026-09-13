use worth_ui::facade::appearance::*;
use worth_ui::facade::declaration::{ThemeTokenFamily, ThemeTokenId, ThemeTokenSource};

pub(super) fn roles() -> [UiAppearanceRoleDeclaration; 2] {
    [
        (
            super::ROOT,
            super::ROOT_TOKEN,
            UiAppearanceAspect::Background,
        ),
        (
            super::TEXT,
            super::TEXT_TOKEN,
            UiAppearanceAspect::Foreground,
        ),
    ]
    .map(|(component, token, aspect)| {
        UiAppearanceRole::authoring(UiAppearanceRoleIdentity::new(component).unwrap())
            .applies_to_component(UiDslComponentReference::new(component).unwrap())
            .cover(
                aspect,
                UiAppearancePartitionAuthoring::new([]).with_cell(
                    UiAppearanceCell::when([]).uses_slot(
                        UiThemeSlotIdentity::new(token).unwrap(),
                        UiThemeValueKind::Color,
                    ),
                ),
            )
            .unwrap()
            .build()
            .unwrap()
    })
}

pub(super) fn theme() -> FrozenAppearanceThemeCapabilities {
    let colors = [
        (super::ROOT_TOKEN, "#17202a"),
        (super::TEXT_TOKEN, "#ffffff"),
    ];
    let catalog = UiThemeSlotCatalog::admit(
        1,
        colors.map(|(token, _)| {
            UiThemeSlotDeclaration::new(
                ThemeTokenId::new(token).unwrap(),
                ThemeTokenFamily::surface(),
                UiThemeValueKind::Color,
                ThemeTokenSource::application(),
                UiThemeSlotDisclosure::Public,
                UiThemeSlotSuccessorCompatibility::ExactMeaning,
                None,
            )
        }),
    )
    .unwrap();
    let identity = UiThemeDefinitionIdentity::new("theme.phase_f").unwrap();
    let definition = UiThemeDefinition::admit(
        identity.clone(),
        1,
        &catalog,
        colors.map(|(token, color)| {
            (
                ThemeTokenId::new(token).unwrap(),
                UiThemeValue::Color(UiThemeColor::parse(color).unwrap()),
            )
        }),
    )
    .unwrap();
    FrozenAppearanceThemeCapabilities::admit(catalog, identity, vec![definition]).unwrap()
}
