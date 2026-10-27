//! Bootstrap only the affected before-image, then apply real coupling/vertex deltas.
use super::*;
use worth_execution::{
    ComponentEdit, ComponentPartitioner, ExecutionRequest, MapKernelFailure, PartitionItemId,
    PartitionUpdateDenial,
};
use worth_foundational::PartitionIdentity;

type Pair = (EntityId, EntityId);
struct Candidate {
    graph: ComponentPartitioner,
    items: BTreeMap<EntityId, PartitionItemId>,
    physical: BTreeMap<RelationId, Pair>,
    multiplicities: BTreeMap<Pair, usize>,
}

/// The complete before cohorts bound all reads. A large affected split may grow
/// with that cohort; no unrelated component or branch-wide cache is acquired.
pub(in crate::aggregate_identity::cohorts) fn maintain(
    execution: &ExecutionRequest<'_, '_>,
    seeds: &BTreeSet<EntityId>,
    prior: &[BTreeSet<EntityId>],
    before: &InvariantStateView<'_>,
    after: &InvariantStateView<'_>,
    contract: &AggregateIdentityContractDeclaration,
    effect: &AggregateChanges,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<Vec<BTreeSet<EntityId>>> {
    let mut candidate = Candidate::new(seeds, work)?;
    candidate.seed_before(execution, prior, before, contract, work)?;
    candidate.remove_physical_changes(execution, effect, contract, work)?;
    candidate.set_proposed_members(execution, seeds, after, contract, work)?;
    candidate.apply_created(execution, effect, after, contract, work)?;
    candidate.finish(work)
}

impl Candidate {
    fn seed_before(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        prior: &[BTreeSet<EntityId>],
        before: &InvariantStateView<'_>,
        contract: &AggregateIdentityContractDeclaration,
        work: &mut AggregateWork<'_, '_, '_, '_>,
    ) -> BuildResult<()> {
        for members in prior {
            for &member in members {
                self.edit(
                    execution,
                    ComponentEdit::UpsertItem(self.items[&member]),
                    work,
                )?;
            }
        }
        for members in prior {
            for &member in members {
                for id in outgoing(before, member, work)? {
                    work.checkpoint(1)?;
                    let Some(edge) = before.relation_metadata(id) else {
                        continue;
                    };
                    work.acquired(Acquisition::Relation)?;
                    if !contract.source.contains_relation(edge.kind_id)
                        || self
                            .items
                            .get(&edge.target)
                            .and_then(|item| self.graph.route(*item))
                            .is_none()
                    {
                        continue;
                    }
                    self.add(execution, id, edge.source, edge.target, work)?;
                }
            }
        }
        Ok(())
    }
    fn remove_physical_changes(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        effect: &AggregateChanges,
        contract: &AggregateIdentityContractDeclaration,
        work: &mut AggregateWork<'_, '_, '_, '_>,
    ) -> BuildResult<()> {
        // Remove original physical bindings first, including old endpoints of an
        // updated relation. A transient business edge absent before needs no removal.
        for delta in &effect.adjacency {
            work.checkpoint(1)?;
            if contract.source.contains_relation(delta.kind_id)
                && matches!(delta.kind, AdjacencyDeltaKind::Deleted { .. })
            {
                self.remove(execution, delta.relation_id, work)?;
            }
        }
        Ok(())
    }
    fn set_proposed_members(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        seeds: &BTreeSet<EntityId>,
        after: &InvariantStateView<'_>,
        contract: &AggregateIdentityContractDeclaration,
        work: &mut AggregateWork<'_, '_, '_, '_>,
    ) -> BuildResult<()> {
        // All endpoints are present before newly created couplings are introduced.
        // Deleted members may retain audit relations; those cannot couple live groups.
        for &member in seeds {
            work.checkpoint(1)?;
            let metadata = after.entity_metadata(member);
            if metadata.is_some() {
                work.acquired(Acquisition::Entity)?;
            }
            let live = metadata.is_some_and(|value| value.kind_id == contract.member_kind);
            let item = self.items[&member];
            let edit = if live {
                ComponentEdit::UpsertItem(item)
            } else {
                ComponentEdit::RemoveItem(item)
            };
            self.edit(execution, edit, work)?;
        }
        Ok(())
    }
    fn apply_created(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        effect: &AggregateChanges,
        after: &InvariantStateView<'_>,
        contract: &AggregateIdentityContractDeclaration,
        work: &mut AggregateWork<'_, '_, '_, '_>,
    ) -> BuildResult<()> {
        for delta in &effect.adjacency {
            work.checkpoint(1)?;
            if !contract.source.contains_relation(delta.kind_id)
                || !matches!(delta.kind, AdjacencyDeltaKind::Created { .. })
            {
                continue;
            }
            let Some(edge) = after.relation_metadata(delta.relation_id) else {
                continue;
            };
            work.acquired(Acquisition::Relation)?;
            let AdjacencyDeltaKind::Created { source, target } = delta.kind else {
                unreachable!()
            };
            // Only the actual final physical binding can survive a multi-edit batch.
            if edge.kind_id != delta.kind_id || edge.source != source || edge.target != target {
                continue;
            }
            if self.graph.route(self.items[&source]).is_none()
                || self.graph.route(self.items[&target]).is_none()
            {
                continue;
            }
            self.add(execution, delta.relation_id, source, target, work)?;
        }
        Ok(())
    }
    fn new(
        seeds: &BTreeSet<EntityId>,
        work: &mut AggregateWork<'_, '_, '_, '_>,
    ) -> BuildResult<Self> {
        work.retain(
            (seeds.len() as u64).saturating_mul(extent::tree_entry::<EntityId, PartitionItemId>()),
        )?;
        let mut items = BTreeMap::new();
        for (index, &entity) in seeds.iter().enumerate() {
            work.checkpoint(1)?;
            let index =
                u64::try_from(index).map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
            items.insert(entity, PartitionItemId(index));
        }
        Ok(Self {
            graph: ComponentPartitioner::new(),
            items,
            physical: BTreeMap::new(),
            multiplicities: BTreeMap::new(),
        })
    }
    fn edit(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        edit: ComponentEdit,
        work: &mut AggregateWork<'_, '_, '_, '_>,
    ) -> BuildResult<()> {
        work.poll()?;
        self.graph
            .edit_with_request(execution, edit)
            .map_err(|denial| match denial {
                PartitionUpdateDenial::Stop(stop) => {
                    MapKernelFailure::Domain(PlanDenial::Execution(stop.into()))
                }
                PartitionUpdateDenial::Admission(denial) => MapKernelFailure::Domain(
                    PlanDenial::Execution(crate::execution::denial::lease_denial(denial)),
                ),
                PartitionUpdateDenial::Components(_) | PartitionUpdateDenial::Bisection(_) => {
                    MapKernelFailure::Domain(PlanDenial::InvalidComponentGraph)
                }
                PartitionUpdateDenial::KernelPanic => MapKernelFailure::Panic,
                PartitionUpdateDenial::ResultCapacityExceeded => {
                    MapKernelFailure::ResultCapacityExceeded
                }
            })?;
        // Checked edits have already charged the original request. Never charge
        // their PartitionWork again through this preparation's measured frame.
        work.poll()
    }
    fn add(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        id: RelationId,
        a: EntityId,
        b: EntityId,
        work: &mut AggregateWork<'_, '_, '_, '_>,
    ) -> BuildResult<()> {
        if a == b {
            return Ok(());
        }
        let pair = if a < b { (a, b) } else { (b, a) };
        if let Some(old) = self.physical.get(&id) {
            return if *old == pair {
                Ok(())
            } else {
                work.invalid(PlanDenial::InvalidComponentGraph)
            };
        }
        work.retain(
            extent::tree_entry::<RelationId, Pair>() + extent::tree_entry::<Pair, usize>(),
        )?;
        let count = self.multiplicities.get(&pair).copied().unwrap_or(0);
        let next = count
            .checked_add(1)
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
        if count == 0 {
            self.edit(
                execution,
                ComponentEdit::AddEdge(self.items[&a], self.items[&b]),
                work,
            )?;
        }
        self.multiplicities.insert(pair, next);
        self.physical.insert(id, pair);
        Ok(())
    }
    fn remove(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        id: RelationId,
        work: &mut AggregateWork<'_, '_, '_, '_>,
    ) -> BuildResult<()> {
        let Some(&pair) = self.physical.get(&id) else {
            return Ok(());
        };
        let count = self.multiplicities[&pair];
        if count == 1 {
            self.edit(
                execution,
                ComponentEdit::RemoveEdge(self.items[&pair.0], self.items[&pair.1]),
                work,
            )?;
            self.multiplicities.remove(&pair);
        } else {
            self.multiplicities.insert(pair, count - 1);
        }
        self.physical.remove(&id);
        Ok(())
    }
    fn finish(
        self,
        work: &mut AggregateWork<'_, '_, '_, '_>,
    ) -> BuildResult<Vec<BTreeSet<EntityId>>> {
        let mut groups = BTreeMap::<PartitionIdentity, BTreeSet<EntityId>>::new();
        for (&entity, &item) in &self.items {
            work.checkpoint(1)?;
            let Some(route) = self.graph.route(item) else {
                continue;
            };
            work.retain(
                extent::tree_entry::<PartitionIdentity, BTreeSet<EntityId>>() + extent::seed(),
            )?;
            groups.entry(route).or_default().insert(entity);
        }
        work.retain((groups.len() as u64).saturating_mul(extent::group()))?;
        Ok(groups.into_values().collect())
    }
}
