//! Complete affected graph cohorts using Native frontier mechanics.
use std::collections::{BTreeMap, BTreeSet};

use super::changes::AggregateChanges;
use crate::authority::mutation::AdjacencyDeltaKind;
use crate::graph_member::NativeMemberKey as Member;
use crate::identity::data::{EntityId, RelationId};
use crate::schema::data::{
    AggregateIdentityContractDeclaration, AggregateIdentitySource, RelationalSchemaRegistry,
};
mod checked_components;
mod components;
mod derived;
mod stored;
use crate::structural_traversal::{walk as walk_native_breadth_first, BreadthFirstRead};
use crate::transactions::data::RecordRef;
use crate::validation::engine::state_view::InvariantStateView;
pub(super) use stored::read_stored_group;
use worth_foundational::facade::AggregateIdentity;

use super::observations::Acquisition;
use super::preparation::{AggregateWork, BuildResult, PlanDenial};
use super::record::AggregateRecord;
use super::retention_extent as extent;
use super::PriorAggregate;

#[derive(Clone)]
pub(super) struct StoredAggregate {
    pub(super) entity: EntityId,
    pub(super) record: AggregateRecord,
}
pub(super) struct CollectedCohorts {
    pub(super) prior: Vec<PriorAggregate>,
    pub(super) proposed: Vec<BTreeSet<Member>>,
    pub(super) rows: BTreeMap<AggregateIdentity, StoredAggregate>,
    pub(super) memberships: BTreeMap<Member, (RelationId, AggregateIdentity)>,
}

