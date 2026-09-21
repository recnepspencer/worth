//! Staging one Scroll owner's settle transition from an admitted coarse-wheel
//! observation.
//!
//! Under a declared smooth wheel the observation moves no accepted offset. It
//! advances the owner's semantic target and lowers that target into the Motion
//! request that walks the accepted sample there over what remains of the settle
//! horizon. Displayed truth stays with the accepted sample throughout; nothing
//! here samples, interpolates or paints.

use crate::runtime::scroll::transition::{
    scroll_settle_motion_request, UiScrollMotionBinding, UiScrollMotionRequestDenial,
    UiScrollTransitionDenial, UiScrollWheelInput, UiScrollWheelLineDelta,
    UI_SCROLL_WHEEL_LINE_MILLI_PER_LINE,
};

/// A `Line`-precision host delta carries lines at
/// [`worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT`]
/// subpixels per line, and the wheel accumulator counts thousandths of a line.
/// The two scales are the same number, so the delta already is a milli-line
/// count; this states that equality instead of assuming it.
const _: () = assert!(
    worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT
        == UI_SCROLL_WHEEL_LINE_MILLI_PER_LINE
);

/// One host wheel observation admitted as coarse-line evidence under a declared
/// smooth wheel, bound to the mounted occurrence and the presentation it was
/// observed against.
#[derive(Clone, Copy, Debug)]
pub(in crate::facade::entry) struct UiScrollWheelObservation {
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    lines: UiScrollWheelLineDelta,
    phase: worth_ui_host_contract::UiHostScrollDeltaPhase,
    input_tick: u64,
    settle_ticks: u32,
}

/// Why a coarse-wheel observation could not become a staged Scroll transition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::facade::entry) enum UiScrollTransitionStagingDenial {
    /// The routed receipt named no owner, so there is nothing to stage against.
    NoRoutedOwner,
    /// The latched owner is not a declared region occurrence, or its
    /// region-kind declares no line extent, so a notch has no travel here.
    OwnerDeclaresNoLineExtent,
    /// The routed state does not hold the latched owner under the incarnation
    /// the chain named, so there is no accepted offset to settle from.
    LatchedOwnerNotRouted(crate::runtime::scroll::UiScrollRouteDenial),
    /// The region occurrence has no mounted content box to translate.
    ContentGeometryUnavailable,
    Transition(UiScrollTransitionDenial),
    MotionRequest(UiScrollMotionRequestDenial),
    /// The staged settle could not become a publishable transition.
    Publishable(crate::runtime::scroll::UiScrollSettleTransitionDenial),
}

impl super::super::WorthUiActiveApplicationSession {
    /// The coarse-line evidence in one host scroll delta, when the declared
    /// wheel behaviour is smooth and the host reported the delta in lines.
    /// Anything else -- an immediate declared wheel, a pixel or page delta --
    /// has no transition to stage and stays on the direct offset path.
    pub(in crate::facade::entry) fn admit_smooth_wheel_observation(
        &self,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        phase: worth_ui_host_contract::UiHostScrollDeltaPhase,
        precision: worth_ui_host_contract::UiHostScrollDeltaPrecision,
        offset_delta: crate::runtime::scroll::UiScrollDelta,
        input_tick: u64,
    ) -> Option<UiScrollWheelObservation> {
        let settle_ticks = self
            .application
            .prepared_authority()
            .service_policy_plan()
            .scroll()?
            .wheel_behavior()
            .settle_ticks()?;
        precision.lines_per_notch()?;
        // `offset_delta` already travels in offset direction; the caller turned
        // the host sign once, and a `Line`-precision delta is a milli-line count.
        Some(UiScrollWheelObservation {
            mounted_instance,
            presentation,
            lines: UiScrollWheelLineDelta::new(
                offset_delta.inline_subpixels(),
                offset_delta.block_subpixels(),
            ),
            phase,
            input_tick,
            settle_ticks,
        })
    }

