pub(crate) fn for_aspect(
    aspect: worth_ui_dsl::UiAppearanceAspect,
    theme: &super::super::super::theme::UiThemeResolutionView,
) -> super::super::UiAppearanceSupportPosture {
    let mechanic = match aspect {
        worth_ui_dsl::UiAppearanceAspect::Background
        | worth_ui_dsl::UiAppearanceAspect::Opacity => {
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::SurfaceFill
        }
        worth_ui_dsl::UiAppearanceAspect::Foreground => {
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::TextRangeForeground
        }
        worth_ui_dsl::UiAppearanceAspect::Border => {
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::SurfaceBorder
        }
        worth_ui_dsl::UiAppearanceAspect::Radius => {
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::CornerRadii
        }
        worth_ui_dsl::UiAppearanceAspect::Outline => {
            worth_ui_host_contract::UiHostAppearanceMechanicFamily::Outline
        }
    };
    if theme.supports(mechanic) {
        super::super::UiAppearanceSupportPosture::Supported
    } else {
        super::super::UiAppearanceSupportPosture::Unsupported
    }
}
