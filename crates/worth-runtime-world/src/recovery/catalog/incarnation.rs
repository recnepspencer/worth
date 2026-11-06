//! Reject an old Home claim before catalog lookup or materialization.
use super::ProductUnpublishedRecoveryCatalog;
use crate::recovery::{ProductUnpublishedRecoveryHandle, RuntimeWorldRecoveryDenial as Denial};

impl ProductUnpublishedRecoveryCatalog {
    pub(crate) fn admit_handle_incarnation(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<(), worth_proof::HomeIncarnationDenial> {
        self.locked_state()
            .owner
            .admit_incarnation(handle.identity().owner_identity())
    }
    pub(crate) fn admit_handle(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<(), Denial> {
        self.admit_handle_incarnation(handle)
            .map_err(|cause| match cause {
                worth_proof::HomeIncarnationDenial::ForeignHome => Denial::ForeignHandle,
                worth_proof::HomeIncarnationDenial::StaleIncarnation => Denial::StaleIncarnation,
            })?;
        if handle.catalog_affinity() != self.affinity() {
            return Err(Denial::ForeignHandle);
        }
        Ok(())
    }
}
