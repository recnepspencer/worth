//! Which Scroll owner a scroll gesture belongs to, for as long as it lasts.
//!
//! Pointer location answers this for the first event of a gesture and for no
//! event after it. An inner list that reaches its edge halfway through a flick
//! would hand the rest of one physical gesture to whatever sits behind it, and
//! the reader would watch a panel they never aimed at take over. So the owner
//! that first has room for the gesture takes it, and keeps it: travel it can no
//! longer consume is discarded rather than bubbled outward, and the pointer may
//! wander off the owner or off the surface without moving the gesture.
//!
//! The latch is resolved here and held next door in Interaction, because its
//! lifetime is a lifecycle question rather than a scrolling one. Resolution is
//! a read: nothing in this file moves an offset, mints a capture or installs a
//! successor. The latch is taken only after the route it describes has been
//! committed, so a refused route leaves the previous gesture's latch untouched.
//!
//! Taking and releasing are not symmetric, and the asymmetry is the point.
//! Taking a latch is a claim about an owner that moved, so it waits for the
//! movement. Releasing one is a report that the gesture is over, which the host
//! makes and the runtime does not get to disagree with: the release therefore
//! happens on the phase alone, even when the report carrying it was refused.

use super::super::WorthUiActiveApplicationSession;
use crate::runtime::scroll::{
    UiHostScrollObservationDenial, UiScrollChainEntry, UiScrollGestureLatch,
    UiScrollGestureLatchLifetime,
};

/// One region of a routed chain: the exact owner occurrence at that position,
/// and the mounted geometry slot its boxes are read from.
///
/// The owner travels here rather than the position it was found at. A caller
/// holding a position would have to know which list to spend it on, and the
/// two candidate lists are different lengths: the chain names every owner the
/// gesture could reach, while a route receipt names only the owners that had
/// travel left to visit. A zero-delta route visits one owner whatever the
/// chain's depth, so the two disagree on exactly the gesture a declared smooth
/// wheel routes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::facade::entry) struct UiScrollRoutedRegion {
    entry: UiScrollChainEntry,
    slot: usize,
}

impl UiScrollRoutedRegion {
    pub(in crate::facade::entry) const fn entry(self) -> UiScrollChainEntry {
        self.entry
    }

    pub(in crate::facade::entry) const fn slot(self) -> usize {
        self.slot
    }
}

/// The owner chain one scroll observation routes through, and the mounted
/// occurrence its geometry is read from.
///
/// A chain resolved from the pointer is the full ownership chain, innermost
/// first, so remainder bubbles outward the way an unlatched gesture should. A
/// latched chain is exactly one owner: bubbling is what the latch exists to
/// prevent, and a chain of one cannot bubble.
pub(in crate::facade::entry) struct UiScrollRoutedChain {
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    graph_node: crate::graph::UiGraphNodeIdentity,
    entries: Vec<UiScrollChainEntry>,
    slots: Vec<usize>,
}

impl UiScrollRoutedChain {
    pub(in crate::facade::entry) const fn mounted_instance(
        &self,
    ) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.mounted_instance
    }

    pub(in crate::facade::entry) const fn graph_node(&self) -> crate::graph::UiGraphNodeIdentity {
        self.graph_node
    }

    pub(in crate::facade::entry) fn entries(&self) -> &[UiScrollChainEntry] {
        &self.entries
    }

    pub(in crate::facade::entry) fn slots(&self) -> &[usize] {
        &self.slots
    }

    /// The region at `chain_index`, when the chain reaches that far. The two
    /// lists are filled together, one pair per owner, so a chain that reaches
    /// the index has both halves of the answer.
    pub(in crate::facade::entry) fn region(
        &self,
        chain_index: usize,
    ) -> Option<UiScrollRoutedRegion> {
        Some(UiScrollRoutedRegion {
            entry: *self.entries.get(chain_index)?,
            slot: *self.slots.get(chain_index)?,
        })
    }
}

