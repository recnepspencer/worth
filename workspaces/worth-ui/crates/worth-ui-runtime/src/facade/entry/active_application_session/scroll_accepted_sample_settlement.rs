//! Settling the accepted Motion sample of a Scroll content group into the
//! mounted scroll pose once a witness displays it.
//!
//! A displayed sample is the only authority over scrolled geometry, so every
//! frame whose witness displays one has to move the region's descendants to
//! the offset that sample stands at. Mounted geometry refuses while a presentation
//! attempt is in flight; that refusal defers the settle to the next frame and
//! never drops it, because the accepted sample remains true until a later one
//! replaces it. Every other refusal is reported through the disposition.
//!
//! A settle deferred behind a staged layout whose Motion has arrived is
//! carried into that layout, so the frame that shows it lands the content
//! where the sample put it.
//!
//! While a settle is owed, the host shows the content where the sample put it
//! and committed geometry has yet to follow, so hit rows lead to the sample
//! until the settle is paid. Every outcome that owes nothing retires the
//! leads: a pointer reaches content where the host shows it, never where a
//! sample nobody shows any more left it.

use super::scroll_settle_disposition::{
    UiAcceptedScrollSettlementDenial, UiScrollSettleDisposition, UiScrollSettleRefusal,
};
use crate::mounting::presentation::motion_sampling::UiPresentationMotionPresentedSurface;
use crate::mounting::presentation::{
    UiDisplayedRect, UiDisplayedScrollOffset, UiPublishedRect, UiScrollStandingDenial,
};
use crate::runtime::motion::{UiMotionTargetIdentity, UiMotionTargetScope};

/// The region owner an accepted Scroll sample belongs to, and the rest box
/// its offset is measured from.
#[derive(Clone, Copy, Debug)]
pub(in crate::facade::entry) struct UiScrollSettlementOwner {
    pub(super) surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    pub(super) owner_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    pub(super) region_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    pub(super) slot: usize,
    pub(super) entry: crate::runtime::scroll::UiScrollChainEntry,
    rest: UiPublishedRect,
}

/// One displayed Scroll sample resolved against the owner it belongs to: where
/// the witness showed the content, and which owner's offset has to follow it
/// there.
#[derive(Clone, Copy, Debug)]
pub(in crate::facade::entry) struct UiAcceptedScrollSettlement {
    pub(super) owner: UiScrollSettlementOwner,
    pub(super) offset: UiDisplayedScrollOffset,
}

impl UiScrollSettlementOwner {
    /// The settlement `sample` asks of this owner: the offset that places its
    /// content group where the witness displayed it.
    pub(super) fn settle(
        self,
        sample: UiDisplayedRect,
    ) -> Result<UiAcceptedScrollSettlement, UiAcceptedScrollSettlementDenial> {
        let offset =
            UiDisplayedScrollOffset::from_rest(self.rest, sample).map_err(
                |denial| match denial {
                    UiScrollStandingDenial::CoordinateSpaceChanged => {
                        UiAcceptedScrollSettlementDenial::SampleOutsideRestSpace
                    }
                    UiScrollStandingDenial::BeforeRest => {
                        UiAcceptedScrollSettlementDenial::SampleBeforeRest
                    }
                },
            )?;
        Ok(UiAcceptedScrollSettlement {
            owner: self,
            offset,
        })
    }
}

impl super::super::WorthUiActiveApplicationSession {
    /// Apply the accepted offset of every retained Scroll content group to the
    /// surface a presentation witness committed, and record what happened
    /// where a reader can find it.
    pub(in crate::facade::entry) fn settle_accepted_scroll_sample(
        &mut self,
        presented: UiPresentationMotionPresentedSurface,
    ) -> UiScrollSettleDisposition {
        let disposition = self.settle_accepted_scroll_sample_inner(presented);
        self.last_scroll_settle_disposition = disposition;
        if !matches!(
            disposition,
            UiScrollSettleDisposition::DeferredPendingGeometry
                | UiScrollSettleDisposition::DeferredPresentationInFlight
        ) {
            self.retire_scroll_hit_leads(presented);
        }
        if matches!(
            disposition,
            UiScrollSettleDisposition::Applied | UiScrollSettleDisposition::Idle
        ) {
            self.finish_pending_scroll_chrome_capture();
        }
        disposition
    }

