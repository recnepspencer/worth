//! Exact disposal of the initial WAL fingerprint after its final reread agrees.

use super::{Denial, ResidentDenial, SelectedWalMediaFingerprint, StoreRejoinResidentLedger};

impl SelectedWalMediaFingerprint {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn discard_with_resident(
        self,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<(), Denial> {
        let total = self.owned_heap_bytes().ok_or_else(|| {
            Denial::Resident(ResidentDenial::SizeOverflow {
                admitted: resident.used().saturating_add(resident.remaining()),
            })
        })?;
        drop(self);
        resident.release(total);
        Ok(())
    }
}
