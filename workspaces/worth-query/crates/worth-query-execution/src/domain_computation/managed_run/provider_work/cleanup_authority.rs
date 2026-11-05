use worth_runtime_bridge::facade::{
    BridgeBoundExecutionBasis, BridgeManagedQueueFailure, BridgeManagedQueueOccupancy,
};

use crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryGraphProviderMemoryArena;

use crate::domain_computation::vocabulary::managed_run::evidence::WorthQueryManagedProviderWorkEvidence;

#[derive(Default)]
pub(crate) struct WorthQueryManagedProviderCleanupAuthority {
    queue_occupancies: Vec<BridgeManagedQueueOccupancy>,
    provider_memories: Vec<WorthQueryGraphProviderMemoryArena>,
}

impl WorthQueryManagedProviderCleanupAuthority {
    pub(super) fn retain_queue_occupancy(&mut self, occupancy: BridgeManagedQueueOccupancy) {
        self.queue_occupancies.push(occupancy);
    }

    pub(super) fn retain_provider_memory(
        &mut self,
        memory: WorthQueryGraphProviderMemoryArena,
    ) -> usize {
        if memory.snapshot().retained_bytes() != 0 {
            self.provider_memories.push(memory);
        }
        self.provider_retained_bytes()
    }

    pub(super) fn provider_retained_bytes(&self) -> usize {
        self.provider_memories
            .iter()
            .map(|memory| as_usize(memory.snapshot().retained_bytes()))
            .fold(0usize, usize::saturating_add)
    }

    pub(crate) fn reconcile_provider_memory(&mut self) -> usize {
        self.provider_memories
            .retain(|memory| memory.snapshot().retained_bytes() != 0);
        self.provider_retained_bytes()
    }

    pub(crate) fn release_queue_occupancies(
        &mut self,
        basis: &mut BridgeBoundExecutionBasis,
        evidence: &mut WorthQueryManagedProviderWorkEvidence,
    ) -> Result<(), BridgeManagedQueueFailure> {
        while let Some(occupancy) = self.queue_occupancies.pop() {
            match basis.release_managed_queue_occupancy(occupancy) {
                Ok(mutation) => evidence.record_queue_mutation(mutation.counters()),
                Err(failure) => {
                    let cause = failure.failure().clone();
                    self.queue_occupancies.push(failure.into_occupancy());
                    return Err(cause);
                }
            }
        }
        Ok(())
    }
}

fn as_usize(bytes: u64) -> usize {
    usize::try_from(bytes).unwrap_or(usize::MAX)
}
