//! Nonwaiting discharge of the original C.10 slot reference.
use super::super::PhysicalRootReadRetirementDenial as Failure;

#[cfg(test)]
mod tests;
use super::*;

impl RootProtectionRegistry {
    pub(in crate::physical_runtime::stability::retention) fn try_release(
        &self,
        slot: usize,
        root: RootPublicationCell,
        armed: &mut bool,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), Failure> {
        request.consult().map_err(Failure::Execution)?;
        let mut state = match self.state.try_lock() {
            Ok(state) => state,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(Failure::Busy),
        };
        // This is the actual privately issued lease, not a caller description.
        // Its slot cannot be recycled while this reference remains armed.
        let entry = &state.slots[slot];
        assert_eq!(
            entry.root,
            Some(root),
            "retirement releases its original slot"
        );
        assert!(
            entry.owners > 0,
            "retirement owns an original slot reference"
        );
        request.consult().map_err(Failure::Execution)?;
        Self::release_registered(&mut state, slot, root);
        *armed = false;
        Ok(())
    }
}
