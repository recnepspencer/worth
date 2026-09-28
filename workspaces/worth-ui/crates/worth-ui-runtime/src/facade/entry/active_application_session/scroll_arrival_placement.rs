//! Placing a settle whose arrival a relayout pulled back.
//!
//! A frame prepared before a settle's Motion arrived places the group where
//! its layout was lowered, and once that frame lands the host draws the
//! content there, although the reader last saw it arrive at the settle's
//! target. A sample no track stands behind is that arrival, so the target is
//! where the reader last saw the content. No track is left to carry it on,
//! so the settle ends as a page ends one: placed at its target, as far as
//! the extent the frame published reaches, and the next frame lands the
//! content where the reader saw it arrive.

use crate::mounting::WorthUiMountedSessionState;
use crate::runtime::motion::UiMotionTargetIdentity;
use crate::runtime::scroll::{
    UiScrollChainEntry, UiScrollDelta, UiScrollDeltaCause, UiScrollDeltaRequest,
    UiScrollOwnerIdentity, UiScrollRuntimeState,
};

/// Stage `owner`'s settle, whose arrival `target` showed, as a direct
/// placement at its target. `false` when the placement moves nothing or
/// cannot stage, which leaves the settle for the caller to end.
pub(super) fn place_pulled_back_arrival(
    scroll: &mut UiScrollRuntimeState,
    mounted: &mut WorthUiMountedSessionState,
    owner: UiScrollOwnerIdentity,
    target: UiMotionTargetIdentity,
) -> bool {
    let occurrence = target.mounted_instance();
    let Some(slot) = scroll.ownership_chain(occurrence).ok().and_then(|chain| {
        chain
            .owners()
            .iter()
            .position(|candidate| *candidate == owner)
    }) else {
        return false;
    };
    let (Some(incarnation), Some((owner_instance, region))) = (
        mounted.scroll_region_incarnation(occurrence, slot),
        mounted.scroll_region_geometry(occurrence, slot),
    ) else {
        return false;
    };
    let Some(arrived) = scroll
        .transition_target(owner, incarnation)
        .map(|pending| pending.target_offset())
    else {
        return false;
    };
    let entry = UiScrollChainEntry::new(owner, incarnation);
    let Ok(mut successor) = scroll.route_candidate(&[entry], true) else {
        return false;
    };
    let Ok(current) = successor.state().offset(owner, incarnation) else {
        return false;
    };
    let (Some(inline), Some(block), Some(bounds)) = (
        arrived
            .inline_subpixels()
            .checked_sub(current.inline_subpixels()),
        arrived
            .block_subpixels()
            .checked_sub(current.block_subpixels()),
        region.in_layout_space().bounds(),
    ) else {
        return false;
    };
    let Ok(request) = UiScrollDeltaRequest::new(
        vec![entry],
        UiScrollDelta::new(inline, block),
        UiScrollDeltaCause::AcceptedSampleSettlement,
    ) else {
        return false;
    };
    successor.state_mut().retire_transition(owner);
    let Ok(receipt) = successor
        .state_mut()
        .route_with_reconciled_bounds(request, &[bounds])
    else {
        return false;
    };
    if successor
        .state()
        .offset(owner, incarnation)
        .is_ok_and(|placed| placed == current)
    {
        return false;
    }
    let prepared =
        successor
            .state()
            .prepare_direct_succession(&receipt, occurrence, &[Some(owner_instance)]);
    if mounted.stage_direct_scroll_geometry(&prepared).is_err() {
        return false;
    }
    scroll.stage_direct_succession(successor.state(), &prepared);
    true
}
