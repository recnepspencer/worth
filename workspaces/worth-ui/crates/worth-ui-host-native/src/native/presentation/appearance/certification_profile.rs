use crate::native_profile::STAGED_APPEARANCE_PROFILE;

pub(crate) const STAGED_APPEARANCE_MECHANICS:
    [worth_ui_host_contract::UiHostAppearanceMechanicFamily; 11] = [
    worth_ui_host_contract::UiHostAppearanceMechanicFamily::SurfaceFill,
    worth_ui_host_contract::UiHostAppearanceMechanicFamily::SurfaceBorder,
    worth_ui_host_contract::UiHostAppearanceMechanicFamily::CornerRadii,
    worth_ui_host_contract::UiHostAppearanceMechanicFamily::Outline,
    worth_ui_host_contract::UiHostAppearanceMechanicFamily::TextRangeForeground,
    worth_ui_host_contract::UiHostAppearanceMechanicFamily::PortalSurface,
    worth_ui_host_contract::UiHostAppearanceMechanicFamily::Backdrop,
    worth_ui_host_contract::UiHostAppearanceMechanicFamily::OverlayOrder,
    worth_ui_host_contract::UiHostAppearanceMechanicFamily::PointerAffordance,
    worth_ui_host_contract::UiHostAppearanceMechanicFamily::Damage,
    worth_ui_host_contract::UiHostAppearanceMechanicFamily::Clip,
];

pub(crate) fn staged_appearance_profile_contract(
) -> worth_ui_host_contract::UiHostAppearanceProfileContract {
    worth_ui_host_contract::UiHostAppearanceProfileContract::admit(
        STAGED_APPEARANCE_PROFILE.identity,
        STAGED_APPEARANCE_PROFILE.version,
        STAGED_APPEARANCE_MECHANICS,
        STAGED_APPEARANCE_PROFILE.primary_pointer,
    )
    .expect("the qualified staged native appearance profile must admit")
}
