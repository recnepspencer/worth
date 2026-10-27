//! Explicit observer copies retain and extend the original source metadata grant.
use super::*;
use worth_execution::ExecutionRequest;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

impl SourceFacts {
    pub(in crate::domain_computation::primary_graph) fn copy_for_observer(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Cause> {
        if let Some(allocation) = &self.allocation {
            allocation.claim_slice::<EntityId>(execution, self.entities.len())?;
            allocation
                .claim_slice::<WorthQueryObservedFieldRevision>(execution, self.aspects.len())?;
            allocation.claim_slice::<WorthQueryObservedAdjacencyRevision>(
                execution,
                self.adjacencies.len(),
            )?;
            for aspect in &self.aspects {
                allocation.claim(execution, aspect.owned_capacity_bytes() as u64)?;
            }
            for adjacency in &self.adjacencies {
                allocation.claim(execution, adjacency.owned_capacity_bytes() as u64)?;
            }
        }
        Ok(Self {
            entities: self.entities.clone(),
            aspects: self.aspects.clone(),
            adjacencies: self.adjacencies.clone(),
            walks: self.walks.clone(),
            allocation: self.allocation.clone(),
        })
    }

    pub(in crate::domain_computation::primary_graph) fn admit_container<T>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Cause> {
        if let Some(allocation) = &self.allocation {
            allocation.claim(execution, (size_of::<T>() + 2 * size_of::<usize>()) as u64)?;
        }
        Ok(())
    }
}
