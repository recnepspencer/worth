use crate::native_profile::STAGED_APPEARANCE_PROFILE;

fn staged_geometry_qualification() -> worth_ui_host_contract::UiHostAppearanceGeometryQualification
{
    let rows = STAGED_APPEARANCE_PROFILE.scales_milli.iter().map(|scale| {
        let scale_milli = u32::from(*scale);
        let scale = u64::from(scale_milli);
        let numerator = u64::from(STAGED_APPEARANCE_PROFILE.anti_alias_fringe_physical_pixels)
            .checked_mul(u64::from(
                worth_ui_host_contract::UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT,
            ))
            .and_then(|value| value.checked_mul(1_000))
            .expect("the staged geometry qualification numerator must fit");
        let logical_subpixels = numerator
            .checked_add(scale - 1)
            .expect("the staged geometry qualification ceiling must fit")
            / scale;
        let logical_subpixels = i32::try_from(logical_subpixels)
            .expect("the staged logical fringe enclosure must fit the contract");
        worth_ui_host_contract::UiHostAppearanceScaleGeometryQualification::new(
            scale_milli,
            u32::from(STAGED_APPEARANCE_PROFILE.anti_alias_fringe_physical_pixels),
            worth_ui_host_contract::UiAppearanceLogicalLength::new(logical_subpixels)
                .expect("the staged logical fringe enclosure must be nonnegative"),
            STAGED_APPEARANCE_PROFILE.geometry_basis,
        )
    });
    worth_ui_host_contract::UiHostAppearanceGeometryQualification::admit(rows)
        .expect("the qualified staged native geometry rows must admit")
}

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
        staged_geometry_qualification(),
    )
    .expect("the qualified staged native appearance profile must admit")
}
