mod review_dialog;
mod theme;

use theme::theme_bundle;
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
    PlatformPulsePaletteRole, PlatformPulseProductComponent, PlatformPulseSourceSignalRole,
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

impl std::fmt::Display for PlatformPulseAppearanceRegistrationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Role(denial) => write!(formatter, "appearance role: {denial:?}"),
            Self::Theme(denial) => write!(formatter, "appearance theme: {denial:?}"),
        }
    }
}

pub(crate) fn register(
    builder: WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied>,
) -> Result<
    WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied>,
    PlatformPulseAppearanceRegistrationDenial,
> {
    let source_signal_blue = PlatformPulseSourceSignalRole::SourceSignalBlue;
    let source_signal_green = PlatformPulseSourceSignalRole::SourceSignalGreen;
    let builder = builder
        .register_theme_token(PlatformPulsePaletteRole::Canvas.token_descriptor())
        .register_theme_token(PlatformPulsePaletteRole::Canvas.source_alias_descriptor())
        .register_theme_token(source_signal_blue.token_descriptor())
        .register_theme_token(source_signal_green.token_descriptor());
    let mut builder = builder
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
    builder = builder
        .register_appearance_role(component_role(
            "platform.pulse.appearance.service_stage",
            PlatformPulseProductComponent::ServiceStage,
            &[
                (
                    UiAppearanceAspect::Background,
                    PlatformPulsePaletteRole::ElevatedSurface.token_id(),
                ),
                (UiAppearanceAspect::Border, token(QUERY_CARD_BORDER)),
            ],
        ))
        .map_err(PlatformPulseAppearanceRegistrationDenial::Role)?
        .register_appearance_role(component_role(
            "platform.pulse.appearance.native_card",
            PlatformPulseProductComponent::NativeCard,
            &[
                (
                    UiAppearanceAspect::Background,
                    PlatformPulsePaletteRole::RaisedSurface.token_id(),
                ),
                (UiAppearanceAspect::Border, token(QUERY_CARD_BORDER)),
            ],
        ))
        .map_err(PlatformPulseAppearanceRegistrationDenial::Role)?;
    for (identity, component, slot) in simple_background_roles() {
        builder = builder
            .register_appearance_role(component_role(
                identity,
                component,
                &[(UiAppearanceAspect::Background, slot)],
            ))
            .map_err(PlatformPulseAppearanceRegistrationDenial::Role)?;
    }
    for (identity, component, slot) in text_foreground_roles() {
        builder = builder
            .register_appearance_role(component_role(
                identity,
                component,
                &[(UiAppearanceAspect::Foreground, slot)],
            ))
            .map_err(PlatformPulseAppearanceRegistrationDenial::Role)?;
    }
    review_dialog::register(builder)?
        .register_appearance_theme_bundle(theme_bundle())
        .map_err(PlatformPulseAppearanceRegistrationDenial::Theme)
}

