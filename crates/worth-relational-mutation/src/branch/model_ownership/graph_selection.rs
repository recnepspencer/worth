//! One funded graph boundary; publication requires the original custody owner.
use super::RelationalModelOwnership;
use super::RelationalOwnedModelEntityBoundary;
use super::RelationalOwnedModelForeignRelation;
use super::RelationalOwnedModelSelectionDenial as Denial;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::validation::data::structural_read::StructuralRelationRecord;
use std::collections::{BTreeMap, BTreeSet};
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_branch::internal_port::branch::model_ownership::basis;
use worth_relational_branch::internal_port::branch::model_ownership::traversal::Read;
use worth_relational_branch::internal_port::history::envelope_demand::CanonicalHistoryRead;
pub mod canonical;
pub mod transfers;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelContext};

/// Records outside the owned closure that follow one navigation: tree-based
/// undo's reversal facts (public #56, ruling (u)). The caller names the
/// records the steps it leaves posted (`retire`, live on the current basis)
/// and the records the steps it enters posted (`restore`, live on the
/// selected basis, rematerialized in their original full-generation slots).
/// Each must be relation-free and outside the owned closure; preparation
/// checks both on the exact bases and grants no other authority.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RelationalOwnedGraphCompanions {
    pub retire: BTreeSet<EntityId>,
    pub restore: BTreeSet<EntityId>,
}

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
    companions: RelationalOwnedGraphCompanions,
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
    /// Companion records this selection retires and rematerializes.
    pub fn companions(&self) -> &RelationalOwnedGraphCompanions {
        &self.companions
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

fn in_union(entities: &RelationalOwnedModelEntityBoundary, id: EntityId) -> bool {
    entities.current_entities().contains(&id) || entities.selected_entities().contains(&id)
}
fn declared(
    read: &mut basis::BasisRead<'_, '_, '_, '_, '_, '_>,
    ownership: &RelationalModelOwnership,
    edge: StructuralRelationRecord,
) -> Result<bool, Denial> {
    read.charge(ownership.plan.relation_lookup_work())?;
    Ok(ownership.plan.declares_relation(edge.kind_id))
}
fn insert(
    read: &mut basis::BasisRead<'_, '_, '_, '_, '_, '_>,
    map: &mut BTreeMap<RelationId, StructuralRelationRecord>,
    edge: StructuralRelationRecord,
) -> Result<(), Denial> {
    if let std::collections::btree_map::Entry::Vacant(entry) = map.entry(edge.relation_id) {
        read.retain()?;
        entry.insert(edge);
    }
    Ok(())
}
#[allow(clippy::too_many_arguments)]
pub fn prepare(
    history: CanonicalHistoryRead<'_>,
    execution: &ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
    entities: RelationalOwnedModelEntityBoundary,
    ownership: &RelationalModelOwnership,
    installed: &RelationalModelOwnership,
    meaning_memory: Option<ExecutionMemoryReservation>,
    companions: RelationalOwnedGraphCompanions,
) -> Result<RelationalOwnedModelGraphSelection, Denial> {
    let current = entities.current_basis();
    let selected = entities.selected_basis();
    let restoration = transfers::prepare(history, execution, context, &entities, installed)?;
    let restored = &restoration.selected;
    let transferred = &restoration.transferred;
    let in_domain = |id: EntityId| restoration.current.contains(&id) || restored.contains(&id);
    let (current_relations, current_memory) = basis::read(
        history.branch.runtime_instance_id,
        execution,
        current,
        context,
        |read| {
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
        },
    )?;
    let (selected_relations, selected_memory) = basis::read(
        history.branch.runtime_instance_id,
        execution,
        selected,
        context,
        |read| {
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
        },
    )?;
    let ((), validation_memory) = basis::read(
        history.branch.runtime_instance_id,
        execution,
        current,
        context,
        |read| {
            for edge in selected_relations.values() {
                if let Some(now) = read.relation(edge.relation_id)?
                    && !in_domain(now.source)
                    && !(declared(read, installed, now)? && in_domain(now.target))
                {
                    return Err(Denial::ForeignRelationSource {
                        relation: now.relation_id,
                        owner: now.source,
                    });
                }
                if let Some(&owner) = transferred.get(&edge.target)
                    && !restored.contains(&edge.source)
                {
                    if read.kind(edge.source)?.is_none() {
                        return Err(Denial::SelectedOwnerUnavailable {
                            entity: edge.target,
                            owner,
                            source: edge.source,
                            relation: edge.relation_id,
                        });
                    }
                    let actual_owner = installed.plan.owner(read, edge.source)?;
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
                for endpoint in [edge.source, edge.target] {
                    if !restored.contains(&endpoint)
                        && (restoration.current.contains(&endpoint)
                            || read.kind(endpoint)?.is_none())
                    {
                        return Err(Denial::SelectedEndpointUnavailable {
                            relation: edge.relation_id,
                            endpoint,
                        });
                    }
                }
            }
            companion_check(read, &companions, &in_domain, true)
        },
    )?;
    basis::read(history.branch.runtime_instance_id, execution, selected, context, |read| {
        companion_check(read, &companions, &in_domain, false)
    })?;
    Ok(RelationalOwnedModelGraphSelection {
        ownership: ownership.clone(),
        installed_ownership: installed.clone(),
        _meaning_memory: meaning_memory,
        entities,
        restoration,
        companions,
        current: current_relations,
        selected: selected_relations,
        _current_memory: current_memory,
        _selected_memory: selected_memory,
        _validation_memory: validation_memory,
    })
}

/// One basis's side of the companion contract. On the current basis each
/// retired record is live and each restored record is not; on the selected
/// basis each restored record is live. Every live companion is relation-free
/// and outside the owned closure.
fn companion_check(
    read: &mut basis::BasisRead<'_, '_, '_, '_, '_, '_>,
    companions: &RelationalOwnedGraphCompanions,
    in_domain: &impl Fn(EntityId) -> bool,
    current: bool,
) -> Result<(), Denial> {
    for &entity in companions.retire.iter().chain(&companions.restore) {
        if in_domain(entity)
            || (companions.retire.contains(&entity) && companions.restore.contains(&entity))
        {
            return Err(Denial::CompanionOverlap { entity });
        }
        let live = read.kind(entity)?.is_some();
        let expected = current == companions.retire.contains(&entity);
        if live != expected {
            return Err(if live {
                Denial::CompanionOverlap { entity }
            } else {
                Denial::CompanionUnavailable { entity }
            });
        }
        if live
            && !(read.relations(entity, true)?.is_empty()
                && read.relations(entity, false)?.is_empty())
        {
            return Err(Denial::CompanionRelated { entity });
        }
    }
    Ok(())
}
