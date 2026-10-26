//! Pre-admit copied source metadata, retaining its original read-batch grant.
use super::*;
use crate::domain_computation::primary_graph::application_query::observed_source::{
    tree_clone_bytes, RootPathAllocation,
};
use worth_execution::ExecutionRequest;

impl RootPathSourceBuilder {
    pub(in crate::domain_computation::primary_graph::application_query::read_execution::root_selection) fn protect(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        allocation: &Arc<RootPathAllocation>,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        match &self.allocation {
            Some(original) if Arc::ptr_eq(original, allocation) => return Ok(()),
            Some(_) => return Err(source_denial("foreign source backing")),
            None => {}
        }
        // Adopt a legacy prefix for retained handoff. New closure/following-step
        // allocations are then admitted before they occur.
        allocation.claim(execution, self.copy_bytes()?)?;
        self.allocation = Some(Arc::clone(allocation));
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::application_query::read_execution::root_selection) fn copy_for_step(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        allocation: Option<&Arc<RootPathAllocation>>,
    ) -> Result<Self, WorthQueryApplicationReadExecutionDenial> {
        let allocation = allocation.or(self.allocation.as_ref());
        if let Some(allocation) = allocation {
            allocation.claim(execution, self.copy_bytes()?)?;
        }
        Ok(Self {
            entities: self.entities.clone(),
            aspects: self.aspects.clone(),
            adjacencies: self.adjacencies.clone(),
            walks: self.walks.clone(),
            allocation: allocation.cloned(),
        })
    }

    fn copy_bytes(&self) -> Result<u64, WorthQueryApplicationReadExecutionDenial> {
        let mut bytes = tree_clone_bytes::<(EntityId, ())>(self.entities.len())
            .and_then(|n| {
                n.checked_add(tree_clone_bytes::<(
                    (EntityId, AspectKey, FieldKey),
                    WorthQueryObservedFieldRevision,
                )>(self.aspects.len())?)
            })
            .and_then(|n| {
                n.checked_add(tree_clone_bytes::<(
                    (EntityId, KindId, u8),
                    WorthQueryObservedAdjacencyRevision,
                )>(self.adjacencies.len())?)
            })
            .ok_or_else(|| source_denial("source copy capacity"))?;
        for ((_, aspect, field), value) in &self.aspects {
            bytes = bytes
                .checked_add(value.owned_capacity_bytes() as u64)
                .and_then(|n| n.checked_add(aspect.owned_allocation_capacity_bytes() as u64))
                .and_then(|n| n.checked_add(field.owned_allocation_capacity_bytes() as u64))
                .ok_or_else(|| source_denial("source copy capacity"))?;
        }
        for value in self.adjacencies.values() {
            let bytes_for_endpoints = value
                .endpoints
                .capacity()
                .checked_mul(size_of::<EntityId>())
                .and_then(|n| u64::try_from(n).ok())
                .ok_or_else(|| source_denial("source copy capacity"))?;
            bytes = bytes
                .checked_add(bytes_for_endpoints)
                .ok_or_else(|| source_denial("source copy capacity"))?;
        }
        Ok(bytes)
    }
}
