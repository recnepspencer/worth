//! Rebuild selected history in an unexposed owner; never execute a live CAS.
use super::*;
impl ProductBranchReferenceCell {
    pub(crate) fn restore_readmitted(
        &self,
        record: &crate::history::ReadmittedPublicationRecord,
        successor: ProductBranchHeadProtection,
    ) -> Result<(), ProductBranchHeadProtection> {
        let mut current = self.state.write();
        if current.snapshot != record.before
            || successor.snapshot() != &record.after
            || current.movement.denial().is_some()
            || current.protection.is_none()
            || current.snapshot.generation().advance().ok() != Some(record.after.generation())
        {
            return Err(successor);
        }
        let retired = current.protection.replace(successor);
        current.snapshot = record.after.clone();
        // Existing seat accounting follows the selected recovered head.
        if let Some(protection) = current.protection.as_mut() {
            protection.seat_history(true);
        }
        drop(current);
        drop(retired);
        Ok(())
    }

    pub(crate) fn restore_readmitted_horizon(
        &self,
        before: &ProductBranchReferenceSnapshot,
        successor: ProductBranchHeadProtection,
    ) -> Result<(), ProductBranchHeadProtection> {
        let mut current = self.state.write();
        if current.snapshot != *before
            || current.movement.denial().is_some()
            || current.protection.is_none()
            || successor.snapshot().generation().get() <= current.snapshot.generation().get()
        {
            return Err(successor);
        }
        let after = successor.snapshot().clone();
        let retired = current.protection.replace(successor);
        current.snapshot = after;
        if let Some(protection) = current.protection.as_mut() {
            protection.seat_history(true);
        }
        drop(current);
        drop(retired);
        Ok(())
    }
}
