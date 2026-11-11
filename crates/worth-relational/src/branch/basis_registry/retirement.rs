//! Exact-registration removal retaining the original index storage until reclaim.
use super::*;
use crate::history::retention::RelationalRetentionRetirementError;
use worth_execution::ExecutionRequest;
#[cfg(test)]
mod tests;
#[cfg(test)]
pub(crate) use tests::{
    assert_original_registry_busy, assert_pruned_original_registration,
    assert_successor_registration_preserved,
};

#[derive(Debug, Default)]
pub(crate) struct RelationalBasisRegistryRetirementCustody {
    registry: Option<Arc<Mutex<RelationalRetainedBasisState>>>,
    metrics: Option<Arc<RelationalBranchBasisRegistryMetrics>>,
    removed: Option<(RelationalRetainedBasisKey, RelationalRetainedBasisEntry)>,
}

impl RelationalBasisRegistryLease {
    pub(crate) fn try_retire_retaining(
        &mut self,
        custody: &mut RelationalBasisRegistryRetirementCustody,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalRetentionRetirementError> {
        if self.retired {
            return Ok(());
        }
        if custody.registry.is_none() {
            custody.registry = Some(
                self.registry
                    .upgrade()
                    .ok_or(RelationalRetentionRetirementError::OwnerUnavailable)?,
            );
        }
        if custody.metrics.is_none() {
            custody.metrics = Some(
                self.metrics
                    .upgrade()
                    .ok_or(RelationalRetentionRetirementError::OwnerUnavailable)?,
            );
        }
        let registry = custody.registry.as_ref().expect(/* Proof: The preceding upgrade installs the original registry or returns without effects. */ "retirement retains the registry");
        let metrics = custody.metrics.as_ref().expect(/* Proof: The preceding upgrade installs the original metrics or returns without effects. */ "retirement retains the metrics");
        let mut state = match registry.try_lock() {
            Ok(value) => value,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => {
                return Err(RelationalRetentionRetirementError::Busy)
            }
        };
        request
            .consult()
            .map_err(RelationalRetentionRetirementError::Stopped)?;
        metrics.record_lookup();
        // Extraction makes the original Weak dead. Ordinary readmission can
        // already have pruned it or installed a successor at this same key.
        if state
            .entries
            .get(&self.key)
            .is_some_and(|entry| entry.registration_id == self.registration_id)
        {
            custody.removed = state.entries.remove_entry(&self.key);
            metrics.record_remove();
        }
        self.retired = true;
        Ok(())
    }
}
