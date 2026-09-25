//! Ending every scroll settle inside a scope that has stopped being something
//! the reader is looking at.
//!
//! A settle is three things at once: a semantic target in Scroll, an accepted
//! sample in the sampler, and a committed track in Motion. Direct control ends
//! all three for one region because a pointer has said where that region's
//! content is. A lifecycle boundary ends them for a whole scope because the
//! question the settle was answering has gone away -- the occurrence was
//! unmounted, the surface was rebound, the window stopped being the reader's.
//!
//! Either way the rule is the same. Content stops where its last accepted
//! sample left it, because that is the last thing anyone saw, and nothing keeps
//! moving afterwards. A settle left alive across one of these boundaries is not
//! a cosmetic leak: it goes on writing offsets into a region nobody is watching
//! and hands the next reader content that moved while they were away.
//!
//! The scope is named in Scroll's own terms -- an occurrence or a semantic
//! surface -- so deciding what falls inside it needs no mounted lookup. That
//! matters because these boundaries are exactly the moments when the mounted
//! rows a lookup would read are being torn down.

use crate::runtime::motion::UiMotionTerminalCause;

/// Which live settles a lifecycle boundary reaches.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::facade::entry) enum UiScrollSettlementScope {
    /// One mounted occurrence, as removing that occurrence reaches it.
    MountedOccurrence(worth_ui_host_contract::UiMountedInstanceIdentity),
    /// Every owner presented on one semantic surface, as rebinding that
    /// surface reaches them: the rectangles they were moving content through
    /// belong to the binding that is going away.
    SemanticSurface(worth_ui_host_contract::UiSemanticSurfaceIdentity),
    /// Every live settle. Losing the reader's attention is a fact about the
    /// window rather than about any one surface, so it reaches all of them.
    Everything,
}

impl UiScrollSettlementScope {
    /// Whether one pending settle falls inside this scope.
    fn reaches(
        self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        occurrence: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> bool {
        match self {
            Self::MountedOccurrence(identity) => occurrence == identity,
            Self::SemanticSurface(surface) => owner.semantic_surface() == surface,
            Self::Everything => true,
        }
    }
}

impl super::super::WorthUiActiveApplicationSession {
    /// Input reconciliation can retire only this chain's targets. Smooth
    /// staging requires a declared region line extent, so surface/viewport
    /// owners (shared by occurrences) cannot own such a host-wheel track.
    /// Layout/lifecycle publication retains the broader sweep below.
    pub(super) fn settle_routed_scroll_motion_without_a_target(
        &mut self,
        routed: &super::scroll_gesture_latching::UiScrollRoutedChain,
    ) {
        for entry in routed.entries() {
            if matches!(
                entry.owner(),
                crate::runtime::scroll::UiScrollOwnerIdentity::Region { .. }
            ) && self.scroll.as_ref().is_some_and(|scroll| {
                scroll
                    .transition_target(entry.owner(), entry.incarnation())
                    .is_none()
            }) {
                self.end_scroll_content_motion(
                    entry.owner(),
                    routed.mounted_instance(),
                    UiMotionTerminalCause::NowhereLeftToSettle,
                );
            }
        }
    }

    /// End every scroll settle `scope` reaches, recording `cause` as why.
    ///
    /// The settles are enumerated once, before any of them is ended, because
    /// ending one mutates the lists the enumeration reads. Each is then ended
    /// the same way direct control ends one: the semantic target first, then
    /// the accepted sample and the Motion track that were carrying it out.
    ///
    /// Nothing here is fallible. A boundary that has already happened cannot be
    /// refused a settlement, so this is called after whatever fallible step the
    /// boundary had -- a mounted unmount, a surface deregistration -- has
    /// already succeeded.
    pub(in crate::facade::entry) fn settle_live_scroll_motion(
        &mut self,
        scope: UiScrollSettlementScope,
        cause: UiMotionTerminalCause,
    ) {
        let Some(scroll) = self.scroll.as_ref() else {
            return;
        };
        let reached = scroll
            .pending_settle_occurrences()
            .into_iter()
            .filter(|(owner, occurrence)| scope.reaches(*owner, *occurrence))
            .collect::<Vec<_>>();
        for (owner, occurrence) in reached {
            if let Some(scroll) = self.scroll.as_mut() {
                scroll.retire_transition(owner);
            }
            self.end_scroll_content_motion(owner, occurrence, cause);
        }
    }

    /// End the content motion of every scroll settle that no longer has a
    /// semantic target to settle toward.
    ///
    /// Called once a publication has reconciled geometry, which is when an
    /// owner's extent can collapse: its content emptied, or shrank to fit its
    /// viewport. Bounds reconciliation retires a target the new extent leaves
    /// nowhere to travel to, and this is what ends the sampler entry and the
    /// Motion track that were carrying that target out. Without it the frame
    /// settle goes on re-applying an accepted translation for content with no
    /// room to be translated, and the reader watches a list that has one row
    /// left in it drift.
    ///
    /// Reading the sampler rather than Scroll is what makes this a sweep: the
    /// targets are gone by now, so what is left to find is the motion they
    /// left behind.
    pub(in crate::facade::entry) fn settle_scroll_motion_without_a_target(&mut self) {
        let settling = self
            .scroll
            .as_ref()
            .map(|scroll| {
                scroll
                    .pending_settle_occurrences()
                    .into_iter()
                    .map(|(owner, occurrence)| {
                        super::scroll_direct_control::scroll_content_motion_target(
                            owner, occurrence,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let orphaned = self
            .mounted
            .accepted_scroll_group_samples()
            .into_iter()
            .map(|(target, _)| target)
            .filter(|target| !settling.contains(target))
            .collect::<Vec<_>>();
        for target in orphaned {
            self.end_scroll_content_motion_target(
                target,
                UiMotionTerminalCause::NowhereLeftToSettle,
            );
        }
    }

    /// The window stopped being the reader's, so every settle ends and the
    /// wheel gesture that was feeding them gives its owner back.
    ///
    /// A latch outliving focus would hand the first notch after the reader
    /// returns to whatever they were scrolling before they left, which is a
    /// gesture they did not make.
    pub(in crate::facade::entry) fn settle_scroll_after_attention_loss(&mut self) {
        self.interaction.end_scroll_gesture_latch();
        self.settle_live_scroll_motion(
            UiScrollSettlementScope::Everything,
            UiMotionTerminalCause::AttentionLost,
        )
    }
}
