//! Writing an applied accepted sample back into the Scroll owner's offset.
//!
//! Until this runs, a settling region has two different truths: the displayed
//! content is where the accepted sample put it, and the Scroll owner still
//! holds the offset it had when the notch arrived. The write-back closes that
//! gap once the pose is actually applied, never before -- an offset that moved
//! ahead of the displayed pose would make every later delta start from a place
//! the reader has not seen.
//!
//! The write is a route under the region's current bounds, so the accepted
//! offset and the owner's pending settle target are clamped against the same
//! bounds in the same call.

use super::scroll_accepted_sample_settlement::UiAcceptedScrollSettlement;
use super::scroll_settle_disposition::UiScrollWriteBackRefusal;

impl super::super::WorthUiActiveApplicationSession {
    /// Write every applied settlement back into its owner's offset.
    ///
    /// Every settlement is attempted; a refused one leaves its owner holding
    /// the previous offset and is reported. When more than one is refused the
    /// first is reported, and the rest come back with the next frame's
    /// accepted samples, so nothing is lost by naming only one.
    pub(super) fn write_back_accepted_scroll_offsets(
        &mut self,
        settlements: &[UiAcceptedScrollSettlement],
    ) -> Result<(), UiScrollWriteBackRefusal> {
        let reconciled = settlements
            .iter()
            .map(|settlement| {
                self.accepted_scroll_bounds(settlement)
                    .map(|bounds| (settlement.entry, settlement.offset, bounds))
            })
            .collect::<Vec<_>>();
        let scroll = self
            .scroll
            .as_mut()
            .ok_or(UiScrollWriteBackRefusal::ScrollUnavailable)?;
        let mut first_refusal = None;
        for (settlement, reconciled) in settlements.iter().zip(reconciled) {
            let outcome = reconciled.and_then(|(entry, offset, bounds)| {
                scroll
                    .settle_accepted_sample(entry, offset, bounds)
                    .map(|_| ())
                    .map_err(|denial| UiScrollWriteBackRefusal::Route {
                        region_instance: settlement.region_instance,
                        denial,
                    })
            });
            if let Err(refusal) = outcome {
                first_refusal.get_or_insert(refusal);
            }
        }
        // A settle that has arrived stops being a pending intention here, so a
        // later notch accumulates from the offset the reader can see rather
        // than from a target already reached.
        scroll.retire_reached_transitions();
        first_refusal.map_or(Ok(()), Err)
    }

    /// The bounds the settling region currently has, resolved the same way a
    /// host delta resolves them.
    fn accepted_scroll_bounds(
        &self,
        settlement: &UiAcceptedScrollSettlement,
    ) -> Result<crate::runtime::scroll::UiScrollBounds, UiScrollWriteBackRefusal> {
        let region_instance = settlement.region_instance;
        let basis = self
            .mounted
            .current_mounted_identity_basis(region_instance)
            .ok_or(UiScrollWriteBackRefusal::RegionBasisUnavailable { region_instance })?;
        self.scroll_bounds_for_mounted_owner(
            settlement.entry.owner(),
            region_instance,
            basis.graph_node_identity(),
            settlement.slot,
        )
        .map_err(|denial| UiScrollWriteBackRefusal::Bounds {
            region_instance,
            denial,
        })
    }
}