fn text_foreground_roles() -> [(&'static str, PlatformPulseProductComponent, ThemeTokenId); 23] {
    use PlatformPulsePaletteRole as Palette;
    use PlatformPulseProductComponent as Component;
    [
        (
            "platform.pulse.appearance.runtime_badge",
            Component::RuntimeBadge,
            Palette::Positive.token_id(),
        ),
        (
            "platform.pulse.appearance.evidence_title",
            Component::EvidenceTitle,
            Palette::SecondaryText.token_id(),
        ),
        (
            "platform.pulse.appearance.evidence_body",
            Component::EvidenceBody,
            Palette::PrimaryText.token_id(),
        ),
        (
            "platform.pulse.appearance.source_signal_title",
            Component::SourceSignalTitle,
            Palette::SecondaryText.token_id(),
        ),
        (
            "platform.pulse.appearance.evidence_service_label",
            Component::EvidenceServiceLabel,
            Palette::SecondaryText.token_id(),
        ),
        (
            "platform.pulse.appearance.evidence_service_body",
            Component::EvidenceServiceBody,
            Palette::PrimaryText.token_id(),
        ),
        (
            "platform.pulse.appearance.service_eyebrow",
            Component::ServiceEyebrow,
            Palette::SecondaryText.token_id(),
        ),
        (
            "platform.pulse.appearance.service_title",
            Component::ServiceTitle,
            Palette::PrimaryText.token_id(),
        ),
        (
            "platform.pulse.appearance.service_body",
            Component::ServiceBody,
            Palette::SecondaryText.token_id(),
        ),
        (
            "platform.pulse.appearance.action_label",
            Component::ActionLabel,
            Palette::PrimaryText.token_id(),
        ),
        (
            "platform.pulse.appearance.confirmation_label",
            Component::ConfirmationLabel,
            Palette::SecondaryText.token_id(),
        ),
        (
            "platform.pulse.appearance.portal_label",
            Component::PortalLabel,
            Palette::SecondaryText.token_id(),
        ),
        (
            "platform.pulse.appearance.portal_title",
            Component::PortalTitle,
            Palette::PrimaryText.token_id(),
        ),
        (
            "platform.pulse.appearance.portal_body",
            Component::PortalBody,
            Palette::SecondaryText.token_id(),
        ),
        (
            "platform.pulse.appearance.portal_cancel_label",
            Component::PortalCancelLabel,
            Palette::SecondaryText.token_id(),
        ),
        (
            "platform.pulse.appearance.portal_primary_label",
            Component::PortalPrimaryLabel,
            Palette::ActionText.token_id(),
        ),
        (
            "platform.pulse.appearance.query_label",
            Component::QueryLabel,
            Palette::SecondaryText.token_id(),
        ),
        (
            "platform.pulse.appearance.projected_status",
            Component::ProjectedStatus,
            Palette::Caution.token_id(),
        ),
        (
            "platform.pulse.appearance.native_label",
            Component::NativeLabel,
            Palette::Positive.token_id(),
        ),
        (
            "platform.pulse.appearance.native_body",
            Component::NativeBody,
            Palette::PrimaryText.token_id(),
        ),
        (
            "platform.pulse.appearance.query_denial_label",
            Component::QueryDenialLabel,
            Palette::Caution.token_id(),
        ),
        (
            "platform.pulse.appearance.query_denial_body",
            Component::QueryDenialBody,
            Palette::PrimaryText.token_id(),
        ),
        (
            "platform.pulse.appearance.status_text",
            Component::StatusText,
            Palette::PrimaryText.token_id(),
        ),
    ]
}

fn simple_background_roles() -> [(&'static str, PlatformPulseProductComponent, ThemeTokenId); 13] {
    use PlatformPulsePaletteRole as Palette;
    use PlatformPulseProductComponent as Component;
    [
        (
            "platform.pulse.appearance.evidence_border",
            Component::EvidenceBorder,
            Palette::StructuralRule.token_id(),
        ),
        (
            "platform.pulse.appearance.evidence_rail",
            Component::EvidenceRail,
            Palette::RaisedSurface.token_id(),
        ),
        (
            "platform.pulse.appearance.source_signal_blue",
            Component::SourceSignalActive,
            PlatformPulseSourceSignalRole::SourceSignalBlue.token_id(),
        ),
        (
            "platform.pulse.appearance.source_signal_green",
            Component::SourceSignalActive,
            PlatformPulseSourceSignalRole::SourceSignalGreen.token_id(),
        ),
        (
            "platform.pulse.appearance.query_accent",
            Component::QueryAccent,
            Palette::PrincipalAccent.token_id(),
        ),
        (
            "platform.pulse.appearance.identity_target",
            Component::ActionTarget,
            Palette::ActionFill.token_id(),
        ),
        (
            "platform.pulse.appearance.portal_target",
            Component::PortalTarget,
            Palette::ElevatedSurface.token_id(),
        ),
        (
            "platform.pulse.appearance.portal_accent",
            Component::PortalAccent,
            Palette::PrincipalAccent.token_id(),
        ),
        (
            "platform.pulse.appearance.portal_cancel_target",
            Component::PortalCancelTarget,
            Palette::RaisedSurface.token_id(),
        ),
        (
            "platform.pulse.appearance.portal_primary_target",
            Component::PortalPrimaryTarget,
            Palette::ActionFill.token_id(),
        ),
        (
            "platform.pulse.appearance.confirmation_target",
            Component::ConfirmationTarget,
            Palette::ElevatedSurface.token_id(),
        ),
        (
            "platform.pulse.appearance.lower_shelf_divider",
            Component::LowerShelfDivider,
            Palette::StructuralRule.token_id(),
        ),
        (
            "platform.pulse.appearance.masthead_border",
            Component::MastheadBorder,
            Palette::StructuralRule.token_id(),
        ),
    ]
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

fn token(value: &str) -> ThemeTokenId {
    ThemeTokenId::new(value).expect("Pulse appearance token identity is valid")
}
