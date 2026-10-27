//! Scoped partition navigation keeps original funding on every exit.
use super::{AggregateWork, BuildResult, PlanDenial};
use crate::{identity::data::PartitionId, storage::overlay::PartitionAccess};
use worth_execution::MapKernelFailure;

impl AggregateWork<'_, '_, '_, '_> {
    pub(in crate::aggregate_identity) fn with_partition_ids<T>(
        &mut self,
        state: &dyn PartitionAccess,
        visit: impl FnOnce(&[PartitionId], &mut Self) -> BuildResult<T>,
    ) -> BuildResult<T> {
        self.checkpoint(1)?;
        let (ids, held) = crate::storage::overlay::partition_ids_in_result_memory(
            state,
            self.memory.as_deref_mut(),
        )
        .map_err(|cause| MapKernelFailure::Domain(PlanDenial::Execution(cause)))?;
        self.claimed = self
            .claimed
            .checked_add(held)
            .expect("original memory admitted total");
        let loan = PartitionIdLoan {
            work: self,
            ids,
            held,
        };
        visit(&loan.ids, &mut *loan.work)
    }
}

struct PartitionIdLoan<'work, 'context, 'scope, 'authority, 'memory> {
    work: &'work mut AggregateWork<'context, 'scope, 'authority, 'memory>,
    ids: Vec<PartitionId>,
    held: u64,
}
impl Drop for PartitionIdLoan<'_, '_, '_, '_, '_> {
    fn drop(&mut self) {
        // Drop backing before shrinking its exact original loan, also on panic.
        drop(std::mem::take(&mut self.ids));
        self.work.claimed -= self.held;
        if let Some(memory) = self.work.memory.as_mut() {
            memory
                .resize(self.work.claimed)
                .expect("releasing admitted partition navigation cannot refuse");
        }
    }
}
