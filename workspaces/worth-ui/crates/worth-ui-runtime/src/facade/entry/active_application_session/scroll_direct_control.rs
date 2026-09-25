//! A pointer taking direct control of one Scroll region's offset.
//!
//! A thumb press or a track page places offsets directly: the offset the
//! pointer names is the offset that applies, on this frame, with nothing in
//! between. Anything still settling that same region would fight it -- a live
//! Motion track walking the content toward a wheel target, or a retained
//! accepted sample the frame settle keeps re-applying -- so direct control ends
//! both. The pending semantic target goes with them: the reader has said where
//! the content is, and a notch that arrives later accumulates from there, not
//! from an intention the drag overrode.
//!
//! When that ending happens follows the evidence. A thumb press ends the settle
//! at capture, because the latch is already the pointer's and a settle still
//! walking the region would pull the thumb out from under the grab. A placed
//! offset ends it only once the ordinary host frame carrying its exact pending
//! result is accepted. The target, sampler and Motion track remain authoritative
//! while that candidate is pending or refused.
//!
//! The Motion track ends with its own terminal cause, published as a fact like
//! every other ending, so the record shows the drag displaced it rather than
//! that it completed.

use crate::runtime::motion::{UiMotionTargetIdentity, UiMotionTerminalCause};

/// What taking direct control found to end. Each is `true` when something was
/// actually retired, so a caller can tell a drag that interrupted a settle from
/// one that began at rest.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::facade::entry) struct UiScrollDirectControlReceipt {
    /// The owner held a pending wheel target, now retired.
    pub(in crate::facade::entry) transition_retired: bool,
    /// The sampler held an accepted sample for the content group, now retired.
    pub(in crate::facade::entry) sample_retired: bool,
    /// Motion held a committed track for the content group, now terminal.
    pub(in crate::facade::entry) track_terminalized: bool,
}

/// What ending the content motion of one region found to end.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::facade::entry) struct UiScrollContentMotionEnd {
    /// The sampler held an accepted sample for the content group, now retired.
    pub(in crate::facade::entry) sample_retired: bool,
    /// Motion held a committed track for the content group, now terminal.
    pub(in crate::facade::entry) track_terminalized: bool,
}

impl super::super::WorthUiActiveApplicationSession {
    pub(super) fn stage_direct_scroll_succession(
        &mut self,
        successor: &crate::runtime::scroll::UiScrollRuntimeState,
        receipt: &crate::runtime::scroll::UiScrollRouteReceipt,
        occurrence: worth_ui_host_contract::UiMountedInstanceIdentity,
        geometry: &[Option<worth_ui_host_contract::UiMountedInstanceIdentity>],
    ) -> Result<(), crate::mounting::UiMountedOccurrenceGeometryDenial> {
        let prepared = successor.prepare_direct_succession(receipt, occurrence, geometry);
        self.mounted.stage_direct_scroll_geometry(&prepared)?;
        self.scroll
            .as_mut()
            .expect("direct input has an installed Scroll owner")
            .stage_direct_succession(successor, &prepared);
        Ok(())
    }

