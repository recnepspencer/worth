use super::*;

pub(super) fn register(
    mut builder: WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied>,
) -> Result<
    WorthUiApplicationBuilder<UiChangeProfileInstalled, UiIntentWiringSatisfied>,
    PlatformPulseAppearanceRegistrationDenial,
> {
    for (identity, component, aspect, palette) in [
        (
            "platform.pulse.appearance.review_target",
            PlatformPulseProductComponent::ReviewTarget,
            UiAppearanceAspect::Background,
            PlatformPulsePaletteRole::ElevatedSurface,
        ),
        (
            "platform.pulse.appearance.review_label",
            PlatformPulseProductComponent::ReviewLabel,
            UiAppearanceAspect::Foreground,
            PlatformPulsePaletteRole::SecondaryText,
        ),
        (
            "platform.pulse.appearance.review_title",
            PlatformPulseProductComponent::ReviewTitle,
            UiAppearanceAspect::Foreground,
            PlatformPulsePaletteRole::PrimaryText,
        ),
        (
            "platform.pulse.appearance.review_body",
            PlatformPulseProductComponent::ReviewBody,
            UiAppearanceAspect::Foreground,
            PlatformPulsePaletteRole::SecondaryText,
        ),
        (
            "platform.pulse.appearance.review_cancel_target",
            PlatformPulseProductComponent::ReviewCancelTarget,
            UiAppearanceAspect::Background,
            PlatformPulsePaletteRole::RaisedSurface,
        ),
        (
            "platform.pulse.appearance.review_cancel_label",
            PlatformPulseProductComponent::ReviewCancelLabel,
            UiAppearanceAspect::Foreground,
            PlatformPulsePaletteRole::SecondaryText,
        ),
        (
            "platform.pulse.appearance.review_primary_target",
            PlatformPulseProductComponent::ReviewPrimaryTarget,
            UiAppearanceAspect::Background,
            PlatformPulsePaletteRole::ActionFill,
        ),
        (
            "platform.pulse.appearance.review_primary_label",
            PlatformPulseProductComponent::ReviewPrimaryLabel,
            UiAppearanceAspect::Foreground,
            PlatformPulsePaletteRole::ActionText,
        ),
    ] {
        builder = builder
            .register_appearance_role(component_role(
                identity,
                component,
                &[(aspect, palette.token_id())],
            ))
            .map_err(PlatformPulseAppearanceRegistrationDenial::Role)?;
    }
    let backdrop = UiAppearanceRole::authoring(
        UiAppearanceRoleIdentity::new("platform.pulse.appearance.modal_backdrop").unwrap(),
    )
    .applies_to_backdrop()
    .cover(
        UiAppearanceAspect::Background,
        UiAppearancePartitionAuthoring::new([]).with_cell(UiAppearanceCell::when([]).uses_slot(
            UiThemeSlotIdentity::new("theme.platform_pulse.modal_scrim").unwrap(),
            UiThemeValueKind::Color,
        )),
    )
    .unwrap()
    .cover(
        UiAppearanceAspect::Opacity,
        UiAppearancePartitionAuthoring::new([]).with_cell(UiAppearanceCell::when([]).uses_slot(
            UiThemeSlotIdentity::new("theme.platform_pulse.modal_opacity").unwrap(),
            UiThemeValueKind::Opacity,
        )),
    )
    .unwrap()
    .build()
    .unwrap();
    builder
        .register_appearance_role(backdrop)
        .map_err(PlatformPulseAppearanceRegistrationDenial::Role)
}
