//! Original grants follow actual shared registry backing through its last owner.
use super::*;
use crate::domain_computation::primary_graph::application_query::one_shot::outcome::material::MaterialCompletion;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

impl RegistryState {
    pub(super) fn admit_index(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<(), Cause> {
        let storage = MaterialCompletion::new(execution, allocation);
        let work = self
            .views
            .len()
            .checked_add(self.owned.len())
            .and_then(|n| n.checked_add(self.material_allocations.len()))
            .and_then(|n| n.checked_add(1))
            .and_then(|n| n.checked_mul(size_of::<OwnedDerivedView>()))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::WorkCounterOverflow)?;
        storage.work(work)?;
        // Retain this header's original payer as well as common-container
        // growth. A dead Weak may still keep its Arc header allocated.
        if !self
            .material_allocations
            .iter()
            .any(|owner| Arc::ptr_eq(owner, allocation))
        {
            let count = self
                .material_allocations
                .len()
                .checked_add(1)
                .ok_or(Cause::ChargedBytesOverflow)?;
            allocation.claim_slice::<Arc<SourceFactsAllocation>>(execution, count)?;
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(count)
                .map_err(|_| Cause::AllocationFailed)?;
            replacement.extend(self.material_allocations.iter().cloned());
            replacement.push(Arc::clone(allocation));
            self.material_allocations = replacement;
        }
        allocation
            .claim_tree::<(u64, Weak<dyn RegisteredDerivedView>)>(execution, self.views.len())?;
        if self.owned.len() == self.owned.capacity() {
            let count = self
                .owned
                .len()
                .checked_add(1)
                .ok_or(Cause::ChargedBytesOverflow)?;
            // Full replacement span covers old/new backing coexistence.
            allocation.claim_slice::<(u64, OwnedDerivedView)>(execution, count)?;
            self.owned
                .try_reserve_exact(1)
                .map_err(|_| Cause::AllocationFailed)?;
        }
        Ok(())
    }

    pub(super) fn take_empty_material_storage(
        &mut self,
    ) -> Option<(
        BTreeMap<u64, Weak<dyn RegisteredDerivedView>>,
        Vec<(u64, OwnedDerivedView)>,
        Vec<Arc<SourceFactsAllocation>>,
    )> {
        (self.views.is_empty() && self.owned.is_empty()).then(|| {
            (
                std::mem::take(&mut self.views),
                std::mem::take(&mut self.owned),
                std::mem::take(&mut self.material_allocations),
            )
        })
    }
}
