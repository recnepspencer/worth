use super::{UiInteractionTargetingDenial, UiPresentedInteractionTargetView};

/// Refreshes an already selected pointer target from its exact indexed row.
/// Neighborhood selection remains with the pointer owner; this is not hit testing.
pub(crate) fn refresh_pointer_target(
    mounted: &crate::mounting::WorthUiMountedSessionState,
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    target: UiPresentedInteractionTargetView,
    work: &mut crate::mounting::UiHitTestSpatialWork,
) -> Result<UiPresentedInteractionTargetView, UiInteractionTargetingDenial> {
    let row = mounted
        .current_presented_hit_row(presentation, target.mounted_instance(), work)
        .map_err(|denial| match denial {
            crate::mounting::UiPresentedFrameBasisDenial::InstanceNotPresented => {
                UiInteractionTargetingDenial::GraphTargetNotPresented
            }
            denial => super::presented_frame::map_presentation_denial(denial),
        })?;
    let current = mounted
        .admit_current_hit_target(row.mounted())
        .map_err(super::map_current_affinity_denial)?;
    Ok(super::presented_target::seal_target(
        presentation,
        super::UiPresentedTargetFrameRelation::Current,
        current,
        row,
        0,
    )
    .view())
}
