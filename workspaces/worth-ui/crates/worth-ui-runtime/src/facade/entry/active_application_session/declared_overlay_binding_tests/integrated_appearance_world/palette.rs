use crate::capability::*;
use worth_ui_dsl::*;
use worth_ui_host_contract::*;

pub(super) const TEXT_TOKEN: &str = "theme.integrated.text";

pub(super) fn assert_green_content(
    output: &UiUnpublishedAppearanceFrameProjection,
    instance: UiMountedInstanceIdentity,
    opacity: u16,
) {
    let content = output
        .fragments()
        .iter()
        .find(|fragment| {
            matches!(fragment.identity(),
                UiUnpublishedAppearanceFragmentIdentity::NodeReceipt { successor: Some(receipt), .. }
                    if receipt.mounted_instance() == instance)
        })
        .unwrap();
    assert_eq!(content.work().successor().mechanics().len(), 3);
    for mechanic in content.work().successor().mechanics() {
        match mechanic {
            UiMountedAppearanceMechanic::Surface(row) => {
                assert!(
                    matches!(row.paint(), UiMountedSurfacePaint::FillAndBorder { fill, .. }
                    if fill.straight_srgba() == [16, 128, 32, 128])
                );
                assert_eq!(row.opacity().units(), opacity);
            }
            UiMountedAppearanceMechanic::TextForeground(row) => {
                assert_eq!(row.foreground().straight_srgba(), [208, 240, 224, 255]);
                assert_eq!(row.opacity().units(), opacity);
            }
            UiMountedAppearanceMechanic::Outline(row) => assert_eq!(row.opacity().units(), opacity),
            _ => panic!("content has surface, original-range text, and outline mechanics"),
        }
    }
    assert!(content
        .text_candidates()
        .iter()
        .any(|text| text.text() == "AB"));
}

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
        (
            "overlay.unused.background",
            UiThemeValue::Color(UiThemeColor::from_channels([1, 2, 3, 255])),
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
    let variant = |name: &str, changes: &[(&str, [u8; 4])]| {
        UiThemeDefinition::admit(
            UiThemeDefinitionIdentity::new(name).unwrap(),
            1,
            &catalog,
            values.into_iter().map(|(slot, value)| {
                let value = changes
                    .iter()
                    .find(|(changed, _)| *changed == slot)
                    .map_or(value, |(_, channels)| {
                        UiThemeValue::Color(UiThemeColor::from_channels(*channels))
                    });
                (ThemeTokenId::new(slot).unwrap(), value)
            }),
        )
        .unwrap()
    };
    let equal = variant("theme.integrated.equal", &[]);
    let unused = variant(
        "theme.integrated.unused",
        &[("overlay.unused.background", [9, 8, 7, 255])],
    );
    let backdrop = variant(
        "theme.integrated.backdrop",
        &[("overlay.scrim.background", [12, 4, 8, 255])],
    );
    let green = variant(
        "theme.integrated.green",
        &[
            ("overlay.content.background", [16, 128, 32, 128]),
            ("overlay.content.foreground", [208, 240, 224, 255]),
            ("overlay.scrim.background", [12, 4, 8, 255]),
        ],
    );
    FrozenAppearanceThemeCapabilities::admit(
        catalog,
        identity,
        vec![definition, equal, unused, backdrop, green],
    )
    .unwrap()
}
