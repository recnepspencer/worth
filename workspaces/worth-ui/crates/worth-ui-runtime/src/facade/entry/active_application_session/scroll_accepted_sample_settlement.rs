//! Settling the accepted Motion sample of a Scroll content group into the
//! displayed scroll pose.
//!
//! The accepted sample is the only authority over displayed scrolled geometry,
//! so every frame that commits one has to move the region's descendants to the
//! offset that sample reports. Mounted geometry refuses while a presentation
//! attempt is in flight; that refusal defers the settle to the next frame and
//! never drops it, because the accepted sample remains true until a later one
//! replaces it. Every other refusal is reported through the disposition.

use super::scroll_settle_disposition::{
    UiAcceptedScrollSettlementDenial, UiScrollSettleDisposition, UiScrollSettleRefusal,
};
use crate::mounting::presentation::motion_sampling::UiPresentationMotionPresentedSurface;
use crate::runtime::motion::{UiMotionTargetIdentity, UiMotionTargetScope};

/// One accepted Scroll sample resolved against the owner it belongs to: where
/// it puts the displayed content, and which owner's semantic offset has to
/// follow it there.
#[derive(Clone, Copy, Debug)]
pub(in crate::facade::entry) struct UiAcceptedScrollSettlement {
    pub(super) surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    pub(super) owner_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    pub(super) region_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    pub(super) slot: usize,
    pub(super) entry: crate::runtime::scroll::UiScrollChainEntry,
    pub(super) offset: crate::runtime::scroll::UiScrollOffset,
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
        let accepted = self.mounted.accepted_scroll_group_translations();
        if accepted.is_empty() {
            self.owed_scroll_settles.paid(presented.semantic_surface());
            return UiScrollSettleDisposition::Idle;
        }
        if self.scroll.as_ref().is_some_and(|scroll| {
            scroll.has_pending_direct(surface) || scroll.has_unpresented_layout(surface)
        }) {
            self.owed_scroll_settles.owe(presented);
            return UiScrollSettleDisposition::DeferredPendingGeometry;
        }
        let mut settlements = Vec::with_capacity(accepted.len());
        let mut refusal = None;
        for (target, sampled_position) in accepted {
            match self.accepted_scroll_settlement(target, sampled_position, surface) {
                Ok(Some(settlement)) => settlements.push(settlement),
                Ok(None) => {}
                Err(denial) => {
                    refusal.get_or_insert(UiScrollSettleRefusal::Resolution(denial));
                }
            }
        }
        if settlements.is_empty() {
            self.owed_scroll_settles.paid(presented.semantic_surface());
            return refusal.map_or(UiScrollSettleDisposition::Idle, |refusal| {
                UiScrollSettleDisposition::Refused(refusal)
            });
        }
        let poses = settlements
            .iter()
            .map(|settlement| {
                (
                    settlement.surface,
                    settlement.owner_instance,
                    settlement.offset,
                )
            })
            .collect::<Vec<_>>();
        match self.mounted.apply_presented_scroll_geometries(&poses) {
            Ok(transitions) => {
                for transition in transitions.iter() {
                    self.interaction
                        .observe_presented_hit_transition(transition, &self.mounted);
                }
                self.owed_scroll_settles.paid(presented.semantic_surface());
                if let Err(write_back) = self.write_back_accepted_scroll_offsets(&settlements) {
                    refusal.get_or_insert(UiScrollSettleRefusal::WriteBack(write_back));
                }
                refusal.map_or(UiScrollSettleDisposition::Applied, |refusal| {
                    UiScrollSettleDisposition::Refused(refusal)
                })
            }
            Err(crate::mounting::UiMountedOccurrenceGeometryDenial::PresentationInFlight) => {
                // The accepted sample is still true, so the settle is owed, not
                // lost. Re-arming keeps frames coming until it is paid.
                self.owed_scroll_settles.owe(presented);
                UiScrollSettleDisposition::DeferredPresentationInFlight
            }
            Err(denial) => {
                self.owed_scroll_settles.paid(presented.semantic_surface());
                UiScrollSettleDisposition::Refused(UiScrollSettleRefusal::Geometry(denial))
            }
        }
    }

    /// The displayed pose one accepted sample asks for: the owning region
    /// occurrence, and the offset that places its content group where the
    /// sample says the host already put it.
    ///
    /// `sampled_position` is where the sampler reports the content box now
    /// sits, in the region's own coordinate space. `Ok(None)` names a sample
    /// that is not this surface's Scroll content at all; every failure to
    /// resolve a sample that is comes back typed.
    pub(super) fn accepted_scroll_settlement(
        &self,
        target: UiMotionTargetIdentity,
        sampled_position: [f32; 2],
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Result<Option<UiAcceptedScrollSettlement>, UiAcceptedScrollSettlementDenial> {
        if target.scope() != UiMotionTargetScope::ScrollContents
            || target.semantic_surface() != surface
        {
            return Ok(None);
        }
        let mounted_instance = target.mounted_instance();
        let scroll = self
            .scroll
            .as_ref()
            .ok_or(UiAcceptedScrollSettlementDenial::ScrollUnavailable)?;
        let chain = scroll
            .ownership_chain(mounted_instance)
            .map_err(UiAcceptedScrollSettlementDenial::Ownership)?;
        let motion_owner_key = target.owner_key();
        let slot = latched_chain_slot_for_motion_owner(chain, motion_owner_key)
            .ok_or(UiAcceptedScrollSettlementDenial::OwnerNotInChain { motion_owner_key })?;
        let owner = chain.owners()[slot];
        let (owner_instance, content, _) = self
            .mounted
            .scroll_region_geometry(mounted_instance, slot)
            .ok_or(UiAcceptedScrollSettlementDenial::GeometryUnavailable)?;
        // The owner box is the content box at rest: an applied pose moves the
        // owner's descendants, never the owner. The accepted sample reports
        // where the group is now, so the distance from rest to the sample is
        // the offset that sample stands for.
        let accepted_inline = subpixels(content.x() - sampled_position[0]);
        let accepted_block = subpixels(content.y() - sampled_position[1]);
        let incarnation = self
            .scroll_region_incarnation(mounted_instance, slot)
            .ok_or(UiAcceptedScrollSettlementDenial::IncarnationUnavailable)?;
        let offset = crate::runtime::scroll::UiScrollOffset::new(accepted_inline, accepted_block)
            .ok_or(UiAcceptedScrollSettlementDenial::SampleBeforeRest)?;
        Ok(Some(UiAcceptedScrollSettlement {
            surface: target.semantic_surface(),
            owner_instance,
            region_instance: mounted_instance,
            slot,
            entry: crate::runtime::scroll::UiScrollChainEntry::new(owner, incarnation),
            offset,
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

fn subpixels(logical_points: f32) -> i64 {
    (logical_points * worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f32)
        .round() as i64
}
