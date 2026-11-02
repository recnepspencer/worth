//! Identity-bound lookup custody releases every refused fork's staged metadata.
use super::*;
use std::sync::Weak;
use worth_relational::facade::{history::BranchId, mvcc::ManagedIndexSourceForkPreparation};

#[cfg(test)]
mod tests;

struct PreparedLookup {
    registry: Weak<ManagedDerivedViewRegistry>,
    branch: BranchId,
    slot: worth_relational::facade::mvcc::CompanionBranchCellSlot<ManagedIndexBranchImage>,
    _allocation: Arc<crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation>,
}
impl Drop for PreparedLookup {
    fn drop(&mut self) {
        if self.slot.admitted().is_some() {
            return;
        }
        let Some(registry) = self.registry.upgrade() else {
            return;
        };
        let mut state = registry
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let same = state
            .source_branches
            .get(&self.branch)
            .is_some_and(|entry| entry.slot.same_registration(&self.slot));
        let retired = same.then(|| state.source_branches.remove(&self.branch));
        let empty = state
            .source_branches
            .is_empty()
            .then(|| std::mem::take(&mut state.source_branches));
        drop(state);
        drop(empty);
        drop(retired);
    }
}
impl ManagedDerivedViewRegistry {
    pub(in crate::domain_computation::primary_graph) fn prepare_source_fork(
        self: &Arc<Self>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: &ManagedIndexSourceForkPreparation<'_>,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>, Stop> {
        if self.native_runtime != Some(target.runtime_instance_id()) {
            return Err(Stop::ForeignCell);
        }
        execution
            .consult()
            .map_err(|cause| Stop::Execution(cause.into()))?;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.source_branches.contains_key(target.target_branch()) {
            return Err(Stop::TopologyPending);
        }
        let slot = target.lookup_slot();
        let entry = SourceBranchSlot::new(
            execution,
            target.target_branch(),
            slot.clone(),
            state.source_branches.len(),
        )
        .map_err(Stop::Execution)?;
        entry
            .allocation
            .claim(
                execution,
                u64::try_from(
                    size_of::<PreparedLookup>()
                        .checked_add(target.target_branch().0.len())
                        .ok_or(Stop::PreparationMemoryCounterOverflow)?,
                )
                .map_err(|_| Stop::PreparationMemoryCounterOverflow)?,
            )
            .map_err(Stop::Execution)?;
        let lookup = PreparedLookup {
            registry: Arc::downgrade(self),
            branch: target.target_branch().clone(),
            slot,
            _allocation: Arc::clone(&entry.allocation),
        };
        state
            .source_branches
            .insert(target.target_branch().clone(), entry);
        drop(state);
        Ok(Box::new(lookup))
    }
}
