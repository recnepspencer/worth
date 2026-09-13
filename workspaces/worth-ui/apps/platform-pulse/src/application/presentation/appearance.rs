mod theme;
use worth_ui::facade::app::{
    UiChangeProfileInstalled, UiIntentWiringSatisfied, WorthUiApplicationBuilder,
};
use worth_ui::facade::appearance::*;
use worth_ui::facade::declaration::{
    ThemeTokenDescriptor, ThemeTokenFamily, ThemeTokenId, ThemeTokenSource, ThemeTokenValue,
};
use worth_ui_platform_pulse::product_world::{
    dashboard_elements, DashboardContent, PlatformPulsePaletteRole, PlatformPulseSourceSignalRole,
};

#[derive(Debug)]
pub(crate) enum PlatformPulseAppearanceRegistrationDenial {
    Role(AppearanceRoleRegistrationDenial),
    Theme(FrozenAppearanceThemeCapabilitiesDenial),
}
impl std::fmt::Display for PlatformPulseAppearanceRegistrationDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Role(denial) => write!(f, "role registration: {denial:?}"),
            Self::Theme(denial) => write!(f, "theme registration: {denial:?}"),
        }
    }
}

pub(crate) fn register(
    mut builder: WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied>,
) -> Result<
    WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied>,
    PlatformPulseAppearanceRegistrationDenial,
> {
    for palette in PlatformPulsePaletteRole::ALL {
        builder = builder.register_theme_token(palette.token_descriptor());
        let descriptor = if let Some(gradient) = theme::gradient(palette.token_id().as_str()) {
            ThemeTokenDescriptor::define(
                palette.token_id(),
                ThemeTokenFamily::surface(),
                ThemeTokenSource::application(),
                ThemeTokenValue::linear_gradient(gradient),
            )
        } else {
            palette.source_alias_descriptor()
        };
        builder = builder.register_theme_token(descriptor);
    }
    for signal in PlatformPulseSourceSignalRole::ALL {
        builder = builder.register_theme_token(signal.token_descriptor());
    }
    for (slot, color) in theme::COLORS {
        builder = builder.register_theme_token(ThemeTokenDescriptor::define(
            token(slot),
            ThemeTokenFamily::surface(),
            ThemeTokenSource::application(),
            theme::token_value(slot, UiThemeColor::from_channels(*color)),
        ));
    }
    for element in dashboard_elements() {
        let mut role =
            UiAppearanceRole::authoring(UiAppearanceRoleIdentity::new(element.role_id()).unwrap())
                .applies_to_component(
                    UiDslComponentReference::new(element.component_id()).unwrap(),
                );
        let aspect = if matches!(element.content, DashboardContent::Text { .. }) {
            UiAppearanceAspect::Foreground
        } else {
            UiAppearanceAspect::Background
        };
        let normal = element.color_token();
        let normal_kind = if aspect == UiAppearanceAspect::Background {
            theme::background_kind(&normal)
        } else {
            UiThemeValueKind::Color
        };
        let partition = if element.interaction.is_some() {
            UiAppearancePartitionAuthoring::new([UiAppearanceAxisDomain::complete(
                UiAppearanceStateAxis::Hover,
            )])
            .with_cell(
                UiAppearanceCell::named("outside")
                    .when([UiAppearanceAxisPredicate::exact(
                        UiAppearanceAxisClass::HoverOutside,
                    )])
                    .uses_slot(
                        UiThemeSlotIdentity::new(normal.as_str()).unwrap(),
                        normal_kind,
                    ),
            )
            .with_cell(
                UiAppearanceCell::named("hovered")
                    .when([UiAppearanceAxisPredicate::exact(
                        UiAppearanceAxisClass::Hovered,
                    )])
                    .uses_slot(
                        UiThemeSlotIdentity::new(if element.color == "action_fill" {
                            "theme.platform_pulse.action_hover"
                        } else {
                            "theme.platform_pulse.hover_surface"
                        })
                        .unwrap(),
                        UiThemeValueKind::Color,
                    ),
            )
        } else {
            fixed(&normal, normal_kind)
        };
        role = role.cover(aspect, partition).unwrap();
        if let DashboardContent::Surface { radius, border, .. } = element.content {
            if radius > 0 {
                role = role
                    .cover(
                        UiAppearanceAspect::Radius,
                        fixed(
                            &format!("theme.platform_pulse.radius.r{radius}"),
                            UiThemeValueKind::CornerRadii,
                        ),
                    )
                    .unwrap();
            }
            if border {
                role = role
                    .cover(
                        UiAppearanceAspect::Border,
                        fixed(
                            "theme.platform_pulse.border.card",
                            UiThemeValueKind::SolidStroke,
                        ),
                    )
                    .unwrap();
            }
        }
        builder = builder
            .register_appearance_role(role.build().unwrap())
            .map_err(PlatformPulseAppearanceRegistrationDenial::Role)?;
    }
    let role = UiAppearanceRole::authoring(
        UiAppearanceRoleIdentity::new("platform.pulse.appearance.modal_backdrop").unwrap(),
    )
    .applies_to_backdrop()
    .cover(
        UiAppearanceAspect::Background,
        fixed("theme.platform_pulse.modal_scrim", UiThemeValueKind::Color),
    )
    .unwrap()
    .cover(
        UiAppearanceAspect::Opacity,
        fixed(
            "theme.platform_pulse.modal_opacity",
            UiThemeValueKind::Opacity,
        ),
    )
    .unwrap()
    .build()
    .unwrap();
    builder = builder
        .register_appearance_role(role)
        .map_err(PlatformPulseAppearanceRegistrationDenial::Role)?;
    builder
        .register_appearance_theme_bundle(theme::theme_bundle())
        .map_err(PlatformPulseAppearanceRegistrationDenial::Theme)
}
fn fixed(slot: &str, kind: UiThemeValueKind) -> UiAppearancePartitionAuthoring {
    UiAppearancePartitionAuthoring::new([]).with_cell(
        UiAppearanceCell::when([]).uses_slot(UiThemeSlotIdentity::new(slot).unwrap(), kind),
    )
}
fn token(value: &str) -> ThemeTokenId {
    ThemeTokenId::new(value).unwrap()
}
