use super::targeting::UiInteractionTargetingDenial;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiPointerPresentationRefreshReport {
    pub(crate) retested: usize,
    pub(crate) evidence_refreshed: usize,
    pub(crate) changed: usize,
    pub(crate) unmatched: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiInteractionPresentationRefreshSnapshot {
    pub(crate) comparison_steps: usize,
    pub(crate) comparison_key_probes: usize,
    pub(crate) hover_neighborhood_work: crate::mounting::UiHitTestSpatialWork,
    pub(crate) pressed_neighborhood_work: crate::mounting::UiHitTestSpatialWork,
    pub(crate) hover_retry: bool,
    pub(crate) pressed_retry: bool,
    pub(crate) hover: Result<UiPointerPresentationRefreshReport, UiInteractionTargetingDenial>,
    pub(crate) pressed: Result<UiPointerPresentationRefreshReport, UiInteractionTargetingDenial>,
}

pub(super) fn affected(
    changes: &crate::mounting::UiPresentedHitChanges,
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    position: worth_ui_host_contract::UiHostSurfacePosition,
    target: Option<worth_ui_host_contract::UiMountedInstanceIdentity>,
    work: &mut crate::mounting::UiHitTestSpatialWork,
) -> Result<bool, UiInteractionTargetingDenial> {
    let scale = worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
    // Match targeting's canonical f32 point and half-open endpoints exactly.
    let point = [
        (position.x_subpixels() as f64 / scale) as f32,
        (position.y_subpixels() as f64 / scale) as f32,
    ];
    let (affected, query_work) = changes
        .affects(presentation.binding(), point.map(f64::from), target)
        .map_err(|denial| {
            match &denial {
                crate::mounting::UiPresentedHitQueryDenial::NodeBudget { work: failed }
                | crate::mounting::UiPresentedHitQueryDenial::CandidateBudget { work: failed } => {
                    work.merge(*failed)
                }
                _ => {}
            }
            super::targeting::map_hit_query_denial(denial)
        })?;
    work.merge(query_work);
    Ok(affected)
}
