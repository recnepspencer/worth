//! A declared create/preserve pair follows its actual publication positions.
use super::*;
use std::cmp::Reverse;
use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;

type Publication<'a> = ((Reverse<usize>, u64), &'a RecordedOutput);

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn resolve_binding_succession<
        Preserve: 'static,
        Create: 'static,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        scope: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        partition: [u8; 32],
    ) -> Result<WorthQueryPriorOutputBindingResolution, Stop> {
        execution
            .consult()
            .map_err(|cause| Stop::Execution(cause.into()))?;
        let preserve = TypeId::of::<Preserve>();
        let create = TypeId::of::<Create>();
        if preserve == create
            || !self.output_families.values().any(|bindings| {
                bindings.iter().any(|(binding, _)| *binding == preserve)
                    && bindings.iter().any(|(binding, _)| *binding == create)
            })
        {
            return Err(Stop::SelectedSourceMismatch);
        }
        let coordinate = ProductCoordinate {
            occurrence,
            generation,
        };
        let left = self.binding_publication(execution, scope, coordinate, partition, preserve)?;
        let right = self.binding_publication(execution, scope, coordinate, partition, create)?;
        let selected = match (left, right) {
            (Some(left), Some(right)) => match left.0.cmp(&right.0) {
                std::cmp::Ordering::Greater => Some(left.1),
                std::cmp::Ordering::Less => Some(right.1),
                std::cmp::Ordering::Equal => return Err(Stop::SelectedSourceMismatch),
            },
            (Some((_, row)), None) | (None, Some((_, row))) => Some(row),
            (None, None) => None,
        };
        Ok(WorthQueryPriorOutputBindingResolution {
            correspondence: selected.map(|row| Arc::clone(&row.correspondence)),
        })
    }

    fn binding_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        scope: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
        mut coordinate: ProductCoordinate,
        partition: [u8; 32],
        binding: TypeId,
    ) -> Result<Option<Publication<'_>>, Stop> {
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding: self
                .binding_identity(binding)
                .ok_or(Stop::SelectedSourceMismatch)?,
        };
        let mut depth = 0usize;
        loop {
            if let Some((generation, slot)) = self
                .partition_index
                .latest_admitted(execution, &source, coordinate, partition)?
            {
                let row = self.recorded_at_partition_slot(
                    &source,
                    coordinate.occurrence,
                    generation,
                    Some(partition),
                    slot,
                );
                // Stable aliases may retain an older settlement. The locator,
                // not that inherited settlement, is the publication position.
                return Ok(Some(((Reverse(depth), generation), row)));
            }
            let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                return Ok(None);
            };
            coordinate = parent;
            depth = depth.checked_add(1).ok_or(Stop::WorkCounterOverflow)?;
        }
    }
}
