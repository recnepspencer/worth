//! Admit source fact growth and explicit predecessor copies against one original grant.
use super::*;
use crate::domain_computation::primary_graph::source_facts::allocation::tree_clone_bytes;

impl SourceFactsBuilder {
    pub(in crate::domain_computation::primary_graph) fn protect(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<(), SourceFactReadDenial> {
        match &self.allocation {
            Some(original) if Arc::ptr_eq(original, allocation) => return Ok(()),
            Some(_) => return Err(SourceFactReadDenial::Unavailable),
            None => {}
        }
        allocation
            .claim(
                execution,
                self.copy_bytes().map_err(SourceFactReadDenial::Execution)?,
            )
            .map_err(SourceFactReadDenial::Execution)?;
        self.allocation = Some(Arc::clone(allocation));
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn copy_for_step(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Cause> {
        if let Some(allocation) = &self.allocation {
            allocation.claim(execution, self.copy_bytes()?)?;
        }
        Ok(Self {
            entities: self.entities.clone(),
            aspects: self.aspects.clone(),
            adjacencies: self.adjacencies.clone(),
            walks: self.walks.clone(),
            allocation: self.allocation.clone(),
        })
    }

    fn copy_bytes(&self) -> Result<u64, Cause> {
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
            .ok_or(Cause::ChargedBytesOverflow)?;
        for ((_, aspect, field), value) in &self.aspects {
            bytes = bytes
                .checked_add(value.owned_capacity_bytes() as u64)
                .and_then(|n| n.checked_add(aspect.owned_allocation_capacity_bytes() as u64))
                .and_then(|n| n.checked_add(field.owned_allocation_capacity_bytes() as u64))
                .ok_or(Cause::ChargedBytesOverflow)?;
        }
        for value in self.adjacencies.values() {
            bytes = bytes
                .checked_add(value.owned_capacity_bytes() as u64)
                .ok_or(Cause::ChargedBytesOverflow)?;
        }
        Ok(bytes)
    }

    pub(super) fn admit_field_record(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        value: &WorthQueryObservedFieldRevision,
    ) -> Result<(), Cause> {
        if let Some(allocation) = &self.allocation {
            let bytes = value
                .owned_capacity_bytes()
                .checked_add(value.aspect.owned_allocation_capacity_bytes())
                .and_then(|n| n.checked_add(value.field.owned_allocation_capacity_bytes()))
                .and_then(|n| u64::try_from(n).ok())
                .ok_or(Cause::ChargedBytesOverflow)?;
            allocation.claim(execution, bytes)?;
            allocation.claim_tree::<(
                (EntityId, AspectKey, FieldKey),
                WorthQueryObservedFieldRevision,
            )>(execution, self.aspects.len())?;
        }
        Ok(())
    }

    pub(super) fn admit_field_lookup(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        aspect: &AspectKey,
        field: &FieldKey,
    ) -> Result<(), Cause> {
        if let Some(allocation) = &self.allocation {
            let bytes = aspect
                .owned_allocation_capacity_bytes()
                .checked_add(field.owned_allocation_capacity_bytes())
                .and_then(|n| u64::try_from(n).ok())
                .ok_or(Cause::ChargedBytesOverflow)?;
            allocation.claim(execution, bytes)?;
        }
        Ok(())
    }

    pub(super) fn admit_adjacency_record(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        value: &WorthQueryObservedAdjacencyRevision,
    ) -> Result<(), Cause> {
        if let Some(allocation) = &self.allocation {
            allocation.claim(execution, value.owned_capacity_bytes() as u64)?;
            allocation
                .claim_tree::<((EntityId, KindId, u8), WorthQueryObservedAdjacencyRevision)>(
                    execution,
                    self.adjacencies.len(),
                )?;
        }
        Ok(())
    }

    pub(super) fn admit_guard_field(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        entity_name: &str,
        aspect: &AspectKey,
        field: &FieldKey,
    ) -> Result<(), Cause> {
        if let Some(allocation) = &self.allocation {
            let names = aspect
                .owned_allocation_capacity_bytes()
                .checked_add(field.owned_allocation_capacity_bytes())
                .and_then(|n| u64::try_from(n).ok())
                .ok_or(Cause::ChargedBytesOverflow)?;
            // Index keys, stored/returned names and the temporary field locator.
            let bytes = names
                .checked_mul(6)
                .and_then(|n| n.checked_add(2 * entity_name.len() as u64))
                .and_then(|n| {
                    n.checked_add((size_of::<AspectFieldLocator>() + usize::BITS as usize) as u64)
                })
                .ok_or(Cause::ChargedBytesOverflow)?;
            allocation.claim(execution, bytes)?;
            allocation.claim_tree::<(
                (EntityId, AspectKey, FieldKey),
                WorthQueryObservedFieldRevision,
            )>(execution, self.aspects.len())?;
        }
        Ok(())
    }
}