    /// Advance the settle target of the owner this gesture latched to, and
    /// lower that target into the Motion request that interpolates it.
    ///
    /// `receipt` is the zero-delta route this observation produced: it moved no
    /// offset, it only reconciled bounds and named the chain. The offset it
    /// reports for the innermost owner is therefore still the accepted one, and
    /// that is the basis this settle starts from.
    ///
    /// `scroll` is the routed successor that receipt came from, not the
    /// session's installed state: the target stages into the candidate, and
    /// the caller installs the candidate only once the settle is published.
    /// A settle that fails to publish therefore leaves no orphaned target.
    pub(in crate::facade::entry) fn stage_scroll_transition(
        &self,
        scroll: &mut crate::runtime::scroll::UiScrollRuntimeState,
        receipt: &crate::runtime::scroll::UiScrollRouteReceipt,
        observation: UiScrollWheelObservation,
        region: Option<super::scroll_gesture_latching::UiScrollRoutedRegion>,
    ) -> Result<
        crate::runtime::scroll::UiPreparedScrollSettleTransition,
        UiScrollTransitionStagingDenial,
    > {
        let region = region.ok_or(UiScrollTransitionStagingDenial::NoRoutedOwner)?;
        // The owner comes from the chain, and its accepted offset from the
        // successor the route produced. Neither can come from `receipt`: a
        // receipt names only the owners the route had travel left to visit,
        // and the zero delta a smooth wheel routes stops at the first of them,
        // so the owner a gesture latched outward to is routinely absent from
        // it. The successor holds every owner's offset whether the route
        // touched it or not, and for an owner it did touch that offset is the
        // one the receipt would have reported.
        let entry = region.entry();
        let owner = entry.owner();
        let incarnation = entry.incarnation();
        let accepted_offset = scroll
            .offset(owner, incarnation)
            .map_err(UiScrollTransitionStagingDenial::LatchedOwnerNotRouted)?;
        let line_extent_logical_points = self
            .declared_scroll_line_extent_logical_points(owner)
            .ok_or(UiScrollTransitionStagingDenial::OwnerDeclaresNoLineExtent)?;
        let content_at_rest =
            self.unscrolled_region_content(observation.mounted_instance, region.slot())?;
        let input = UiScrollWheelInput::admit(
            observation.lines,
            observation.phase,
            observation.input_tick,
            line_extent_logical_points,
            observation.settle_ticks,
        )
        .map_err(UiScrollTransitionStagingDenial::Transition)?;
        // A target whose horizon has ended is no intention anyone is settling
        // toward any more; it goes before this notch decides what it
        // accumulates against.
        scroll.advance_transitions(observation.input_tick);
        let target = scroll
            .stage_wheel_transition(
                crate::runtime::scroll::UiScrollChainEntry::new(owner, incarnation),
                input,
            )
            .map_err(UiScrollTransitionStagingDenial::Transition)?;
        let request = scroll_settle_motion_request(
            target,
            accepted_offset,
            observation.input_tick,
            UiScrollMotionBinding::new(
                observation.mounted_instance,
                scroll_motion_owner_key(owner),
                observation.input_tick,
                observation.input_tick.saturating_add(1),
                observation.presentation,
                content_at_rest,
            ),
        )
        .map_err(UiScrollTransitionStagingDenial::MotionRequest)?;
        crate::runtime::scroll::UiPreparedScrollSettleTransition::prepare(
            target,
            observation.mounted_instance,
            request,
            receipt.revision(),
        )
        .map_err(UiScrollTransitionStagingDenial::Publishable)
    }

    /// The region's content box where an offset of zero places it. The owner
    /// box is that box: an applied pose translates the owner's descendants and
    /// leaves the owner where layout put it, so the Motion request that
    /// translates from rest composes with it directly.
    fn unscrolled_region_content(
        &self,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        slot: usize,
    ) -> Result<worth_ui_host_contract::UiMountedCanonicalBox, UiScrollTransitionStagingDenial>
    {
        let (_, content, _) = self
            .mounted
            .scroll_region_geometry(mounted_instance, slot)
            .ok_or(UiScrollTransitionStagingDenial::ContentGeometryUnavailable)?;
        Ok(content)
    }
}

/// The Motion owner key for a Scroll region occurrence. The plan index names
/// the exact occurrence, which is what separates two Scroll regions bound to
/// one mounted instance. A surface or viewport owner never reaches here: it
/// declares no line extent and is refused before staging.
pub(in crate::facade::entry) fn scroll_motion_owner_key(
    owner: crate::runtime::scroll::UiScrollOwnerIdentity,
) -> u64 {
    match owner {
        crate::runtime::scroll::UiScrollOwnerIdentity::Region {
            plan_region_index, ..
        } => u64::from(plan_region_index),
        crate::runtime::scroll::UiScrollOwnerIdentity::Surface(_)
        | crate::runtime::scroll::UiScrollOwnerIdentity::Viewport(_) => 0,
    }
}
