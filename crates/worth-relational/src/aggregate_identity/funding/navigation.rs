//! Scoped partition and occupied-slot navigation keeps original funding on every exit.
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
        let loan = NavigationLoan {
            work: self,
            backing: Some(ids),
            held,
        };
        visit(
            loan.backing.as_deref().expect("partition backing retained"),
            &mut *loan.work,
        )
    }

    pub(in crate::aggregate_identity) fn visit_entity_slots(
        &mut self,
        state: &crate::validation::engine::state_view::InvariantStateView<'_>,
        partition: PartitionId,
        mut visit: impl FnMut(usize, &mut Self) -> BuildResult<()>,
    ) -> BuildResult<()> {
        let before = self.claimed;
        let slots = state.try_entity_slots(partition, |units, bytes| {
            self.checkpoint(units)?;
            self.retain(bytes)
        });
        let held = self.claimed - before;
        let mut loan = NavigationLoan {
            work: self,
            backing: Some(slots),
            held,
        };
        match loan.backing.as_mut().expect("slot preparation retained") {
            Ok(slots) => {
                for slot in slots {
                    visit(slot, &mut *loan.work)?;
                }
            }
            Err(_) => {
                let Some(Err(stop)) = loan.backing.take() else {
                    unreachable!("failed slot preparation retained its original stop");
                };
                return Err(stop);
            }
        }
        Ok(())
    }
}

struct NavigationLoan<'work, 'context, 'scope, 'authority, 'memory, T> {
    work: &'work mut AggregateWork<'context, 'scope, 'authority, 'memory>,
    backing: Option<T>,
    held: u64,
}
impl<T> Drop for NavigationLoan<'_, '_, '_, '_, '_, T> {
    fn drop(&mut self) {
        // Drop backing before shrinking its exact original loan, also on panic.
        drop(self.backing.take());
        self.work.claimed -= self.held;
        if let Some(memory) = self.work.memory.as_mut() {
            memory
                .resize(self.work.claimed)
                .expect("releasing admitted partition navigation cannot refuse");
        }
    }
}