    fn settle_accepted_scroll_sample_inner(
        &mut self,
        presented: UiPresentationMotionPresentedSurface,
    ) -> UiScrollSettleDisposition {
        let Some(surface) = self
            .mounted
            .current_surface_for_binding(presented.displayed().binding())
        else {
            return UiScrollSettleDisposition::Idle;
        };
        let accepted = self.mounted.accepted_scroll_group_samples();
        if accepted.is_empty() {
            self.owed_scroll_settles.paid(presented.semantic_surface());
            return UiScrollSettleDisposition::Idle;
        }
        let pending = self.scroll.as_ref().is_some_and(|scroll| {
            scroll.has_pending_direct(surface) || scroll.has_unpresented_layout(surface)
        });
        let mut settlements = Vec::with_capacity(accepted.len());
        let mut targets = Vec::with_capacity(accepted.len());
        let mut refusal = None;
        for (target, sample) in accepted {
            let settlement = self
                .scroll_settlement_reading()
                .owner(target, surface)
                .and_then(|owner| {
                    owner
                        .map(|owner| {
                            UiDisplayedRect::displayed(sample, presented.displayed())
                                .map_err(|_| UiAcceptedScrollSettlementDenial::SampleNotDisplayed)
                                .and_then(|sample| owner.settle(sample))
                        })
                        .transpose()
                });
            match settlement {
                Ok(Some(settlement)) => {
                    targets.push(target);
                    settlements.push(settlement);
                }
                Ok(None) => {}
                Err(denial) => {
                    refusal.get_or_insert(UiScrollSettleRefusal::Resolution(denial));
                }
            }
        }
        let poses = settlements
            .iter()
            .map(|settlement| {
                (
                    settlement.owner.surface,
                    settlement.owner.owner_instance,
                    settlement.offset,
                )
            })
            .collect::<Vec<_>>();
        if pending {
            let arrivals = targets.into_iter().zip(settlements.iter().copied());
            self.carry_arrivals_into_staged_layout(&arrivals.collect::<Vec<_>>());
            self.owed_scroll_settles.owe(presented);
            self.lead_scroll_hits(&settlements, &poses, presented);
            return UiScrollSettleDisposition::DeferredPendingGeometry;
        }
        if settlements.is_empty() {
            let Some(refusal) = refusal else {
                self.owed_scroll_settles.paid(presented.semantic_surface());
                return UiScrollSettleDisposition::Idle;
            };
            self.owed_scroll_settles
                .refused(presented.semantic_surface());
            return UiScrollSettleDisposition::Refused(refusal);
        }
        match self.mounted.apply_presented_scroll_geometries(&poses) {
            Ok(transitions) => {
                for transition in transitions.iter() {
                    self.interaction
                        .observe_presented_hit_transition(transition, &self.mounted);
                }
                if let Err(write_back) = self.write_back_accepted_scroll_offsets(&settlements) {
                    refusal.get_or_insert(UiScrollSettleRefusal::WriteBack(write_back));
                }
                let Some(refusal) = refusal else {
                    self.owed_scroll_settles.paid(presented.semantic_surface());
                    return UiScrollSettleDisposition::Applied;
                };
                self.owed_scroll_settles
                    .refused(presented.semantic_surface());
                UiScrollSettleDisposition::Refused(refusal)
            }
            Err(crate::mounting::UiMountedOccurrenceGeometryDenial::PresentationInFlight) => {
                // The accepted sample is still true, so the settle is owed, not
                // lost. Re-arming keeps frames coming until it is paid.
                self.owed_scroll_settles.owe(presented);
                self.lead_scroll_hits(&settlements, &poses, presented);
                UiScrollSettleDisposition::DeferredPresentationInFlight
            }
            Err(denial) => {
                self.owed_scroll_settles
                    .refused(presented.semantic_surface());
                UiScrollSettleDisposition::Refused(UiScrollSettleRefusal::Geometry(denial))
            }
        }
    }

    /// Whether the accepted sample of one region's content group, if it has
    /// one, has settled: the surface's current witness displays it, and both
    /// the mounted pose and the owner's offset stand where it was displayed.
    /// A group with no accepted sample has nothing left to settle.
    pub(super) fn accepted_scroll_sample_settled(
        &self,
        target: UiMotionTargetIdentity,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        mounted_offset: crate::runtime::scroll::UiScrollOffset,
        owner_offset: Option<crate::runtime::scroll::UiScrollOffset>,
    ) -> bool {
        if self.mounted.accepted_scroll_group_sample(target).is_none() {
            return true;
        }
        self.scroll_settlement_reading()
            .displayed_offset(target, surface)
            .is_some_and(|displayed| {
                displayed.stands_at(mounted_offset)
                    && owner_offset.is_some_and(|offset| displayed.stands_at(offset))
            })
    }

