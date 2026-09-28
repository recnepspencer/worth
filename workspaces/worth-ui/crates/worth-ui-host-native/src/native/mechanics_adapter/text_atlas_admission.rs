//! Effect-free native capacity and pin admission for text-atlas work.

use worth_ui_host_contract::{UiGlyphRasterPinTransitionView, UiGlyphRasterTransactionDenial};

use crate::native::text_atlas::{
    UiNativeGpuAtlasKind, UiNativeTextAtlasGpuPages, UiNativeTextAtlasPinRequest,
    UiNativeTextAtlasPinTransition, UiNativeTextAtlasTransactionPlan,
};
use crate::native::UiNativeHostState;

pub(super) fn physical_capacity_denial(
    state: &UiNativeHostState,
    plan: &UiNativeTextAtlasTransactionPlan,
) -> Option<UiGlyphRasterTransactionDenial> {
    let pending_staging = state
        .text_atlas_gpu
        .as_ref()
        .map_or(0, UiNativeTextAtlasGpuPages::pending_physical_bytes);
    if pending_staging
        .checked_add(plan.physical_staged_bytes())
        .is_none_or(|peak| peak > 8 * 1_024 * 1_024)
    {
        return Some(UiGlyphRasterTransactionDenial::CapacityExceeded);
    }
    state.presentation_access()?;
    let current = state.text_atlas_gpu.as_ref().map_or((0, 0), |gpu| {
        (
            gpu.page_count(UiNativeGpuAtlasKind::Alpha),
            gpu.page_count(UiNativeGpuAtlasKind::Color),
        )
    });
    let needed = plan.candidate_page_counts();
    let additional_pages = needed
        .0
        .saturating_sub(current.0)
        .saturating_add(needed.1.saturating_sub(current.1));
    let staging_owners = usize::from(!plan.miss_demands().is_empty());
    (!state
        .resources
        .admits(additional_pages.saturating_add(staging_owners)))
    .then_some(UiGlyphRasterTransactionDenial::CapacityExceeded)
}

pub(super) fn native_pin_transition(
    pins: UiGlyphRasterPinTransitionView<'_>,
) -> UiNativeTextAtlasPinTransition {
    worth_ui_host_contract::record_presentation_glyphs(
        worth_ui_host_contract::UiPresentationWorkStage::Pins,
        pins.additions().len() + pins.releases().len(),
    );
    UiNativeTextAtlasPinTransition::from_text_mechanics(
        pins.additions().iter().copied().map(|pin| {
            UiNativeTextAtlasPinRequest::from_text_mechanics(pin.layout_identity(), pin.key())
        }),
        pins.releases().iter().copied().map(|pin| {
            UiNativeTextAtlasPinRequest::from_text_mechanics(pin.layout_identity(), pin.key())
        }),
    )
}
