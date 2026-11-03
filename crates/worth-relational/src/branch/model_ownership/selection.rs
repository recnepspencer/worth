//! Two exact-basis entity closures; selection material grants no write authority.
use super::{basis, traversal, *};
use crate::{branch::AdmittedRelationalBranchBasis, runtime::RelationalRuntime};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
#[cfg(test)]
#[path = "selection_schema_tests.rs"]
mod schema_tests;

/// The entity boundary for an owned-model navigation. Both original bases and
/// charged result storage remain retained together. This is not a commit plan.
pub struct RelationalOwnedModelEntityBoundary {
    current: AdmittedRelationalBranchBasis,
    selected: AdmittedRelationalBranchBasis,
    model: EntityId,
    current_entities: BTreeSet<EntityId>,
    selected_entities: BTreeSet<EntityId>,
    _current_memory: Vec<ExecutionMemoryReservation>,
    _selected_memory: Vec<ExecutionMemoryReservation>,
}

impl RelationalOwnedModelEntityBoundary {
    pub fn model(&self) -> EntityId {
        self.model
    }
    pub fn current_basis(&self) -> &AdmittedRelationalBranchBasis {
        &self.current
    }
    pub fn selected_basis(&self) -> &AdmittedRelationalBranchBasis {
        &self.selected
    }
    pub fn current_entities(&self) -> &BTreeSet<EntityId> {
        &self.current_entities
    }
    pub fn selected_entities(&self) -> &BTreeSet<EntityId> {
        &self.selected_entities
    }
    /// Current-only entities retire; selected-only entities restore. Iteration
    /// allocates nothing and never adds the payload of an outside model.
    pub fn entities(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.current_entities
            .union(&self.selected_entities)
            .copied()
    }
}

impl RelationalRuntime {
    pub fn prepare_owned_model_entity_boundary(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        current: &AdmittedRelationalBranchBasis,
        selected: &AdmittedRelationalBranchBasis,
        ownership: &RelationalModelOwnership,
        model: EntityId,
    ) -> Result<RelationalOwnedModelEntityBoundary, RelationalModelOwnershipDenial> {
        // Both closure traversals spend one request meter, including on the
        // serial backend. Their retained results coexist under the same budget.
        let mut result = None;
        execution
            .run_native_with_checkpoint(|context| {
                result = Some(self.prepare_entity_boundary(
                    execution, context, current, selected, ownership, model,
                ));
            })
            .map_err(RelationalModelOwnershipDenial::Request)?;
        result.expect("completed entity boundary preparation")
    }

    fn prepare_entity_boundary(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        current: &AdmittedRelationalBranchBasis,
        selected: &AdmittedRelationalBranchBasis,
        ownership: &RelationalModelOwnership,
        model: EntityId,
    ) -> Result<RelationalOwnedModelEntityBoundary, RelationalModelOwnershipDenial> {
        use RelationalModelOwnershipDenial as Denial;
        for basis in [current, selected] {
            if basis.identity().runtime_instance_id() != self.runtime_instance_id() {
                return Err(Denial::ForeignBasis);
            }
        }
        if !current.is_current() {
            return Err(Denial::TargetNotCurrent);
        }
        require_same_schema(
            current.descriptor().schema_commitment(),
            selected.descriptor().schema_commitment(),
        )?;
        let ((kind, current_entities), current_memory) =
            basis::read(self, execution, current, context, |read| {
                let kind = read.kind(model)?.ok_or(Denial::EntityUnavailable(model))?;
                Ok((kind, ownership.owned_entities(read, model, kind)?))
            })?;
        let (selected_entities, selected_memory) =
            basis::read(self, execution, selected, context, |read| {
                let actual = read.kind(model)?.ok_or(Denial::EntityUnavailable(model))?;
                if actual != kind {
                    return Err(Denial::ModelRootKind { model, actual });
                }
                ownership.owned_entities(read, model, actual)
            })?;
        Ok(RelationalOwnedModelEntityBoundary {
            current: current.clone(),
            selected: selected.clone(),
            model,
            current_entities,
            selected_entities,
            _current_memory: current_memory,
            _selected_memory: selected_memory,
        })
    }
}

fn require_same_schema(
    current: [u8; 32],
    selected: [u8; 32],
) -> Result<(), RelationalModelOwnershipDenial> {
    if current != selected {
        return Err(RelationalModelOwnershipDenial::SchemaChanged);
    }
    Ok(())
}

impl RelationalModelOwnership {
    fn owned_entities(
        &self,
        read: &mut dyn traversal::Read,
        model: EntityId,
        kind: KindId,
    ) -> Result<BTreeSet<EntityId>, RelationalModelOwnershipDenial> {
        let mut entities = BTreeSet::new();
        let mut declared = false;
        for domain in self.domains.iter().filter(|domain| domain.root == kind) {
            declared = true;
            for entity in traversal::walk(read, model, true, &domain.relations)? {
                if !entities.contains(&entity) {
                    read.retain()?;
                    entities.insert(entity);
                }
            }
        }
        if !declared {
            return Err(RelationalModelOwnershipDenial::ModelRootKind {
                model,
                actual: kind,
            });
        }
        Ok(entities)
    }
}