impl WorthUiActiveApplicationSession {
    /// The chain this observation routes through: the latched owner if a
    /// gesture still holds one, and otherwise the ownership chain under the
    /// pointer.
    ///
    /// A live latch answers before the pointer is consulted, so a gesture whose
    /// pointer has left the owner, or whose host reports only a surface it
    /// cannot disambiguate, still reaches the owner it started on. The
    /// presentation is checked either way: a latch names an owner, never a
    /// stale frame to route against.
    ///
    /// The frame is asked once more whether it still admits input to that
    /// owner. Skipping the pointer is what the latch is for, but the pointer
    /// path is also where modal shielding is applied, so a latch that answered
    /// unconditionally would be the one way a reader keeps scrolling content a
    /// modal has since closed over.
    pub(in crate::facade::entry) fn resolve_scroll_routing(
        &self,
        target: worth_ui_host_contract::UiHostScrollDeltaTargetAffinity,
        work: &mut crate::mounting::UiHitTestSpatialWork,
        observation_tick: Option<u64>,
    ) -> Result<UiScrollRoutedChain, UiHostScrollObservationDenial> {
        if let Some(chain) = self.latched_scroll_chain(observation_tick) {
            crate::runtime::interaction::targeting::require_current_presentation(
                &self.mounted,
                target.presentation(),
            )
            .map_err(UiHostScrollObservationDenial::Targeting)?;
            self.mounted
                .current_presented_hit_row(target.presentation(), chain.mounted_instance(), work)
                .map_err(UiHostScrollObservationDenial::LatchedOwnerNotAdmitted)?;
            return Ok(chain);
        }
        let (mounted_instance, mounted) = self.resolve_scroll_target(target, work)?;
        self.pointer_scroll_chain(mounted_instance, mounted.graph_node_identity())
    }

    /// The one-owner chain a held latch names, when the owner it names is still
    /// the occurrence it latched to.
    ///
    /// An owner that was unmounted, or that came back as a new incarnation,
    /// leaves a latch pointing at nothing. Rather than deny the event, the
    /// gesture falls back to the pointer: the reader is still scrolling, and
    /// the thing they were scrolling is gone.
    fn latched_scroll_chain(&self, observation_tick: Option<u64>) -> Option<UiScrollRoutedChain> {
        let latch = self
            .interaction
            .scroll_gesture_latch_at(observation_tick?)?;
        let mounted_instance = latch.mounted_instance();
        let graph_node = self
            .mounted
            .current_mounted_identity_basis(mounted_instance)?
            .graph_node_identity();
        let owner = *self
            .scroll
            .as_ref()?
            .ownership_chain(mounted_instance)
            .ok()?
            .owners()
            .get(latch.slot())?;
        let incarnation =
            self.scroll_owner_chain_incarnation(owner, mounted_instance, latch.slot())?;
        latch
            .binds(owner, incarnation)
            .then(|| UiScrollRoutedChain {
                mounted_instance,
                graph_node,
                entries: vec![UiScrollChainEntry::new(owner, incarnation)],
                slots: vec![latch.slot()],
            })
    }

    /// The full ownership chain under a mounted occurrence, innermost first.
    fn pointer_scroll_chain(
        &self,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        graph_node: crate::graph::UiGraphNodeIdentity,
    ) -> Result<UiScrollRoutedChain, UiHostScrollObservationDenial> {
        let scroll = self
            .scroll
            .as_ref()
            .ok_or(UiHostScrollObservationDenial::NoDeclaredScrollOwner)?;
        let chain = scroll
            .ownership_chain(mounted_instance)
            .map_err(UiHostScrollObservationDenial::Ownership)?;
        if chain.owners().is_empty() {
            return Err(UiHostScrollObservationDenial::NoDeclaredScrollOwner);
        }
        let mut entries = Vec::with_capacity(chain.owners().len());
        let mut slots = Vec::with_capacity(chain.owners().len());
        for (slot, owner) in chain.owners().iter().copied().enumerate() {
            let incarnation = self
                .scroll_owner_chain_incarnation(owner, mounted_instance, slot)
                .ok_or(UiHostScrollObservationDenial::AllocationUnavailable)?;
            entries.push(UiScrollChainEntry::new(owner, incarnation));
            slots.push(slot);
        }
        Ok(UiScrollRoutedChain {
            mounted_instance,
            graph_node,
            entries,
            slots,
        })
    }

    /// The incarnation one chain position holds. A declared region occurrence
    /// carries its own; a surface or viewport owner carries the surface's.
    fn scroll_owner_chain_incarnation(
        &self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        slot: usize,
    ) -> Option<crate::runtime::scroll::UiScrollOwnerIncarnation> {
        match owner {
            crate::runtime::scroll::UiScrollOwnerIdentity::Region { .. } => {
                self.scroll_region_incarnation(mounted_instance, slot)
            }
            crate::runtime::scroll::UiScrollOwnerIdentity::Surface(_)
            | crate::runtime::scroll::UiScrollOwnerIdentity::Viewport(_) => {
                Some(self.scroll_owner_incarnation())
            }
        }
    }