    /// Stage a published Portal's focus reveal as the direct placement it is.
    /// Like a thumb placed on a track, it says where the content is, so it
    /// lands with the frame that carries it. Until then Scroll holds it
    /// pending and mounted geometry holds its pose, both awaiting publication.
    /// Owners past the prefix the route moved keep their accepted records: a
    /// reveal that never reached them has no offset to land there.
    pub(in crate::facade::entry) fn stage_focus_reveal_placement(
        &mut self,
        reveal: crate::runtime::session::UiStagedFocusReveal,
    ) {
        let mut successor = self
            .scroll
            .as_ref()
            .expect("a staged reveal retains its installed Scroll owner")
            .clone();
        let (target, receipt) = reveal.route(&mut successor);
        let installed = self
            .scroll
            .as_mut()
            .expect("a staged reveal retains its installed Scroll owner");
        if successor.holds_offsets_of(installed) {
            // A reveal that moves nothing leaves no pose awaiting publication,
            // only its anchor rebind, which is Scroll's own record.
            *installed = successor;
            return;
        }
        // A route moves a prefix of the chain, so each transition's index is
        // its owner's slot in the target's chain.
        let geometry = receipt
            .transitions()
            .iter()
            .enumerate()
            .map(|(slot, transition)| match transition.owner() {
                crate::runtime::scroll::UiScrollOwnerIdentity::Region { .. } => {
                    // Publication lands the placement only on the incarnation
                    // the region's frame carries; any other would drop it.
                    debug_assert!(
                        self.mounted
                            .scroll_region_incarnation(target, slot)
                            .is_some_and(|incarnation| successor
                                .offset(transition.owner(), incarnation)
                                .is_ok()),
                        "a revealed region owner holds the incarnation its frame lands on"
                    );
                    self.mounted
                        .scroll_region_geometry(target, slot)
                        .map(|row| row.0)
                }
                crate::runtime::scroll::UiScrollOwnerIdentity::Surface(_)
                | crate::runtime::scroll::UiScrollOwnerIdentity::Viewport(_) => None,
            })
            .collect::<Vec<_>>();
        self.stage_direct_scroll_succession(&successor, &receipt, target, &geometry)
            .expect("a settled Portal publication leaves no presentation in flight");
    }

    /// End everything still moving `owner`'s content on `mounted_instance` at
    /// capture, so the pointer that just latched the thumb is the only
    /// authority left over the region.
    pub(in crate::facade::entry) fn take_direct_scroll_control(
        &mut self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> UiScrollDirectControlReceipt {
        let transition_retired = self
            .scroll
            .as_mut()
            .is_some_and(|scroll| scroll.retire_transition(owner));
        let ended = self.end_scroll_content_motion(
            owner,
            mounted_instance,
            UiMotionTerminalCause::DisplacedByDirectControl,
        );
        UiScrollDirectControlReceipt {
            transition_retired,
            sample_retired: ended.sample_retired,
            track_terminalized: ended.track_terminalized,
        }
    }

    /// End the sampler track and the Motion track still moving `owner`'s
    /// content on `mounted_instance`, recording `cause` as why.
    ///
    /// A pointer placing a pose calls this once that pose has landed, so the
    /// displayed pose from here is the one direct control applied and no
    /// accepted sample stands behind it. A lifecycle boundary calls it because
    /// the content has stopped being something the reader can see. The ending
    /// is the same either way; only the reason differs, and the reason is
    /// published as a fact, so the record says which happened.
    pub(in crate::facade::entry) fn end_scroll_content_motion(
        &mut self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        cause: UiMotionTerminalCause,
    ) -> UiScrollContentMotionEnd {
        let target = scroll_content_motion_target(owner, mounted_instance);
        self.end_scroll_content_motion_target(target, cause)
    }

    /// The same ending, named by the Motion target rather than by the owner it
    /// was derived from. A sweep that starts from what the sampler still holds
    /// has the target in hand and no owner to rebuild it from.
    pub(in crate::facade::entry) fn end_scroll_content_motion_target(
        &mut self,
        target: UiMotionTargetIdentity,
        cause: UiMotionTerminalCause,
    ) -> UiScrollContentMotionEnd {
        let sample_retired = self.mounted.retire_scroll_motion_sample(target);
        let terminal = self
            .motion
            .as_mut()
            .and_then(|motion| motion.terminalize_target(target, cause));
        if let Some(terminal) = terminal {
            // A Scroll content track retains no Portal exit, so the coordinator
            // has nothing to settle for it; it is told so it can prove that.
            self.portal_exit_retention
                .observe_terminal(terminal)
                .expect("a Scroll content track holds no portal exit retention");
        }
        UiScrollContentMotionEnd {
            sample_retired,
            track_terminalized: terminal.is_some(),
        }
    }
}

/// The Motion target one Scroll owner's content moves under on one occurrence.
pub(in crate::facade::entry) fn scroll_content_motion_target(
    owner: crate::runtime::scroll::UiScrollOwnerIdentity,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
) -> UiMotionTargetIdentity {
    UiMotionTargetIdentity::from_scroll_region_owner(
        owner.semantic_surface(),
        mounted_instance,
        super::scroll_transition_preparation::scroll_motion_owner_key(owner),
    )
}
