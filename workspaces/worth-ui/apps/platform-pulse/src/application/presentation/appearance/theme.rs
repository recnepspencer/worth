use super::*;

pub(super) fn theme_bundle() -> FrozenAppearanceThemeCapabilities {
    let color_slots = PlatformPulsePaletteRole::ALL
        .into_iter()
        .map(|role| (role.token_id(), role.authored_rgba().channels()))
        .chain(
            PlatformPulseSourceSignalRole::ALL
                .into_iter()
                .map(|role| (role.token_id(), role.authored_rgba().channels())),
        )
        .chain(std::iter::once((
            token("theme.platform_pulse.unused_accent"),
            [0, 0, 0, 255],
        )))
        .chain(std::iter::once((
            token("theme.platform_pulse.modal_scrim"),
            [0, 0, 0, 128],
        )))
        .collect::<Vec<_>>();
    let radius = token(QUERY_CARD_RADIUS);
    let border = token(QUERY_CARD_BORDER);
    let catalog = UiThemeSlotCatalog::admit(
        1,
        color_slots
            .iter()
            .map(|(slot, _)| slot_declaration(slot.clone(), UiThemeValueKind::Color))
            .chain(std::iter::once(slot_declaration(
                token("theme.platform_pulse.modal_opacity"),
                UiThemeValueKind::Opacity,
            )))
            .chain(std::iter::once(slot_declaration(
                border.clone(),
                UiThemeValueKind::SolidStroke,
            )))
            .chain(std::iter::once(slot_declaration(
                radius.clone(),
                UiThemeValueKind::CornerRadii,
            ))),
    )
    .expect("Pulse appearance catalog is valid");
    let identity = UiThemeDefinitionIdentity::new("theme.platform_pulse.default").unwrap();
    let values = color_slots
        .into_iter()
        .map(|(slot, channels)| {
            (
                slot,
                UiThemeValue::Color(UiThemeColor::from_channels(channels)),
            )
        })
        .chain(std::iter::once((
            token("theme.platform_pulse.modal_opacity"),
            UiThemeValue::Opacity(
                worth_ui::facade::appearance::UiThemeOpacity::from_ratio(1, 1).unwrap(),
            ),
        )))
        .chain(std::iter::once((
            border,
            UiThemeValue::SolidStroke(
                UiThemeSolidStroke::new(
                    UiThemeColor::from_channels(
                        PlatformPulsePaletteRole::StructuralRule
                            .authored_rgba()
                            .channels(),
                    ),
                    UiLogicalLength::new(1_000),
                )
                .unwrap(),
            ),
        )))
        .chain(std::iter::once((
            radius,
            UiThemeValue::CornerRadii(
                UiThemeCornerRadii::new(
                    UiLogicalLength::new(24_000),
                    UiLogicalLength::new(24_000),
                    UiLogicalLength::new(24_000),
                    UiLogicalLength::new(24_000),
                )
                .unwrap(),
            ),
        )))
        .collect::<Vec<_>>();
    let alternate_values = values.iter().cloned().map(|(slot, value)| {
        let value = if slot == PlatformPulsePaletteRole::Canvas.token_id() {
            UiThemeValue::Color(UiThemeColor::from_channels([0x18, 0x12, 0x0D, 0xFF]))
        } else if slot == PlatformPulsePaletteRole::RaisedSurface.token_id() {
            UiThemeValue::Color(UiThemeColor::from_channels([0x24, 0x1B, 0x14, 0xFF]))
        } else if slot == PlatformPulsePaletteRole::PrincipalAccent.token_id() {
            UiThemeValue::Color(UiThemeColor::from_channels([0xF2, 0xAD, 0x67, 0xFF]))
        } else if slot == PlatformPulsePaletteRole::ActionFill.token_id() {
            UiThemeValue::Color(UiThemeColor::from_channels([0xA0, 0x54, 0x18, 0xFF]))
        } else if slot == token("theme.platform_pulse.unused_accent") {
            UiThemeValue::Color(UiThemeColor::from_channels([255, 255, 255, 255]))
        } else {
            value
        };
        (slot, value)
    });
    let alternate = UiThemeDefinition::admit(
        UiThemeDefinitionIdentity::new("theme.platform_pulse.alternate").unwrap(),
        1,
        &catalog,
        alternate_values,
    )
    .expect("Pulse alternate theme is complete");
    let definition = UiThemeDefinition::admit(identity.clone(), 1, &catalog, values)
        .expect("Pulse appearance theme is complete");
    FrozenAppearanceThemeCapabilities::admit(catalog, identity, vec![definition, alternate])
        .expect("Pulse appearance theme bundle is valid")
}

fn slot_declaration(identity: ThemeTokenId, kind: UiThemeValueKind) -> UiThemeSlotDeclaration {
    UiThemeSlotDeclaration::new(
        identity,
        ThemeTokenFamily::surface(),
        kind,
        ThemeTokenSource::application(),
        UiThemeSlotDisclosure::Public,
        UiThemeSlotSuccessorCompatibility::ExactMeaning,
        None,
    )
}
