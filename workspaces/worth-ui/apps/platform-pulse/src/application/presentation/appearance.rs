use worth_ui::facade::app::{
    UiChangeProfileInstalled, UiIntentWiringSatisfied, WorthUiApplicationBuilder,
};
use worth_ui::facade::appearance::{
    FrozenAppearanceThemeCapabilities, FrozenAppearanceThemeCapabilitiesDenial, UiAppearanceAspect,
    UiAppearanceCell, UiAppearancePartitionAuthoring, UiAppearanceRole, UiAppearanceRoleIdentity,
    UiDslComponentReference, UiLogicalLength, UiThemeColor, UiThemeCornerRadii, UiThemeDefinition,
    UiThemeDefinitionIdentity, UiThemeSlotCatalog, UiThemeSlotDeclaration, UiThemeSlotDisclosure,
    UiThemeSlotIdentity, UiThemeSlotSuccessorCompatibility, UiThemeSolidStroke, UiThemeValue,
    UiThemeValueKind,
};
use worth_ui::facade::declaration::{ThemeTokenFamily, ThemeTokenId, ThemeTokenSource};
use worth_ui_platform_pulse::product_world::{
    PlatformPulsePaletteRole, PlatformPulseProductComponent,
};

const ROOT_ROLE: &str = "platform.pulse.appearance.root";
const BRAND_ROLE: &str = "platform.pulse.appearance.brand";
const QUERY_CARD_ROLE: &str = "platform.pulse.appearance.query_card";
const QUERY_CARD_RADIUS: &str = "theme.platform_pulse.radius.query_card";
const QUERY_CARD_BORDER: &str = "theme.platform_pulse.border.query_card";

#[derive(Debug)]
pub(crate) enum PlatformPulseAppearanceRegistrationDenial {
    Role(worth_ui::facade::appearance::AppearanceRoleRegistrationDenial),
    Theme(FrozenAppearanceThemeCapabilitiesDenial),
}

pub(crate) fn register(
    builder: WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied>,
) -> Result<
    WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied>,
    PlatformPulseAppearanceRegistrationDenial,
> {
    let builder = builder
        .register_appearance_role(component_role(
            ROOT_ROLE,
            PlatformPulseProductComponent::Root,
            &[(
                UiAppearanceAspect::Background,
                PlatformPulsePaletteRole::Canvas.token_id(),
            )],
        ))
        .map_err(PlatformPulseAppearanceRegistrationDenial::Role)?
        .register_appearance_role(component_role(
            BRAND_ROLE,
            PlatformPulseProductComponent::Brand,
            &[(
                UiAppearanceAspect::Foreground,
                PlatformPulsePaletteRole::PrincipalAccent.token_id(),
            )],
        ))
        .map_err(PlatformPulseAppearanceRegistrationDenial::Role)?
        .register_appearance_role(component_role(
            QUERY_CARD_ROLE,
            PlatformPulseProductComponent::QueryCard,
            &[
                (
                    UiAppearanceAspect::Background,
                    PlatformPulsePaletteRole::RaisedSurface.token_id(),
                ),
                (UiAppearanceAspect::Border, token(QUERY_CARD_BORDER)),
                (UiAppearanceAspect::Radius, token(QUERY_CARD_RADIUS)),
            ],
        ))
        .map_err(PlatformPulseAppearanceRegistrationDenial::Role)?;
    builder
        .register_appearance_theme_bundle(theme_bundle())
        .map_err(PlatformPulseAppearanceRegistrationDenial::Theme)
}

fn component_role(
    identity: &str,
    component: PlatformPulseProductComponent,
    aspects: &[(UiAppearanceAspect, ThemeTokenId)],
) -> worth_ui::facade::appearance::UiAppearanceRoleDeclaration {
    let mut role = UiAppearanceRole::authoring(UiAppearanceRoleIdentity::new(identity).unwrap())
        .applies_to_component(UiDslComponentReference::new(component.id()).unwrap());
    for (aspect, slot) in aspects {
        role = role
            .cover(
                *aspect,
                UiAppearancePartitionAuthoring::new([]).with_cell(
                    UiAppearanceCell::when([]).uses_slot(
                        UiThemeSlotIdentity::new(slot.as_str()).unwrap(),
                        aspect.value_kind(),
                    ),
                ),
            )
            .expect("Pulse appearance partitions are complete");
    }
    role.build().expect("Pulse appearance roles are valid")
}

fn theme_bundle() -> FrozenAppearanceThemeCapabilities {
    let color_slots = [
        (
            PlatformPulsePaletteRole::Canvas.token_id(),
            PlatformPulsePaletteRole::Canvas.authored_rgba().channels(),
        ),
        (
            PlatformPulsePaletteRole::PrincipalAccent.token_id(),
            PlatformPulsePaletteRole::PrincipalAccent
                .authored_rgba()
                .channels(),
        ),
        (
            PlatformPulsePaletteRole::RaisedSurface.token_id(),
            PlatformPulsePaletteRole::RaisedSurface
                .authored_rgba()
                .channels(),
        ),
    ];
    let radius = token(QUERY_CARD_RADIUS);
    let border = token(QUERY_CARD_BORDER);
    let catalog = UiThemeSlotCatalog::admit(
        1,
        color_slots
            .iter()
            .map(|(slot, _)| slot_declaration(slot.clone(), UiThemeValueKind::Color))
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
        )));
    let definition = UiThemeDefinition::admit(identity.clone(), 1, &catalog, values)
        .expect("Pulse appearance theme is complete");
    FrozenAppearanceThemeCapabilities::admit(catalog, identity, vec![definition])
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

fn token(value: &str) -> ThemeTokenId {
    ThemeTokenId::new(value).expect("Pulse appearance token identity is valid")
}