    /// What a settlement reads, lent from this session.
    pub(in crate::facade::entry) fn scroll_settlement_reading(
        &self,
    ) -> UiScrollSettlementReading<'_> {
        UiScrollSettlementReading {
            mounted: &self.mounted,
            scroll: self.scroll.as_ref(),
        }
    }
}

/// What a Scroll settlement reads: mounted geometry and Scroll, and nothing
/// else. A publication path that lends the session out in parts can read it
/// as well as the session can.
#[derive(Clone, Copy)]
pub(in crate::facade::entry) struct UiScrollSettlementReading<'a> {
    pub(in crate::facade::entry) mounted: &'a crate::mounting::WorthUiMountedSessionState,
    pub(in crate::facade::entry) scroll: Option<&'a crate::runtime::scroll::UiScrollRuntimeState>,
}

impl UiScrollSettlementReading<'_> {
    /// The offset the host shows one region's content group at: its accepted
    /// sample as the surface's current witness displays it, measured from the
    /// owner's rest box. `None` when no sample stands, no witness displays
    /// it, or it resolves to no owner on `surface`.
    pub(in crate::facade::entry) fn displayed_offset(
        self,
        target: UiMotionTargetIdentity,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<UiDisplayedScrollOffset> {
        let sample = self.displayed_sample(target, surface)?;
        let owner = self.owner(target, surface).ok()??;
        owner.settle(sample).ok().map(|settled| settled.offset)
    }

    /// One region's accepted content-group sample as the surface's current
    /// witness displays it. `None` when no sample stands or that witness
    /// does not display it.
    pub(in crate::facade::entry) fn displayed_sample(
        self,
        target: UiMotionTargetIdentity,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<UiDisplayedRect> {
        let sample = self.mounted.accepted_scroll_group_sample(target)?;
        let displayed = self.mounted.current_presentation_for_surface(surface)?;
        UiDisplayedRect::displayed(sample, displayed).ok()
    }

    /// The region owner one accepted sample settles, and the rest box its
    /// offset is measured from.
    ///
    /// `Ok(None)` names a sample that is not this surface's Scroll content at
    /// all; every failure to resolve a sample that is comes back typed.
    pub(in crate::facade::entry) fn owner(
        self,
        target: UiMotionTargetIdentity,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Result<Option<UiScrollSettlementOwner>, UiAcceptedScrollSettlementDenial> {
        if target.scope() != UiMotionTargetScope::ScrollContents
            || target.semantic_surface() != surface
        {
            return Ok(None);
        }
        let mounted_instance = target.mounted_instance();
        let scroll = self
            .scroll
            .ok_or(UiAcceptedScrollSettlementDenial::ScrollUnavailable)?;
        let chain = scroll
            .ownership_chain(mounted_instance)
            .map_err(UiAcceptedScrollSettlementDenial::Ownership)?;
        let motion_owner_key = target.owner_key();
        let slot = latched_chain_slot_for_motion_owner(chain, motion_owner_key)
            .ok_or(UiAcceptedScrollSettlementDenial::OwnerNotInChain { motion_owner_key })?;
        let owner = chain.owners()[slot];
        // Samples are measured from the content box with this region and
        // every region enclosing it at offset zero, which no pose moves.
        let (Some((owner_instance, _)), Some(rest)) = (
            self.mounted.scroll_region_geometry(mounted_instance, slot),
            self.mounted.scroll_region_rest(mounted_instance, slot),
        ) else {
            return Err(UiAcceptedScrollSettlementDenial::GeometryUnavailable);
        };
        let incarnation = self
            .mounted
            .scroll_region_incarnation(mounted_instance, slot)
            .ok_or(UiAcceptedScrollSettlementDenial::IncarnationUnavailable)?;
        Ok(Some(UiScrollSettlementOwner {
            surface: target.semantic_surface(),
            owner_instance,
            region_instance: mounted_instance,
            slot,
            entry: crate::runtime::scroll::UiScrollChainEntry::new(owner, incarnation),
            // A Scroll track moves content where it is laid out, so its
            // samples stand in layout space, wherever a Portal presents them.
            rest: UiPublishedRect::from_committed_box(rest.into_layout_space()),
        }))
    }
}

/// The chain position of the region owner a Motion owner key names.
fn latched_chain_slot_for_motion_owner(
    chain: &crate::runtime::scroll::UiResolvedScrollOwnershipChain,
    motion_owner_key: u64,
) -> Option<usize> {
    chain.owners().iter().position(|owner| {
        super::scroll_transition_preparation::scroll_motion_owner_key(*owner) == motion_owner_key
            && matches!(
                owner,
                crate::runtime::scroll::UiScrollOwnerIdentity::Region { .. }
            )
    })
}
