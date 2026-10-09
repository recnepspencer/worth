//! Exact disposal of the first selected-media observation after the final
//! observation has matched it. Authority remains with the media comparison.

use super::{Denial, ObservedHeadV2Rejoin, StoreRejoinResidentLedger};
use crate::physical_runtime::recovery_construction::selected_rejoin::resident::PhysicalRecoveryRejoinResidentDenial;

impl ObservedHeadV2Rejoin {
    pub(super) fn discard_with_resident(
        self,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<(), Denial> {
        let mut total = 0_u64;
        for bytes in [
            &self.selector,
            &self.root,
            &self.source_root,
            &self.checkpoint,
            &self.selected_free_bytes,
        ] {
            total = total
                .checked_add(resident.vector_bytes(bytes).map_err(Denial::Resident)?)
                .ok_or_else(|| overflow(resident))?;
        }
        if let Some(bytes) = &self.source_free_bytes {
            total = total
                .checked_add(resident.vector_bytes(bytes).map_err(Denial::Resident)?)
                .ok_or_else(|| overflow(resident))?;
        }
        for owned in [
            self.selected_routes.owned_heap_bytes(),
            self.source_routes
                .as_ref()
                .map_or(Some(0), |routes| routes.owned_heap_bytes()),
            self.heads.owned_heap_bytes(),
            self.controls.owned_heap_bytes(),
        ] {
            total = total
                .checked_add(owned.ok_or_else(|| overflow(resident))?)
                .ok_or_else(|| overflow(resident))?;
        }
        total = total
            .checked_add(
                resident
                    .vector_bytes(&self.root_free_slices)
                    .map_err(Denial::Resident)?,
            )
            .ok_or_else(|| overflow(resident))?;
        drop(self);
        resident.release(total);
        Ok(())
    }
}

fn overflow(resident: &StoreRejoinResidentLedger) -> Denial {
    Denial::Resident(PhysicalRecoveryRejoinResidentDenial::SizeOverflow {
        admitted: resident.used().saturating_add(resident.remaining()),
    })
}
