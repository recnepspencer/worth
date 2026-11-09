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
use std::collections::{BTreeMap, BTreeSet};
mod transfers;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelContext};

/// The declared structural part of an owned model selection. Original bases
/// retain payloads; publication custody is a required additional phase. This
/// material grants no mutation authority. Ordinary incoming relations from
/// outside this entity boundary never enter either restoration roster.
pub struct RelationalOwnedModelGraphSelection {
    pub(super) ownership: RelationalModelOwnership,
    pub(super) installed_ownership: RelationalModelOwnership,
    _meaning_memory: Option<ExecutionMemoryReservation>,
    entities: RelationalOwnedModelEntityBoundary,
    restoration: transfers::RestorationBoundary,
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
    /// Selected live records, including union members transferred to another owner.
    pub fn restored_entities(&self) -> &BTreeSet<EntityId> {
        &self.restoration.selected
    }
    pub fn entities_to_retire(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.restoration
            .current
            .difference(&self.restoration.selected)
            .copied()
    }
    pub fn current_entities(&self) -> &BTreeSet<EntityId> {
        &self.restoration.current
    }
    pub fn boundary_entities(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.restoration
            .current
            .union(&self.restoration.selected)
            .copied()
    }
    pub(crate) fn transfer_participants(&self) -> &BTreeSet<EntityId> {
        &self.restoration.participants
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
                    let bytes = ownership
                        .retention_bytes(context)?
                        .saturating_add(installed_ownership.retention_bytes(context)?);
                    crate::execution::request_stage::checkpoint(context, bytes).map_err(
                        |stop| {
                            Denial::Ownership(super::RelationalModelOwnershipDenial::Request(
                                worth_execution::WorkCeilingDenial::Stopped(stop),
                            ))
                        },
                    )?;
                    let meaning_memory =
                        execution.reserve_retained_memory(bytes).map_err(|stop| {
                            Denial::Ownership(super::RelationalModelOwnershipDenial::Request(stop))
                        })?;
                    prepare(
                        self,
                        execution,
                        context,
                        entities,
                        ownership,
                        installed_ownership,
                        meaning_memory,
                    )
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
    ownership: &RelationalModelOwnership,
    installed: &RelationalModelOwnership,
    meaning_memory: Option<ExecutionMemoryReservation>,
) -> Result<RelationalOwnedModelGraphSelection, Denial> {
    let current = entities.current_basis();
    let selected = entities.selected_basis();
    let restoration = transfers::prepare(native, execution, context, &entities, installed)?;
    let restored = &restoration.selected;
    let transferred = &restoration.transferred;
    let in_domain = |id: EntityId| restoration.current.contains(&id) || restored.contains(&id);
    let (current_relations, current_memory) =
        basis::read(native, execution, current, context, |read| {
            let mut map = BTreeMap::new();
            for &entity in restoration.current.union(restored) {
                let owned = if transferred.contains_key(&entity) {
                    read.custody_relation_ids(entity)?
                } else {
                    BTreeSet::new()
                };
                for edge in read.relations(entity, true)? {
                    read.charge(owned.len().checked_ilog2().unwrap_or(0) as usize + 1)?;
                    if !transferred.contains_key(&entity)
                        || declared(read, installed, edge)?
                        || owned.contains(&edge.relation_id)
                    {
                        insert(read, &mut map, edge)?;
                    }
                }
                for edge in read.relations(entity, false)? {
                    if declared(read, installed, edge)? {
                        insert(read, &mut map, edge)?;
                    } else if restoration.current.contains(&entity) && !restored.contains(&entity) {
                        let source_written = if !in_domain(edge.source) {
                            false
                        } else if transferred.contains_key(&edge.source) {
                            let owned = read.custody_relation_ids(edge.source)?;
                            read.charge(owned.len().checked_ilog2().unwrap_or(0) as usize + 1)?;
                            owned.contains(&edge.relation_id)
                        } else {
                            true
                        };
                        if !source_written {
                            return Err(Denial::ForeignIncomingRelation(
                                RelationalOwnedModelForeignRelation::new(current, entity, edge)?,
                            ));
                        }
                    }
                }
            }
            Ok::<_, Denial>(map)
        })?;
    let (selected_relations, selected_memory) =
        basis::read(native, execution, selected, context, |read| {
            let mut map = BTreeMap::new();
            for &entity in restored {
                if entities.selected_entities().contains(&entity) {
                    for edge in read.relations(entity, true)? {
                        insert(read, &mut map, edge)?;
                    }
                } else {
                    let owned = read.custody_relation_ids(entity)?;
                    for edge in read.relations(entity, true)? {
                        read.charge(owned.len().checked_ilog2().unwrap_or(0) as usize + 1)?;
                        if owned.contains(&edge.relation_id) {
                            insert(read, &mut map, edge)?;
                        }
                    }
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
                if !in_domain(now.source)
                    && !(declared(read, installed, now)? && in_domain(now.target))
                {
                    return Err(Denial::ForeignRelationSource {
                        relation: now.relation_id,
                        owner: now.source,
                    });
                }
            }
            if let Some(&owner) = transferred.get(&edge.target) {
                if !restored.contains(&edge.source) {
                    if read.kind(edge.source)?.is_none() {
                        return Err(Denial::SelectedOwnerUnavailable {
                            entity: edge.target,
                            owner,
                            source: edge.source,
                            relation: edge.relation_id,
                        });
                    }
                    let actual_owner = installed.owner(read, edge.source)?;
                    if actual_owner != Some(owner) {
                        return Err(Denial::SelectedOwnerChanged {
                            entity: edge.target,
                            owner,
                            source: edge.source,
                            actual_owner,
                            relation: edge.relation_id,
                        });
                    }
                }
            }
            for endpoint in [edge.source, edge.target] {
                if !restored.contains(&endpoint)
                    && (restoration.current.contains(&endpoint) || read.kind(endpoint)?.is_none())
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
        ownership: ownership.clone(),
        installed_ownership: installed.clone(),
        _meaning_memory: meaning_memory,
        entities,
        restoration,
        current: current_relations,
        selected: selected_relations,
        _current_memory: current_memory,
        _selected_memory: selected_memory,
        _validation_memory: validation_memory,
    })
}
