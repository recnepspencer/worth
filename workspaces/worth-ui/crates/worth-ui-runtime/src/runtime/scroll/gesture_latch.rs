//! The Scroll owner a scroll gesture is latched to, and how long that lasts.
//!
//! Resolving the owner chain from pointer location is right for the first
//! event of a gesture and wrong for every one after it. An inner list that
//! reaches its edge mid-flick would hand the rest of the same physical gesture
//! to the panel behind it, and the reader would watch something they never
//! aimed at take over. The latch is what stops that: the owner that first
//! consumed the gesture keeps it until the gesture ends, and travel that owner
//! can no longer consume is discarded rather than bubbled outward.
//!
//! Two lifetimes, because the host reports two kinds of scroll input. A phased
//! gesture states its own end, so its latch lives from `Started` to `Ended` or
//! `Cancelled`, platform momentum included. An unphased coarse wheel never
//! ends; it goes quiet. Its latch therefore outlives each event by the declared
//! settle horizon -- while the content that notch started is still settling,
//! the same owner keeps the wheel -- and pointer location decides again once
//! the content is still.
//!
//! A latch is a pointer at authority, never authority itself. Holding one moves
//! no offset and mints no capture; it only names which owner the next event of
//! the same gesture belongs to, and an owner that has been removed or
//! reincarnated leaves a latch that names nothing.

use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSurfaceBindingGeneration};

/// How long a latch lives once it is taken.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollGestureLatchLifetime {
    /// A phased gesture: the host says when it ends, so nothing else may.
    PhasedGesture,
    /// An unphased coarse wheel: the latch outlives each consumed event by the
    /// declared settle horizon and no longer.
    QuietInterval { quiet_ticks: u32 },
}

/// One held latch: the exact owner occurrence holding the gesture, where the
/// chain that names it can be found again, and what ends it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollGestureLatch {
    owner: super::UiScrollOwnerIdentity,
    incarnation: super::UiScrollOwnerIncarnation,
    mounted_instance: UiMountedInstanceIdentity,
    slot: usize,
    binding: UiSurfaceBindingGeneration,
    lifetime: UiScrollGestureLatchLifetime,
    latest_input_tick: u64,
}

impl UiScrollGestureLatch {
    pub(crate) const fn new(
        owner: super::UiScrollOwnerIdentity,
        incarnation: super::UiScrollOwnerIncarnation,
        mounted_instance: UiMountedInstanceIdentity,
        slot: usize,
        binding: UiSurfaceBindingGeneration,
        lifetime: UiScrollGestureLatchLifetime,
        latest_input_tick: u64,
    ) -> Self {
        Self {
            owner,
            incarnation,
            mounted_instance,
            slot,
            binding,
            lifetime,
            latest_input_tick,
        }
    }

    pub(crate) const fn mounted_instance(self) -> UiMountedInstanceIdentity {
        self.mounted_instance
    }

    pub(crate) const fn slot(self) -> usize {
        self.slot
    }

    pub(crate) const fn binding(self) -> UiSurfaceBindingGeneration {
        self.binding
    }

    /// The same latch, carried forward to the event that just consumed on it.
    /// A phased gesture does not age, so only the quiet interval moves.
    pub(crate) const fn refreshed(self, input_tick: u64) -> Self {
        Self {
            latest_input_tick: input_tick,
            ..self
        }
    }

    /// Whether the latch still holds at `tick`. A phased gesture holds until
    /// the host ends it; an unphased wheel holds while its content is still
    /// settling.
    pub(crate) fn is_live_at(self, tick: u64) -> bool {
        match self.lifetime {
            UiScrollGestureLatchLifetime::PhasedGesture => true,
            UiScrollGestureLatchLifetime::QuietInterval { quiet_ticks } => {
                tick <= self
                    .latest_input_tick
                    .saturating_add(u64::from(quiet_ticks))
            }
        }
    }

    /// Whether this latch names exactly this owner occurrence. A reincarnated
    /// owner is a different occurrence and inherits nothing.
    pub(crate) fn binds(
        self,
        owner: super::UiScrollOwnerIdentity,
        incarnation: super::UiScrollOwnerIncarnation,
    ) -> bool {
        self.owner == owner && self.incarnation == incarnation
    }
}

/// The chain position a routed gesture latches to: the innermost owner that
/// could consume travel in this gesture's direction, judged at the offset each
/// owner held before the route.
///
/// Asking what an owner *could* consume rather than what it did is what lets
/// one rule serve both wheels. An immediate wheel moves the offset, so the two
/// answers coincide. A declared smooth wheel routes a zero delta -- the notch
/// becomes a settle target, not an offset -- and moved nothing anywhere, so
/// only the question about capacity has an answer at all.
///
/// A gesture that starts at an inner edge finds that owner unable to consume
/// and walks outward to the first ancestor that can, which is the owner the
/// reader will see move. A gesture no owner in the chain can consume latches
/// to nothing: there is no owner to name.
///
/// The offsets are the whole chain's, read before the route rather than taken
/// from what the route touched, and paired with the bounds that route
/// reconciled. A route stops at the owner that leaves nothing over, so a zero
/// delta visits exactly one owner -- which is what a declared smooth wheel
/// routes. Judging a latch from the visited owners would therefore pin every
/// smooth notch to the innermost one, including the one at its edge that the
/// paragraph above says has to hand the gesture outward.
pub(crate) fn latching_chain_index(
    offsets: &[super::UiScrollOffset],
    bounds: &[super::UiScrollBounds],
    delta: super::UiScrollDelta,
) -> Option<usize> {
    offsets.iter().zip(bounds).position(|(offset, bounds)| {
        has_room(
            delta.inline_subpixels(),
            offset.inline_subpixels(),
            bounds.max_inline_subpixels(),
        ) || has_room(
            delta.block_subpixels(),
            offset.block_subpixels(),
            bounds.max_block_subpixels(),
        )
    })
}

/// Whether one axis has travel left in the direction `delta` asks for. An axis
/// the gesture does not name asks for nothing and answers no.
const fn has_room(delta: i64, offset: i64, max: i64) -> bool {
    (delta > 0 && offset < max) || (delta < 0 && offset > 0)
}
