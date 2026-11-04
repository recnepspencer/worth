//! Retirement of the actual extracted head, outside the branch registry.
use super::ProductBranchReferenceCell;

impl ProductBranchReferenceCell {
    pub(crate) fn snapshot_for_close(&self) -> Result<super::ProductBranchReferenceSnapshot, ()> {
        Ok(self
            .state
            .current
            .try_read()
            .map_err(|_| ())?
            .snapshot
            .clone())
    }

    pub(crate) fn try_retire_for_close(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::lifecycle::RuntimeWorldCloseDenial> {
        let mut image = self
            .state
            .current
            .try_write()
            .map_err(|_| crate::lifecycle::RuntimeWorldCloseDenial::Busy)?;
        if let Some(protection) = image.protection.as_mut() {
            protection
                .try_retire_components_for_close()
                .map_err(|_| crate::lifecycle::RuntimeWorldCloseDenial::CleanupRequired)?;
            request
                .check_live()
                .map_err(crate::lifecycle::RuntimeWorldCloseDenial::Stopped)?;
            protection
                .try_retire_history_for_close()
                .map_err(|_| crate::lifecycle::RuntimeWorldCloseDenial::Busy)?;
        }
        // Both original claims and the exact history protection are disarmed.
        // The snapshot's immutable commit remains in the retained history slot.
        let retired = image.protection.take();
        drop(image);
        drop(retired);
        Ok(())
    }
}

impl ProductBranchReferenceCell {
    pub(crate) fn try_begin_home_basis_retirement(
        &self,
    ) -> Result<crate::history::CompositeCommitRetirement, crate::lifecycle::RuntimeWorldCloseDenial>
    {
        let mut image = self
            .state
            .current
            .try_write()
            .map_err(|_| crate::lifecycle::RuntimeWorldCloseDenial::Busy)?;
        Ok(image.snapshot.begin_home_retirement())
    }
}
