//! One funded graph boundary; publication requires the original custody owner.
use super::{
    basis, traversal::Read, RelationalModelOwnership, RelationalOwnedModelEntityBoundary,
    RelationalOwnedModelForeignRelation, RelationalOwnedModelSelectionDenial as Denial,
};
use crate::{
    branch::AdmittedRelationalBranchBasis,
    identity::data::{EntityId, RelationId},
    runtime::RelationalRuntime,
    validation::data::StructuralRelationRecord,
};
use std::collections::BTreeMap;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelContext};

/// The declared structural part of an owned model selection. Original bases
/// retain payloads; publication custody is a required additional phase. This
/// material grants no mutation authority. Ordinary incoming relations from
/// outside this entity boundary never enter either restoration roster.
pub struct RelationalOwnedModelGraphSelection {
    entities: RelationalOwnedModelEntityBoundary,
    current: BTreeMap<RelationId, StructuralRelationRecord>,
    selected: BTreeMap<RelationId, StructuralRelationRecord>,
    _current_memory: Vec<ExecutionMemoryReservation>,
    _selected_memory: Vec<ExecutionMemoryReservation>,
    _validation_memory: Vec<ExecutionMemoryReservation>,
}
impl std::fmt::Debug for RelationalOwnedModelGraphSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelationalOwnedModelGraphSelection")
            .field("model", &self.entities.model())
            .field("current_relations", &self.current.len())
            .field("selected_relations", &self.selected.len())
            .finish_non_exhaustive()
    }
}
impl RelationalOwnedModelGraphSelection {
    pub fn entities(&self) -> &RelationalOwnedModelEntityBoundary {
        &self.entities
    }
    pub fn current_relations(&self) -> &BTreeMap<RelationId, StructuralRelationRecord> {
        &self.current
    }
    pub fn selected_relations(&self) -> &BTreeMap<RelationId, StructuralRelationRecord> {
        &self.selected
    }
    pub fn relations_to_retire(&self) -> impl Iterator<Item = RelationId> + '_ {
        self.current
            .keys()
            .filter(|id| !self.selected.contains_key(id))
            .copied()
    }
}

impl RelationalRuntime {
    pub fn prepare_owned_model_graph_selection(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        current: &AdmittedRelationalBranchBasis,
        selected: &AdmittedRelationalBranchBasis,
        ownership: &RelationalModelOwnership,
        installed_ownership: &RelationalModelOwnership,
        model: EntityId,
    ) -> Result<RelationalOwnedModelGraphSelection, Denial> {
        let mut result = None;
        execution
            .run_native_with_checkpoint(|context| {
                result = Some((|| {
                    let entities = self.prepare_entity_boundary(
                        execution, context, current, selected, ownership, model,
                    )?;
                    prepare(self, execution, context, entities, installed_ownership)
                })());
            })
            .map_err(|cause| {
                Denial::Ownership(super::RelationalModelOwnershipDenial::Request(cause))
            })?;
        result.expect("completed graph selection preparation")
    }
}

fn in_union(entities: &RelationalOwnedModelEntityBoundary, id: EntityId) -> bool {
    entities.current_entities().contains(&id) || entities.selected_entities().contains(&id)
}
fn declared(
    read: &mut basis::BasisRead<'_, '_, '_, '_, '_, '_>,
    ownership: &RelationalModelOwnership,
    edge: StructuralRelationRecord,
) -> Result<bool, Denial> {
    read.charge(ownership.relation_lookup_work())?;
    Ok(ownership.declares_relation(edge.kind_id))
}
fn insert(
    read: &mut basis::BasisRead<'_, '_, '_, '_, '_, '_>,
    map: &mut BTreeMap<RelationId, StructuralRelationRecord>,
    edge: StructuralRelationRecord,
) -> Result<(), Denial> {
    if !map.contains_key(&edge.relation_id) {
        read.retain()?;
        map.insert(edge.relation_id, edge);
    }
    Ok(())
}
fn prepare(
    native: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
    entities: RelationalOwnedModelEntityBoundary,
    installed: &RelationalModelOwnership,
) -> Result<RelationalOwnedModelGraphSelection, Denial> {
    let current = entities.current_basis();
    let selected = entities.selected_basis();
    let (current_relations, current_memory) =
        basis::read(native, execution, current, context, |read| {
            let mut map = BTreeMap::new();
            for entity in entities.entities() {
                for edge in read.relations(entity, true)? {
                    insert(read, &mut map, edge)?;
                }
                for edge in read.relations(entity, false)? {
                    if declared(read, installed, edge)? {
                        insert(read, &mut map, edge)?;
                    } else if entities.current_entities().contains(&entity)
                        && !entities.selected_entities().contains(&entity)
                        && !in_union(&entities, edge.source)
                    {
                        return Err(Denial::ForeignIncomingRelation(
                            RelationalOwnedModelForeignRelation::new(current, entity, edge)?,
                        ));
                    }
                }
            }
            Ok::<_, Denial>(map)
        })?;
    let (selected_relations, selected_memory) =
        basis::read(native, execution, selected, context, |read| {
            let mut map = BTreeMap::new();
            for &entity in entities.selected_entities() {
                for edge in read.relations(entity, true)? {
                    insert(read, &mut map, edge)?;
                }
                for edge in read.relations(entity, false)? {
                    if declared(read, installed, edge)? {
                        insert(read, &mut map, edge)?;
                    }
                }
            }
            Ok::<_, Denial>(map)
        })?;
    let ((), validation_memory) = basis::read(native, execution, current, context, |read| {
        for edge in selected_relations.values() {
            if let Some(now) = read.relation(edge.relation_id)? {
                if !in_union(&entities, now.source)
                    && !(declared(read, installed, now)? && in_union(&entities, now.target))
                {
                    return Err(Denial::ForeignRelationSource {
                        relation: now.relation_id,
                        owner: now.source,
                    });
                }
            }
            for endpoint in [edge.source, edge.target] {
                if !entities.selected_entities().contains(&endpoint)
                    && (entities.current_entities().contains(&endpoint)
                        || read.kind(endpoint)?.is_none())
                {
                    return Err(Denial::SelectedEndpointUnavailable {
                        relation: edge.relation_id,
                        endpoint,
                    });
                }
            }
        }
        Ok::<_, Denial>(())
    })?;
    Ok(RelationalOwnedModelGraphSelection {
        entities,
        current: current_relations,
        selected: selected_relations,
        _current_memory: current_memory,
        _selected_memory: selected_memory,
        _validation_memory: validation_memory,
    })
}