    /// Take, carry forward or end the gesture's latch, after the route it
    /// describes has been committed.
    ///
    /// Committed is the point of it. A latch taken before the offset moved
    /// would name an owner that a later denial leaves unmoved, and the next
    /// event of the gesture would be handed to a region the reader never saw
    /// respond.
    pub(in crate::facade::entry) fn latch_routed_scroll_gesture(
        &mut self,
        routed: &UiScrollRoutedChain,
        region: Option<UiScrollRoutedRegion>,
        phase: worth_ui_host_contract::UiHostScrollDeltaPhase,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        input_tick: u64,
    ) {
        use worth_ui_host_contract::UiHostScrollDeltaPhase;

        if matches!(
            phase,
            UiHostScrollDeltaPhase::Ended | UiHostScrollDeltaPhase::Cancelled
        ) {
            // The release is next door in the observation, where it happens
            // whether or not this route committed. A gesture stating its end
            // has no latch to take and none worth carrying forward, so the
            // committed path only has to leave the dying one alone.
            return;
        }
        let Some(region) = region else {
            // Nothing in the chain had room for this gesture, so there is no
            // owner to name. A latch already held stays held and keeps its
            // remaining interval; there is nothing here to move it to.
            return;
        };
        // A gesture that already holds a latch keeps the lifetime it took;
        // only its last-seen tick moves. This is what makes a quiet interval
        // measure quiet rather than gesture length.
        //
        // The held latch has to name the owner this route just reached before
        // its tick is worth moving. Ordinarily it does, because a live latch
        // is what narrowed the chain this route ran against. When the owner it
        // names has left the chain without being rebound or unmounted -- the
        // chain shortened past its slot -- narrowing finds nothing and the
        // pointer picked the owner instead. Refreshing on that answer would
        // keep an unreachable owner named for the rest of the interval while
        // every event went somewhere else, which is the handover a latch
        // exists to prevent. The gesture latches to where it actually went.
        let entry = region.entry();
        if let Some(held) = self.interaction.scroll_gesture_latch_at(input_tick) {
            if held.binds(entry.owner(), entry.incarnation()) {
                self.interaction
                    .latch_scroll_gesture(held.refreshed(input_tick));
                return;
            }
        }
        let Some(lifetime) = self.scroll_gesture_latch_lifetime(phase) else {
            return;
        };
        self.interaction
            .latch_scroll_gesture(UiScrollGestureLatch::new(
                entry.owner(),
                entry.incarnation(),
                routed.mounted_instance(),
                region.slot(),
                presentation.binding(),
                lifetime,
                input_tick,
            ));
    }

    /// Release the latch a gesture that has just stated its end was holding,
    /// whatever the session made of the report that stated it.
    ///
    /// Every other event of a gesture can afford to release nothing, because
    /// another one follows it. The last one cannot: routing is the first thing
    /// a scroll observation does and the place it is refused -- a modal opened
    /// over the latched owner, a frame the presentation has already replaced --
    /// and a refusal there returns before any of the committing work below it
    /// runs. Releasing only on a committed route would leave a phased latch,
    /// which has no interval to expire, held by a gesture that is over and no
    /// event left to clear it, and the next gesture anywhere on the surface
    /// would be handed to the owner this one left behind.
    pub(in crate::facade::entry) fn end_scroll_gesture_latch_on_phase(
        &mut self,
        phase: worth_ui_host_contract::UiHostScrollDeltaPhase,
    ) {
        if matches!(
            phase,
            worth_ui_host_contract::UiHostScrollDeltaPhase::Ended
                | worth_ui_host_contract::UiHostScrollDeltaPhase::Cancelled
        ) {
            self.interaction.end_scroll_gesture_latch();
        }
    }

    /// How long a latch taken on this phase should live.
    ///
    /// A phased gesture states its own end, so the host decides. An unphased
    /// coarse wheel never ends -- it goes quiet -- so its latch outlives each
    /// notch by the declared settle horizon: while the content that notch
    /// started is still moving, the same owner keeps the wheel. A wheel with no
    /// declared settle leaves nothing in flight to protect and takes no latch.
    fn scroll_gesture_latch_lifetime(
        &self,
        phase: worth_ui_host_contract::UiHostScrollDeltaPhase,
    ) -> Option<UiScrollGestureLatchLifetime> {
        match phase {
            worth_ui_host_contract::UiHostScrollDeltaPhase::Started => {
                Some(UiScrollGestureLatchLifetime::PhasedGesture)
            }
            worth_ui_host_contract::UiHostScrollDeltaPhase::Updated => self
                .application
                .prepared_authority()
                .service_policy_plan()
                .scroll()?
                .wheel_behavior()
                .settle_ticks()
                .map(|quiet_ticks| UiScrollGestureLatchLifetime::QuietInterval { quiet_ticks }),
            worth_ui_host_contract::UiHostScrollDeltaPhase::Ended
            | worth_ui_host_contract::UiHostScrollDeltaPhase::Cancelled => None,
        }
    }
}

#[cfg(test)]
#[path = "scroll_gesture_latching/routed_chain_tests.rs"]
mod routed_chain_tests;
