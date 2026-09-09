use crate::capability::*;
use worth_ui_dsl::*;

pub(super) const TEXT_TOKEN: &str = "theme.integrated.text";

pub(super) fn theme() -> FrozenAppearanceThemeCapabilities {
    let values = [
        (
            "overlay.content.background",
            UiThemeValue::Color(UiThemeColor::from_channels([32, 64, 96, 128])),
        ),
        (
            "overlay.content.hovered",
            UiThemeValue::Color(UiThemeColor::from_channels([192, 64, 32, 128])),
        ),
        (
            "overlay.content.foreground",
            UiThemeValue::Color(UiThemeColor::from_channels([240, 224, 208, 255])),
        ),
        (
            "overlay.content.border",
            UiThemeValue::SolidStroke(
                UiThemeSolidStroke::new(
                    UiThemeColor::from_channels([224, 48, 16, 255]),
                    UiLogicalLength::new(1_000),
                )
                .unwrap(),
            ),
        ),
        (
            "overlay.content.radius",
            UiThemeValue::CornerRadii(
                UiThemeCornerRadii::new(
                    UiLogicalLength::new(10_000),
                    UiLogicalLength::new(10_000),
                    UiLogicalLength::new(10_000),
                    UiLogicalLength::new(10_000),
                )
                .unwrap(),
            ),
        ),
        (
            "overlay.content.opacity",
            UiThemeValue::Opacity(UiThemeOpacity::from_ratio(40_000, 65_535).unwrap()),
        ),
        (
            "overlay.content.outline",
            UiThemeValue::SolidOutline(
                UiThemeOutline::new(
                    UiThemeSolidStroke::new(
                        UiThemeColor::from_channels([16, 160, 224, 255]),
                        UiLogicalLength::new(1_000),
                    )
                    .unwrap(),
                    UiLogicalLength::new(500),
                )
                .unwrap(),
            ),
        ),
        (
            "overlay.scrim.background",
            UiThemeValue::Color(UiThemeColor::from_channels([4, 8, 12, 255])),
        ),
        (
            "overlay.scrim.opacity",
            UiThemeValue::Opacity(UiThemeOpacity::from_ratio(1, 2).unwrap()),
        ),
    ];
    let catalog = UiThemeSlotCatalog::admit(
        1,
        values.iter().map(|(name, value)| {
            UiThemeSlotDeclaration::new(
                ThemeTokenId::new(*name).unwrap(),
                ThemeTokenFamily::surface(),
                value.kind(),
                ThemeTokenSource::application(),
                UiThemeSlotDisclosure::Public,
                UiThemeSlotSuccessorCompatibility::ExactMeaning,
                None,
            )
        }),
    )
    .unwrap();
    let identity = UiThemeDefinitionIdentity::new("theme.integrated.overlay").unwrap();
    let definition = UiThemeDefinition::admit(
        identity.clone(),
        1,
        &catalog,
        values
            .into_iter()
            .map(|(name, value)| (ThemeTokenId::new(name).unwrap(), value)),
    )
    .unwrap();
    FrozenAppearanceThemeCapabilities::admit(catalog, identity, vec![definition]).unwrap()
}
