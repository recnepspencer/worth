//! Exact disposal of the initial WAL inventory after its final reread agrees.

use super::{AdmittedWalInventory, Denial, ResidentDenial, StoreRejoinResidentLedger};

impl AdmittedWalInventory {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn discard_with_resident(
        self,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<(), Denial> {
        let overflow = || {
            Denial::Resident(ResidentDenial::SizeOverflow {
                admitted: resident.used().saturating_add(resident.remaining()),
            })
        };
        let total = resident
            .vector_bytes(&self.frames.values)
            .map_err(Denial::Resident)?
            .checked_add(
                resident
                    .vector_bytes(&self.artifacts.values)
                    .map_err(Denial::Resident)?,
            )
            .ok_or_else(overflow)?;
        let total = self.frames.iter().try_fold(total, |bytes, frame| {
            bytes.checked_add(frame.owned_heap_bytes()?)
        });
        let total = total.ok_or_else(overflow)?;
        drop(self);
        resident.release(total);
        Ok(())
    }
}
