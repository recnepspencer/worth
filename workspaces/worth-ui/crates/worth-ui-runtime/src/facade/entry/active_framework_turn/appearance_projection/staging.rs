pub(super) fn stage_denial(
    frame: &mut crate::mounting::UiPreparedMountedFrame,
    context: crate::runtime::appearance::UiAppearanceAttemptContext,
    denial: crate::runtime::appearance::UiAppearanceInspectionDenial,
) -> Result<(), crate::mounting::UiMountedFramePreparationDenial> {
    frame
        .stage_appearance_projection(
            crate::runtime::appearance::UiAppearanceProjectionAttempt::denied(context, denial),
        )
        .map_err(map_appearance_state_denial)
}

pub(super) fn map_appearance_state_denial(
    denial: crate::mounting::UiMountedAppearanceStateMutationDenial,
) -> crate::mounting::UiMountedFramePreparationDenial {
    match denial {
        crate::mounting::UiMountedAppearanceStateMutationDenial::Capacity(error) => {
            crate::mounting::UiMountedFramePreparationDenial::AppearanceStateCapacityExceeded(error)
        }
        crate::mounting::UiMountedAppearanceStateMutationDenial::LocalIdentityMismatch => {
            crate::mounting::UiMountedFramePreparationDenial::AppearanceStateIdentityMismatch
        }
    }
}
