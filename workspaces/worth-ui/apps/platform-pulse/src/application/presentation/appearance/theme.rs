use super::*;

const GRADIENTS: &[(&str, [[u8; 4]; 2])] = &[
    (
        "theme.platform_pulse.chart_fill",
        [[137, 121, 235, 78], [137, 121, 235, 4]],
    ),
    (
        "theme.platform_pulse.navigation_surface",
        [[25, 37, 61, 255], [18, 27, 44, 255]],
    ),
    (
        "theme.platform_pulse.action_fill",
        [[35, 53, 88, 255], [24, 39, 67, 255]],
    ),
];

pub(super) fn background_kind(slot: &str) -> UiThemeValueKind {
    if GRADIENTS.iter().any(|(identity, _)| *identity == slot) {
        UiThemeValueKind::LinearGradient
    } else {
        UiThemeValueKind::Color
    }
}

pub(super) fn gradient(slot: &str) -> Option<UiThemeLinearGradient> {
    let (_, colors) = GRADIENTS.iter().find(|(identity, _)| *identity == slot)?;
    UiThemeLinearGradient::new(
        UiThemeGradientPoint::new(0, 0).unwrap(),
        UiThemeGradientPoint::new(0, 10_000).unwrap(),
        colors.map(UiThemeColor::from_channels),
    )
}

pub(super) fn token_value(slot: &str, color: UiThemeColor) -> ThemeTokenValue {
    gradient(slot).map_or_else(
        || ThemeTokenValue::color(color),
        ThemeTokenValue::linear_gradient,
    )
}

pub(super) const COLORS: &[(&str, [u8; 4])] = &[
    ("theme.platform_pulse.nav_selected", [35, 52, 84, 255]),
    ("theme.platform_pulse.nav_text", [199, 207, 223, 255]),
    ("theme.platform_pulse.search_fill", [244, 243, 243, 255]),
    ("theme.platform_pulse.lavender_pale", [237, 233, 255, 255]),
    ("theme.platform_pulse.mint_pale", [224, 244, 233, 255]),
    ("theme.platform_pulse.coral_pale", [255, 230, 227, 255]),
    ("theme.platform_pulse.amber_pale", [255, 240, 217, 255]),
    ("theme.platform_pulse.negative", [238, 90, 73, 255]),
    ("theme.platform_pulse.grid", [234, 235, 240, 255]),
    ("theme.platform_pulse.chart_fill", [137, 121, 235, 40]),
    ("theme.platform_pulse.inset_fill", [247, 247, 250, 255]),
    ("theme.platform_pulse.shadow", [18, 26, 46, 70]),
    ("theme.platform_pulse.action_hover", [49, 68, 108, 255]),
    ("theme.platform_pulse.modal_scrim", [32, 30, 40, 70]),
    ("theme.platform_pulse.unused_accent", [0, 0, 0, 255]),
];

pub(super) fn theme_bundle() -> FrozenAppearanceThemeCapabilities {
    let mut values = PlatformPulsePaletteRole::ALL
        .into_iter()
        .map(|p| {
            (
                p.token_id(),
                UiThemeValue::Color(UiThemeColor::from_channels(p.authored_rgba().channels())),
            )
        })
        .chain(PlatformPulseSourceSignalRole::ALL.into_iter().map(|p| {
            (
                p.token_id(),
                UiThemeValue::Color(UiThemeColor::from_channels(p.authored_rgba().channels())),
            )
        }))
        .chain(COLORS.iter().map(|(slot, color)| {
            (
                token(slot),
                UiThemeValue::Color(UiThemeColor::from_channels(*color)),
            )
        }))
        .collect::<Vec<_>>();
    for (slot, value) in &mut values {
        if let Some(gradient) = gradient(slot.as_str()) {
            *value = UiThemeValue::LinearGradient(gradient);
        }
    }
    for radius in [8, 9, 10, 12, 16, 32] {
        let length = UiLogicalLength::new(radius * 1_000);
        values.push((
            token(&format!("theme.platform_pulse.radius.r{radius}")),
            UiThemeValue::CornerRadii(
                UiThemeCornerRadii::new(length, length, length, length).unwrap(),
            ),
        ));
    }
    values.push((
        token("theme.platform_pulse.border.card"),
        UiThemeValue::SolidStroke(
            UiThemeSolidStroke::new(
                UiThemeColor::from_channels([225, 227, 233, 255]),
                UiLogicalLength::new(1_000),
            )
            .unwrap(),
        ),
    ));
    values.push((
        token("theme.platform_pulse.modal_opacity"),
        UiThemeValue::Opacity(UiThemeOpacity::from_ratio(1, 1).unwrap()),
    ));
    let catalog = UiThemeSlotCatalog::admit(
        1,
        values.iter().map(|(slot, value)| {
            UiThemeSlotDeclaration::new(
                slot.clone(),
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
    let identity = UiThemeDefinitionIdentity::new("theme.platform_pulse.default").unwrap();
    let definition =
        UiThemeDefinition::admit(identity.clone(), 1, &catalog, values.clone()).unwrap();
    let alternate = UiThemeDefinition::admit(
        UiThemeDefinitionIdentity::new("theme.platform_pulse.alternate").unwrap(),
        1,
        &catalog,
        values.into_iter().map(|(slot, value)| {
            let replacement = match slot.as_str() {
                "theme.platform_pulse.canvas" => Some([16, 23, 42, 255]),
                "theme.platform_pulse.raised_surface" => Some([25, 36, 59, 255]),
                "theme.platform_pulse.primary_text" => Some([244, 246, 251, 255]),
                "theme.platform_pulse.secondary_text" => Some([168, 177, 196, 255]),
                "theme.platform_pulse.inset_fill" => Some([31, 43, 66, 255]),
                "theme.platform_pulse.search_fill" => Some([31, 43, 66, 255]),
                "theme.platform_pulse.grid" => Some([46, 57, 77, 255]),
                _ => None,
            };
            (
                slot,
                replacement.map_or(value, |color| {
                    UiThemeValue::Color(UiThemeColor::from_channels(color))
                }),
            )
        }),
    )
    .unwrap();
    FrozenAppearanceThemeCapabilities::admit(catalog, identity, vec![definition, alternate])
        .unwrap()
}
