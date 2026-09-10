pub(super) struct ResolvedNodeStyle {
    pub(super) radii: worth_ui_host_contract::UiAppearanceNormalizedLogicalRadii,
    pub(super) fill: Option<worth_ui_host_contract::UiMountedAppearanceColor>,
    pub(super) border: Option<(
        worth_ui_host_contract::UiMountedAppearanceColor,
        worth_ui_host_contract::UiAppearanceLogicalLength,
    )>,
    pub(super) outline: Option<worth_ui_dsl::UiThemeOutline>,
    pub(super) foreground: Option<worth_ui_host_contract::UiMountedAppearanceColor>,
    pub(super) opacity: worth_ui_host_contract::UiMountedAppearanceOpacity,
}

pub(super) fn resolve(
    bounds: worth_ui_host_contract::UiAppearanceAllocationBounds,
    projection: &crate::runtime::appearance::UiAppearanceProjection,
) -> Result<ResolvedNodeStyle, super::UiMountedAppearanceLoweringDenial> {
    let mut style = ResolvedNodeStyle {
        radii: worth_ui_host_contract::UiAppearanceNormalizedLogicalRadii::normalize(
            bounds,
            [worth_ui_host_contract::UiAppearanceLogicalLength::ZERO; 4],
        ),
        fill: None,
        border: None,
        outline: None,
        foreground: None,
        opacity: worth_ui_host_contract::UiMountedAppearanceOpacity::ONE,
    };
    for aspect in projection.aspects() {
        if aspect.support() != crate::runtime::appearance::UiAppearanceSupportPosture::Supported {
            continue;
        }
        match (aspect.aspect(), aspect.value()) {
            (
                worth_ui_dsl::UiAppearanceAspect::Foreground,
                worth_ui_dsl::UiThemeValue::Color(color),
            ) => style.foreground = Some(mounted_color(color)),
            (
                worth_ui_dsl::UiAppearanceAspect::Background,
                worth_ui_dsl::UiThemeValue::Color(color),
            ) => style.fill = Some(mounted_color(color)),
            (
                worth_ui_dsl::UiAppearanceAspect::Border,
                worth_ui_dsl::UiThemeValue::SolidStroke(stroke),
            ) => style.border = Some(resolved_border(stroke)?),
            (
                worth_ui_dsl::UiAppearanceAspect::Radius,
                worth_ui_dsl::UiThemeValue::CornerRadii(authored),
            ) => style.radii = resolved_radii(bounds, authored)?,
            (
                worth_ui_dsl::UiAppearanceAspect::Opacity,
                worth_ui_dsl::UiThemeValue::Opacity(opacity),
            ) => {
                style.opacity =
                    worth_ui_host_contract::UiMountedAppearanceOpacity::from_units(opacity.units())
            }
            (
                worth_ui_dsl::UiAppearanceAspect::Outline,
                worth_ui_dsl::UiThemeValue::SolidOutline(authored),
            ) => style.outline = Some(authored),
            _ => {}
        }
    }
    Ok(style)
}

pub(in crate::mounting::projection) fn resolved_opacity(
    projection: &crate::runtime::appearance::UiAppearanceProjection,
) -> worth_ui_host_contract::UiMountedAppearanceOpacity {
    projection
        .aspects()
        .iter()
        .find_map(|aspect| {
            (aspect.support() == crate::runtime::appearance::UiAppearanceSupportPosture::Supported)
                .then(|| match (aspect.aspect(), aspect.value()) {
                    (
                        worth_ui_dsl::UiAppearanceAspect::Opacity,
                        worth_ui_dsl::UiThemeValue::Opacity(opacity),
                    ) => Some(
                        worth_ui_host_contract::UiMountedAppearanceOpacity::from_units(
                            opacity.units(),
                        ),
                    ),
                    _ => None,
                })
                .flatten()
        })
        .unwrap_or(worth_ui_host_contract::UiMountedAppearanceOpacity::ONE)
}

fn resolved_border(
    stroke: worth_ui_dsl::UiThemeSolidStroke,
) -> Result<
    (
        worth_ui_host_contract::UiMountedAppearanceColor,
        worth_ui_host_contract::UiAppearanceLogicalLength,
    ),
    super::UiMountedAppearanceLoweringDenial,
> {
    let width = worth_ui_host_contract::UiAppearanceLogicalLength::new(stroke.width().subpixels())
        .map_err(|_| super::UiMountedAppearanceLoweringDenial::BorderWidthInvalid)?;
    Ok((mounted_color(stroke.color()), width))
}

fn resolved_radii(
    bounds: worth_ui_host_contract::UiAppearanceAllocationBounds,
    authored: worth_ui_dsl::UiThemeCornerRadii,
) -> Result<
    worth_ui_host_contract::UiAppearanceNormalizedLogicalRadii,
    super::UiMountedAppearanceLoweringDenial,
> {
    let authored = authored
        .corners()
        .map(|length| worth_ui_host_contract::UiAppearanceLogicalLength::new(length.subpixels()))
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| super::UiMountedAppearanceLoweringDenial::RadiusInvalid)?;
    Ok(
        worth_ui_host_contract::UiAppearanceNormalizedLogicalRadii::normalize(
            bounds,
            [authored[0], authored[1], authored[2], authored[3]],
        ),
    )
}

pub(super) fn mounted_color(
    color: worth_ui_dsl::UiThemeColor,
) -> worth_ui_host_contract::UiMountedAppearanceColor {
    worth_ui_host_contract::UiMountedAppearanceColor::from_straight_srgba(color.channels())
}