pub(super) fn collect(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    contract: &AggregateIdentityContractDeclaration,
    registry: &RelationalSchemaRegistry,
    before: &InvariantStateView<'_>,
    after: &InvariantStateView<'_>,
    effect: &AggregateChanges,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<Option<CollectedCohorts>> {
    if contract.source.derived_relation().is_some() {
        return derived::collect(execution, contract, registry, before, after, effect, work);
    }
    let mut seeds = BTreeSet::new();
    let mut additions: BTreeMap<EntityId, BTreeSet<RelationId>> = BTreeMap::new();
    for delta in &effect.adjacency {
        work.checkpoint(1)?;
        if !contract.source.contains_stored_relation(delta.kind_id) {
            continue;
        }
        let (source, target) = match delta.kind {
            AdjacencyDeltaKind::Created { source, target }
            | AdjacencyDeltaKind::Deleted { source, target } => (source, target),
        };
        work.retain(extent::affected_delta())?;
        seeds.extend([source, target]);
        if matches!(delta.kind, AdjacencyDeltaKind::Created { .. }) {
            additions
                .entry(source)
                .or_default()
                .insert(delta.relation_id);
        }
    }
    for record in &effect.records {
        work.checkpoint(1)?;
        let RecordRef::Entity(entity) = record else {
            continue;
        };
        let old = before.entity_metadata(*entity);
        if old.is_some() {
            work.acquired(Acquisition::Entity)?;
        }
        let new = after.entity_metadata(*entity);
        if new.is_some() {
            work.acquired(Acquisition::Entity)?;
        }
        if old
            .as_ref()
            .is_some_and(|entry| entry.kind_id == contract.member_kind)
            != new
                .as_ref()
                .is_some_and(|entry| entry.kind_id == contract.member_kind)
        {
            work.retain(extent::seed())?;
            seeds.insert(*entity);
        }
    }
    if seeds.is_empty() {
        return Ok(None);
    }
    let prior_groups = groups(&seeds, before, contract, &BTreeMap::new(), work)?;
    // A severed component can contain no directly named endpoint after deletion.
    for group in &prior_groups {
        for &member in group {
            work.retain(extent::seed())?;
            seeds.insert(member);
        }
    }
    let proposed = match contract.source {
        AggregateIdentitySource::Orbit(_) => groups(&seeds, after, contract, &additions, work)?,
        AggregateIdentitySource::Components { .. } => components::maintain(
            execution,
            &seeds,
            &prior_groups,
            before,
            after,
            contract,
            effect,
            work,
        )?,
        AggregateIdentitySource::DerivedOrbit { .. }
        | AggregateIdentitySource::DerivedComponents { .. } => {
            return work.invalid(PlanDenial::DefinitionUnavailable)
        }
    };
    let proposed = proposed
        .into_iter()
        .map(|group| tagged_group(group, work))
        .collect::<BuildResult<Vec<_>>>()?;
    let mut collected = CollectedCohorts {
        prior: Vec::new(),
        proposed,
        rows: BTreeMap::new(),
        memberships: BTreeMap::new(),
    };
    for members in prior_groups {
        let members = tagged_group(members, work)?;
        let stored = read_stored_group(
            before,
            &members,
            contract,
            registry,
            &mut collected.memberships,
            work,
        )?;
        if collected.rows.contains_key(&stored.record.identity) {
            return work.invalid(PlanDenial::InvalidStoredIdentity);
        }
        collected.prior.push(PriorAggregate {
            identity: stored.record.identity,
            members,
        });
        collected.rows.insert(stored.record.identity, stored);
    }
    Ok(Some(collected))
}

/// Physical discovery keeps its fixed-width source path. Only the common
/// coordinator receives tagged keys; no projected case is coerced to EntityId.
fn tagged_group(
    group: BTreeSet<EntityId>,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<BTreeSet<Member>> {
    work.retain(
        (group.len() as u64)
            .checked_mul(extent::tree_entry::<Member, ()>())
            .ok_or(worth_execution::MapKernelFailure::ResultCapacityExceeded)?,
    )?;
    Ok(group.into_iter().map(Member::Entity).collect())
}

pub(super) fn member_groups(
    seeds: &BTreeSet<Member>,
    view: &InvariantStateView<'_>,
    contract: &AggregateIdentityContractDeclaration,
    registry: &RelationalSchemaRegistry,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<Vec<BTreeSet<Member>>> {
    if contract.source.derived_relation().is_some() {
        return derived::groups(registry, contract, seeds, view, work);
    }
    let mut physical = BTreeSet::new();
    for member in seeds {
        let Some(entity) = member.entity() else {
            return work.invalid(PlanDenial::WrongMemberKind);
        };
        work.retain(extent::seed())?;
        physical.insert(entity);
    }
    groups(&physical, view, contract, &BTreeMap::new(), work)?
        .into_iter()
        .map(|group| tagged_group(group, work))
        .collect()
}

pub(super) fn groups(
    seeds: &BTreeSet<EntityId>,
    view: &InvariantStateView<'_>,
    contract: &AggregateIdentityContractDeclaration,
    additions: &BTreeMap<EntityId, BTreeSet<RelationId>>,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<Vec<BTreeSet<EntityId>>> {
    if contract.source.stored_relations().is_none() {
        return work.invalid(PlanDenial::DefinitionUnavailable);
    }
    if matches!(contract.source, AggregateIdentitySource::Components { .. }) {
        return components::groups(seeds, view, contract, work);
    }
    let mut visited = BTreeSet::new();
    let mut groups = Vec::new();
    for &seed in seeds {
        work.checkpoint(1)?;
        if visited.contains(&seed) {
            continue;
        }
        if view.entity_metadata(seed).is_none() {
            continue;
        }
        work.acquired(Acquisition::Entity)?;
        let members = walk_native_breadth_first(
            &[seed],
            &mut OrbitRead {
                view,
                contract,
                additions,
                work,
            },
        )?;
        for &member in &members {
            work.retain(extent::seed())?;
            visited.insert(member);
        }
        work.retain(extent::group())?;
        groups.push(members);
    }
    Ok(groups)
}

struct OrbitRead<'a, 'state, 'work, 'scope, 'authority, 'memory> {
    view: &'a InvariantStateView<'state>,
    contract: &'a AggregateIdentityContractDeclaration,
    additions: &'a BTreeMap<EntityId, BTreeSet<RelationId>>,
    work: &'a mut AggregateWork<'work, 'scope, 'authority, 'memory>,
}
impl BreadthFirstRead for OrbitRead<'_, '_, '_, '_, '_, '_> {
    type Error = worth_execution::MapKernelFailure<PlanDenial>;
    fn seed(&mut self, _: EntityId) -> BuildResult<()> {
        self.work.checkpoint(1)
    }
    fn retain(&mut self, _: EntityId, _: usize) -> BuildResult<()> {
        self.work.retain(extent::frontier_vertex())
    }
    fn expand(
        &mut self,
        entity: EntityId,
        _: usize,
        mut enqueue: impl FnMut(&mut Self, EntityId) -> BuildResult<()>,
    ) -> BuildResult<()> {
        self.work.checkpoint(1)?;
        let metadata = self.view.entity_metadata(entity);
        if metadata.is_some() {
            self.work.acquired(Acquisition::Entity)?;
        }
        if metadata.is_none_or(|entry| entry.kind_id != self.contract.member_kind) {
            return self.work.invalid(PlanDenial::InvalidPermutation);
        }
        let ids = outgoing(self.view, entity, self.work)?;
        let mut edges = BTreeSet::new();
        for id in ids
            .into_iter()
            .chain(self.additions.get(&entity).into_iter().flatten().copied())
        {
            self.work.checkpoint(1)?;
            let Some(edge) = self.view.relation_metadata(id) else {
                continue;
            };
            self.work.acquired(Acquisition::Relation)?;
            if Some(edge.kind_id) == self.contract.source.orbit_relation() && edge.source == entity
            {
                self.work.retain(extent::successor_edge())?;
                edges.insert((id, edge.target));
            }
        }
        if edges.len() != 1 {
            return self.work.invalid(PlanDenial::InvalidPermutation);
        }
        enqueue(self, edges.first().expect("exactly one successor").1)
    }
}

pub(super) fn outgoing(
    view: &InvariantStateView<'_>,
    entity: EntityId,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<Vec<RelationId>> {
    adjacency(view, entity, true, work)
}

pub(super) fn adjacency(
    view: &InvariantStateView<'_>,
    entity: EntityId,
    outgoing: bool,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<Vec<RelationId>> {
    let count = view.relation_candidate_count(entity, outgoing) as u64;
    work.retain(
        count
            .saturating_mul(extent::relation_candidate())
            .saturating_add(extent::empty_candidate()),
    )?;
    work.checkpoint(count.saturating_mul(8).saturating_add(1))?;
    let ids = if outgoing {
        view.outgoing_relations_for_entity_observed(entity, || {
            work.acquired(Acquisition::Relation)
        })?
    } else {
        view.incoming_relations_for_entity_observed(entity, || {
            work.acquired(Acquisition::Relation)
        })?
    };
    work.acquired(Acquisition::Adjacency)?;
    Ok(ids)
}
