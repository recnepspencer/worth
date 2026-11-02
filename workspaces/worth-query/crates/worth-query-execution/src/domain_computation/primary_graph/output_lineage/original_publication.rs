//! Original receipt data selects existing owner custody; it creates no output.
use super::*;
use crate::domain_computation::authorization::WorthQueryOperationScopeBinding;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationIdempotencyBinding, WorthQueryRecoveredApplicationCommitReceipt,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
use worth_runtime_world::facade::ProductBranchObservation;

impl WorthQueryApplicationOutputLineage {
    /// Resolve an exact original slot against this owner's retained native
    /// correspondence. Stable consumption still admits currentness/disclosure.
    pub(in crate::domain_computation::primary_graph) fn resolve_original_publication<
        Binding: 'static,
    >(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: &WorthQueryOperationScopeBinding,
        observation: &ProductBranchObservation,
        binding: WorthQueryApplicationIdempotencyBinding,
        receipt: &WorthQueryRecoveredApplicationCommitReceipt,
    ) -> Result<Option<PublishedStableLineage>, Stop> {
        use worth_foundational::facade::{
            DurableQueryOutputSettlement, RecordedProductBranchReference,
        };
        execution
            .consult()
            .map_err(|stop| Stop::Execution(stop.into()))?;
        let (generation, same_occurrence) = match receipt.new_product_head() {
            RecordedProductBranchReference::Home(head) => (
                head.generation().get(),
                observation.owner_identity().durable() == Some(head.branch().owner())
                    && observation.branch_identity().name().as_str() == head.branch().name()
                    && observation.lifecycle_incarnation().ordinal()
                        == head.incarnation().ordinal(),
            ),
            RecordedProductBranchReference::Memory(head) => (
                head.generation(),
                observation.owner_identity().durable().is_none()
                    && observation.owner_identity().get() == head.owner()
                    && observation.branch_identity().name().as_str() == head.branch()
                    && observation.lifecycle_incarnation().ordinal() == head.incarnation(),
            ),
        };
        if !same_occurrence
            || generation > observation.reference_generation().get()
            || receipt.idempotency_key_identity() != binding.key_identity()
        {
            return Ok(None);
        }
        let DurableQueryOutputSettlement::OriginalOccurrenceSlot { slot } =
            receipt.exact_output_settlement()
        else {
            return Ok(None);
        };
        let Some(partition) = binding.source_partition_identity() else {
            return Ok(None);
        };
        let Some(output_binding) = self.binding_identity(TypeId::of::<Binding>()) else {
            return Ok(None);
        };
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding,
        };
        let expected_coordinate = ProductCoordinate {
            occurrence: observation.lifecycle_incarnation(),
            generation,
        };
        let mut coordinate = ProductCoordinate {
            occurrence: observation.lifecycle_incarnation(),
            generation: observation.reference_generation().get(),
        };
        loop {
            execution
                .consult()
                .map_err(|stop| Stop::Execution(stop.into()))?;
            let selected = self
                .partition_index
                .latest_admitted(execution, &source, coordinate, partition)?
                .map(|(generation, slot)| {
                    self.cell_at_partition_slot(
                        &source,
                        coordinate.occurrence,
                        generation,
                        Some(partition),
                        slot,
                    )
                });
            if let Some(selected) = selected {
                let Some(recorded) = selected.get() else {
                    return Ok(None);
                };
                // A retained alias owns its actual performed origin even if an
                // old derived generation/index entry has already retired.
                let origin_cell = recorded.performed_origin.as_ref().unwrap_or(selected);
                let Some(original) = origin_cell.get() else {
                    return Ok(None);
                };
                if original.performed_origin.is_some()
                    || !self.implementation_is_current(original)
                    || original.settlement_identity.source() != &source
                    || original.settlement_identity.coordinate() != expected_coordinate
                    || usize::try_from(slot).ok() != Some(original.settlement_identity.slot())
                    || original.idempotency_key_identity != *receipt.idempotency_key_identity()
                    || original.source_partition_identity != Some(partition)
                    || original.producer_dependency_identity
                        != binding.producer_dependency_identity()
                    || original.correspondence.binding_type() != Some(TypeId::of::<Binding>())
                    || original
                        .native_output_witness_cell()
                        .and_then(|cell| cell.get())
                        .is_none()
                {
                    return Ok(None);
                }
                return Ok(self
                    .implementation_is_current(recorded)
                    .then(|| PublishedStableLineage::selected_at(selected, observation)));
            }
            let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                return Ok(None);
            };
            coordinate = parent;
        }
    }
}

#[cfg(test)]
mod tests;
